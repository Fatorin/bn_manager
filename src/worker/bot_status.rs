use std::sync::RwLock;
use std::time::Instant;

use once_cell::sync::Lazy;
use reqwest::header::{ETAG, IF_NONE_MATCH};
use reqwest::{Client, StatusCode};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{info, warn};

use crate::model::room::{BotGame, BotRooms, BotStatus};
use crate::settings::CONFIG;

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
/// A bot whose last good answer is older than this is shown as offline.
const STALE_AFTER: Duration = Duration::from_secs(10);

struct BotSnapshot {
    bot: String,
    games: Vec<BotGame>,
    updated: Instant,
}

/// One slot per entry of `bot_status_urls`, in config order.
static SNAPSHOTS: Lazy<RwLock<Vec<Option<BotSnapshot>>>> =
    Lazy::new(|| RwLock::new((0..CONFIG.bot_status_urls.len()).map(|_| None).collect()));

pub fn start_bot_status_worker(shutdown_tx: &broadcast::Sender<()>) {
    let client = match Client::builder().timeout(REQUEST_TIMEOUT).build() {
        Ok(client) => client,
        Err(e) => {
            warn!("Bot status worker: failed to build HTTP client: {}", e);
            return;
        }
    };

    for (index, url) in CONFIG.bot_status_urls.iter().enumerate() {
        let client = client.clone();
        let url = url.clone();
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            let mut ticker = interval(POLL_INTERVAL);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            let mut etag: Option<String> = None;
            let mut was_ok = true;
            loop {
                tokio::select! {
                    _ = ticker.tick() => {},
                    _ = shutdown_rx.recv() => break,
                }

                match poll(&client, &url, index, &mut etag).await {
                    Ok(()) => {
                        if !was_ok {
                            info!("Bot status {} is reachable again", url);
                        }
                        was_ok = true;
                    }
                    Err(e) => {
                        // Bots are only up on schedule, so log the transition, not every miss.
                        if was_ok {
                            warn!("Bot status {} unreachable: {}", url, e);
                        }
                        was_ok = false;
                        etag = None;
                    }
                }
            }
            info!("Bot status worker for {} shutdown complete", url);
        });
    }
}

async fn poll(client: &Client, url: &str, index: usize, etag: &mut Option<String>) -> Result<(), String> {
    let mut request = client.get(url);
    if let Some(tag) = etag.as_deref() {
        request = request.header(IF_NONE_MATCH, tag);
    }
    let response = request.send().await.map_err(|e| e.to_string())?;

    if response.status() == StatusCode::NOT_MODIFIED {
        if let Some(snapshot) = SNAPSHOTS.write().unwrap()[index].as_mut() {
            snapshot.updated = Instant::now();
            return Ok(());
        }
        // 304 without a stored body (shouldn't happen); refetch unconditionally next tick.
        *etag = None;
        return Err("304 without a cached snapshot".to_string());
    }
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let new_etag = response
        .headers()
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let status: BotStatus = response.json().await.map_err(|e| e.to_string())?;

    let games = status
        .games
        .into_iter()
        .map(|mut game| {
            // "Maps\\Download\\dota.w3x" -> "dota.w3x"; the folder layout is internal.
            if let Some(file) = game.map.rsplit(['\\', '/']).next() {
                game.map = file.to_string();
            }
            game
        })
        .collect();

    SNAPSHOTS.write().unwrap()[index] = Some(BotSnapshot {
        bot: status.bot,
        games,
        updated: Instant::now(),
    });
    *etag = new_etag;
    Ok(())
}

/// Bots in config order. A bot that has never answered is left out, since its name is unknown.
pub fn bot_rooms() -> Vec<BotRooms> {
    SNAPSHOTS
        .read()
        .unwrap()
        .iter()
        .flatten()
        .map(|snapshot| {
            let online = snapshot.updated.elapsed() < STALE_AFTER;
            BotRooms {
                bot: snapshot.bot.clone(),
                online,
                games: if online { snapshot.games.clone() } else { Vec::new() },
            }
        })
        .collect()
}
