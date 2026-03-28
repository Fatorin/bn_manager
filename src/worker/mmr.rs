use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use sqlx::MySqlPool;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

use crate::database::mysql_pool;

static IS_PROCESSING: AtomicBool = AtomicBool::new(false);

pub fn start_mmr_worker(mut shutdown_rx: broadcast::Receiver<()>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            tokio::select! {
                _ = ticker.tick() => {},
                _ = shutdown_rx.recv() => {
                    info!("MMR worker received shutdown signal");
                    break;
                }
            }

            if IS_PROCESSING
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                warn!("MMR worker: previous round not finished, skipping");
                continue;
            }
            let result = process_all_mmr().await;
            IS_PROCESSING.store(false, Ordering::SeqCst);
            if let Err(e) = result {
                error!("MMR worker error: {}", e);
            }
        }
        info!("MMR worker shutdown complete");
    });
}

async fn process_all_mmr() -> Result<(), sqlx::Error> {
    let pool = mysql_pool();
    let ids = get_unprocessed_game_ids(pool).await?;
    if ids.is_empty() {
        return Ok(());
    }
    info!("MMR worker: processing {} unprocessed games", ids.len());
    for id in ids {
        if let Err(e) = process_game_mmr(pool, id).await {
            error!("Game {} MMR processing failed: {}", id, e);
        }
    }
    Ok(())
}

async fn get_unprocessed_game_ids(pool: &MySqlPool) -> Result<Vec<i32>, sqlx::Error> {
    let ids = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT g.id
        FROM games g
        LEFT JOIN game_mmr_processed p ON g.id = p.gameid
        WHERE p.id IS NULL
        ORDER BY g.id
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(ids)
}

// ── Data structs ──

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
struct GamePlayer {
    pid: i32,
    name: String,
    category: String,
    flag: String,
    servant_raw: Option<String>,
}

#[derive(Debug, Clone)]
struct MmrContext {
    player_current_mmr: f64,
    player_team_avg_mmr: f64,
    opponent_team_avg_mmr: f64,
    is_winner: bool,
}

#[derive(Debug, Clone)]
struct MmrResult {
    delta: f64,
    new_mmr: f64,
}

// ── Pure calculation ──

fn calculate_mmr(ctx: &MmrContext) -> MmrResult {
    // Elo-based MMR system
    //
    // K-factor: base points at stake per game.
    // Higher K = faster rating movement, lower K = more stable ratings.
    const K: f64 = 32.0;

    // Expected win probability using logistic function:
    //   E = 1 / (1 + 10^((opponent_avg - team_avg) / 400))
    //
    // When teams are equal: E = 0.5
    // When team is 200 pts stronger: E ≈ 0.76
    // When team is 200 pts weaker:   E ≈ 0.24
    let rating_diff = ctx.opponent_team_avg_mmr - ctx.player_team_avg_mmr;
    let expected = 1.0 / (1.0 + 10.0_f64.powf(rating_diff / 400.0));

    let actual = if ctx.is_winner { 1.0 } else { 0.0 };

    // Delta = K * (actual - expected)
    //
    // Examples (equal teams, expected=0.5):
    //   winner: 32 * (1.0 - 0.5) = +16
    //   loser:  32 * (0.0 - 0.5) = -16
    //
    // Weak team (expected=0.24) beats strong team:
    //   winner: 32 * (1.0 - 0.24) = +24.3  (big reward)
    //   loser:  32 * (0.0 - 0.76) = -24.3  (big penalty)
    //
    // Strong team (expected=0.76) beats weak team:
    //   winner: 32 * (1.0 - 0.76) = +7.7   (small reward)
    //   loser:  32 * (0.0 - 0.24) = -7.7   (small penalty)
    let delta = K * (actual - expected);
    let new_mmr = (ctx.player_current_mmr + delta).max(0.0);

    MmrResult { delta, new_mmr }
}

fn parse_team_index(servant_raw: &Option<String>) -> Option<i32> {
    let s = servant_raw.as_ref()?;
    let cleaned = s.trim_matches('"');
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() != 2 {
        warn!("servant info format invalid: {}", s);
        return None;
    }
    parts[0].parse::<i32>().ok()
}

// ── Computation (pure, no DB) ──

struct PlayerMmrUpdate {
    name: String,
    category: String,
    flag: String,
    old_mmr: f64,
    result: MmrResult,
    team_avg_mmr: f64,
    opponent_avg_mmr: f64,
}

