use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;
use crate::model::room::RoomInfo;
use crate::settings::CONFIG;

pub async fn room_info() -> impl IntoResponse {
    let file = match fs::File::open(&CONFIG.bn_log_path) {
        Ok(file) => file,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "can't read log"})))
    };

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
                let _room_id: u16 = lines[1].parse().unwrap();
                let room_info = RoomInfo {
                    room_id: _room_id,
                    room_name: lines[2].to_string(),
                    player_count: 0,
                };
                hash_map.insert(_room_id, room_info);
            }
            5 => {
                let _room_id: u16 = lines[4].parse().unwrap();
                if let Some(room) = hash_map.get_mut(&_room_id) {
                    room.player_count += 1;
                }
            }
            _ => {}
        }
    }

    let mut result: Vec<RoomInfo> = hash_map.into_values().collect();
    result.sort_by(|a, b| a.room_id.cmp(&b.room_id));
    return (StatusCode::OK, Json(json!(result)));
}