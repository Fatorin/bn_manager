use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use tracing::debug;
use crate::model::room::{RoomInfo, RoomStatus};
use crate::settings::CONFIG;
use crate::worker::bot_status::bot_rooms;

pub async fn room_info() -> impl IntoResponse {
    let bots = (!CONFIG.bot_status_urls.is_empty()).then(bot_rooms);

    // Bot-hosted games are also advertised on PVPGN; list each of them only once, under its bot.
    let bot_game_names: HashSet<String> = bots
        .iter()
        .flatten()
        .flat_map(|bot| bot.games.iter())
        .map(|game| normalize_name(&game.name))
        .collect();

    let mut pvpgn_error = false;
    let pvpgn = if CONFIG.bn_log_path.is_empty() {
        None
    } else {
        match read_pvpgn_rooms() {
            Ok(rooms) => Some(
                rooms
                    .into_iter()
                    .filter(|room| !bot_game_names.contains(&normalize_name(&room.room_name)))
                    .collect(),
            ),
            Err(e) => {
                debug!("Failed to read PVPGN status file {}: {}", CONFIG.bn_log_path, e);
                pvpgn_error = true;
                Some(Vec::new())
            }
        }
    };

    (StatusCode::OK, Json(RoomStatus {
        server_time: chrono::Utc::now().timestamp(),
        bots,
        pvpgn,
        pvpgn_error,
    }))
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn read_pvpgn_rooms() -> std::io::Result<Vec<RoomInfo>> {
    let file = fs::File::open(&CONFIG.bn_log_path)?;

    let mut hash_map: HashMap<u16, RoomInfo> = HashMap::new();

    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = match line {
            Ok(text) => text,
            Err(_) => continue
        };

        let lines: Vec<&str> = line.split(',').collect();
        match lines.len() {
            3 => {
                let Ok(_room_id) = lines[1].parse::<u16>() else { continue };
                let room_info = RoomInfo {
                    room_id: _room_id,
                    room_name: lines[2].to_string(),
                    player_count: 0,
                };
                hash_map.insert(_room_id, room_info);
            }
            5 => {
                let Ok(_room_id) = lines[4].parse::<u16>() else { continue };
                if let Some(room) = hash_map.get_mut(&_room_id) {
                    room.player_count = room.player_count.saturating_add(1);
                }
            }
            _ => {}
        }
    }

    let mut result: Vec<RoomInfo> = hash_map.into_values().collect();
    result.sort_by(|a, b| a.room_id.cmp(&b.room_id));
    Ok(result)
}