fn compute_all_mmr_updates(
    players: &[GamePlayer],
    current_scores: &HashMap<(String, String), f64>,
) -> Vec<PlayerMmrUpdate> {
    const DEFAULT_MMR: f64 = 1000.0;

    // Resolve each player's current MMR
    let player_mmrs: Vec<f64> = players
        .iter()
        .map(|p| {
            *current_scores
                .get(&(p.category.clone(), p.name.clone()))
                .unwrap_or(&DEFAULT_MMR)
        })
        .collect();

    // Group by team
    let mut team_mmrs: HashMap<i32, Vec<f64>> = HashMap::new();
    let mut player_team_ids: Vec<Option<i32>> = Vec::new();

    for (i, p) in players.iter().enumerate() {
        let team_id = parse_team_index(&p.servant_raw);
        player_team_ids.push(team_id);
        if let Some(tid) = team_id {
            team_mmrs.entry(tid).or_default().push(player_mmrs[i]);
        }
    }

    // Compute team averages
    let team_avgs: HashMap<i32, f64> = team_mmrs
        .iter()
        .map(|(&tid, mmrs)| {
            let avg = mmrs.iter().sum::<f64>() / mmrs.len() as f64;
            (tid, avg)
        })
        .collect();

    // Build updates
    let mut updates = Vec::new();
    for (i, p) in players.iter().enumerate() {
        let current_mmr = player_mmrs[i];
        let team_id = player_team_ids[i];

        let player_team_avg = team_id
            .and_then(|tid| team_avgs.get(&tid).copied())
            .unwrap_or(current_mmr);

        // Opponent team avg: average of all other teams
        let opponent_avg = if let Some(tid) = team_id {
            let opponent_mmrs: Vec<f64> = team_avgs
                .iter()
                .filter(|(&t, _)| t != tid)
                .map(|(_, &avg)| avg)
                .collect();
            if opponent_mmrs.is_empty() {
                player_team_avg
            } else {
                opponent_mmrs.iter().sum::<f64>() / opponent_mmrs.len() as f64
            }
        } else {
            current_mmr
        };

        let ctx = MmrContext {
            player_current_mmr: current_mmr,
            player_team_avg_mmr: player_team_avg,
            opponent_team_avg_mmr: opponent_avg,
            is_winner: p.flag == "winner",
        };

        let result = calculate_mmr(&ctx);

        updates.push(PlayerMmrUpdate {
            name: p.name.clone(),
            category: p.category.clone(),
            flag: p.flag.clone(),
            old_mmr: current_mmr,
            result,
            team_avg_mmr: player_team_avg,
            opponent_avg_mmr: opponent_avg,
        });
    }

    updates
}

// ── Main processing ──

async fn process_game_mmr(pool: &MySqlPool, game_id: i32) -> Result<(), sqlx::Error> {
    // Stage 1: Fetch all data

    let players = sqlx::query_as::<_, GamePlayer>(
        r#"
        SELECT
          p.pid,
          p.name,
          p.category,
          p.flag,
          v.value_string AS servant_raw
        FROM w3mmdplayers p
        LEFT JOIN w3mmdvars v ON v.gameid = p.gameid AND v.pid = p.pid AND v.varname = 'servant'
        WHERE p.gameid = ?
          AND p.flag IN ('winner', 'loser')
        "#,
    )
    .bind(game_id)
    .fetch_all(pool)
    .await?;

    if players.is_empty() {
        // Mark as processed even if no eligible players
        let mut tx = pool.begin().await?;
        sqlx::query("INSERT INTO game_mmr_processed (gameid, processed_at) VALUES (?, ?)")
            .bind(game_id)
            .bind(Utc::now().naive_utc())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(());
    }

    let server = sqlx::query_scalar::<_, String>(
        "SELECT server FROM games WHERE id = ? LIMIT 1",
    )
    .bind(game_id)
    .fetch_one(pool)
    .await?;

    // Fetch current scores for all players in this game
    let mut current_scores: HashMap<(String, String), f64> = HashMap::new();
    for p in &players {
        if current_scores.contains_key(&(p.category.clone(), p.name.clone())) {
            continue;
        }
        let score = sqlx::query_scalar::<_, f64>(
            "SELECT score FROM scores WHERE category = ? AND name = ? AND server = ? LIMIT 1",
        )
        .bind(&p.category)
        .bind(&p.name)
        .bind(&server)
        .fetch_optional(pool)
        .await?;

        if let Some(s) = score {
            current_scores.insert((p.category.clone(), p.name.clone()), s);
        }
    }

    // Stage 2: Compute (pure)
    let updates = compute_all_mmr_updates(&players, &current_scores);

    // Stage 3: Persist
    let mut tx = pool.begin().await?;
    let now = Utc::now().naive_utc();

    for u in &updates {
        let is_new = !current_scores.contains_key(&(u.category.clone(), u.name.clone()));

        if is_new {
            sqlx::query(
                "INSERT INTO scores (category, name, server, score) VALUES (?, ?, ?, ?)",
            )
            .bind(&u.category)
            .bind(&u.name)
            .bind(&server)
            .bind(u.result.new_mmr)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                "UPDATE scores SET score = ? WHERE category = ? AND name = ? AND server = ?",
            )
            .bind(u.result.new_mmr)
            .bind(&u.category)
            .bind(&u.name)
            .bind(&server)
            .execute(&mut *tx)
            .await?;
        }

        // Log the change
        sqlx::query(
            r#"
            INSERT INTO score_change_logs
                (gameid, category, name, server, mmr_before, mmr_after, mmr_delta,
                 result_flag, team_avg_mmr, opponent_avg_mmr, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(game_id)
        .bind(&u.category)
        .bind(&u.name)
        .bind(&server)
        .bind(u.old_mmr)
        .bind(u.result.new_mmr)
        .bind(u.result.delta)
        .bind(&u.flag)
        .bind(u.team_avg_mmr)
        .bind(u.opponent_avg_mmr)
        .bind(now)
        .execute(&mut *tx)
        .await?;

        info!(
            "MMR update: {} ({}) delta={:+.1} new_mmr={:.1}",
            u.name, u.flag, u.result.delta, u.result.new_mmr
        );
    }

    sqlx::query("INSERT INTO game_mmr_processed (gameid, processed_at) VALUES (?, ?)")
        .bind(game_id)
        .bind(now)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    info!(
        "Completed MMR processing GameID {} ({} players)",
        game_id,
        updates.len()
    );
    Ok(())
}
