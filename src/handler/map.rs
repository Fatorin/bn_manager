use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use axum::extract::Multipart;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use axum::response::IntoResponse;
use serde_json::json;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use crate::model::map::MapInfo;
use crate::settings::CONFIG;
use crate::util::file;

pub async fn get_maps(state: Extension<Arc<Mutex<HashMap<String, MapInfo>>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let values: Vec<MapInfo> = state.values().cloned().collect();
    return (StatusCode::OK, Json(json!(values)));
}

pub async fn upload_map(state: Extension<Arc<Mutex<HashMap<String, MapInfo>>>>, headers: HeaderMap, mut multipart: Multipart) -> impl IntoResponse {
    if let Some(header_value) = headers.get("X-API-KEY") {
        let valid_code = header_value.to_str().unwrap_or("").to_string();
        if CONFIG.map_valid_code != valid_code {
            println!("Valid code is wrong");
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Valid code is wrong"})));
        }
    } else {
        println!("Valid code not found");
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Valid code not found"})));
    }

    while let Some(field) = multipart.next_field().await.unwrap() {
        let file_name = field.file_name().unwrap().to_string();
        if file_name.is_empty() {
            println!("Not found map name");
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Not found map name"})));
        }

        let exist_result = file::check_exist(&CONFIG.map_path, &file_name);
        if !exist_result.1 {
            println!("Has same name map");
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Has same name map"})));
        }

        let data = field.bytes().await.unwrap();
        if &data[..4] != b"HM3W" {
            println!("Invalid file format");
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid file format"})));
        }

        let file_path = Path::new(&CONFIG.map_path).join(&file_name);
        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(&data).await.unwrap();

        if let Ok(map_info) = file::analysis_w3x_name(file_name.to_string(), &data) {
            let mut state = state.lock().await;
            state.insert(file_name, map_info);
        } else {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "Can't analysis this map"})));
        }
    }

    return (StatusCode::OK, Json(json!({"ok": "Upload init successful"})));
}