use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use axum::{Json, Router};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum_server::tls_rustls::RustlsConfig;
use config::Config;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let settings = Config::builder()
        .add_source(config::File::with_name("settings.toml"))
        .build()
        .unwrap();

    let runtime_mode = settings.get_string("RUNTIME_MODE").unwrap_or_else(|_| "dev".to_string());
    let certs_path = settings.get_string("CERTS_PATH").unwrap_or_else(|_| "certs".to_string());

    let state = Arc::new(AppState { config: settings });

    let routes_static = Router::new()
        .nest_service("/", ServeDir::new("static"));

    let routes_apis = Router::new()
        .route("/add_user", post(add_user))
        .route("/room_info", get(room_info))
        .with_state(state);

    let app = Router::new()
        .merge(routes_static)
        .merge(routes_apis);

    match runtime_mode.as_str() {
        "dev" => {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
        "prod" => {
            let tls_config = RustlsConfig::from_pem_file(
                Path::new(&certs_path).join("cert.pem"),
                Path::new(&certs_path).join("key.pem"),
            ).await.unwrap();

            let addr = SocketAddr::from(([0, 0, 0, 0], 443));
            axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        _ => {
            panic!("not supported runtime mode")
        }
    }
}

async fn room_info(state: State<Arc<AppState>>) -> impl IntoResponse {
    let log_file_path = match state.config.get_string("BN_LOG_PATH") {
        Ok(path) => path,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Server didn't set log path"})))
    };

    let file = match fs::File::open(log_file_path) {
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

        let datas: Vec<&str> = line.split(',').collect();
        match datas.len() {
            3 => {
                let _room_id: u16 = datas[1].parse().unwrap();
                let room_info = RoomInfo {
                    room_id: _room_id,
                    room_name: datas[2].to_string(),
                    player_count: 0,
                };
                hash_map.insert(_room_id, room_info);
            }
            5 => {
                let _room_id: u16 = datas[4].parse().unwrap();
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

async fn add_user(state: State<Arc<AppState>>, req: Json<AddUserReq>) -> impl IntoResponse {
    let valid_code = match state.config.get_string("VALID_CODE") {
        Ok(code) => code,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Server didn't set valid code"})))
    };

    if req.valid_code != valid_code {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid valid code"})));
    }

    let password = match create_random_password() {
        Some(password) => password,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Server can't create password"}))),
    };

    let write_result = write_user_data(&req.username, &password, &state);
    if !write_result {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "This account is exist"})));
    }

    let result = AddUserResp {
        username: req.username.to_string(),
        password,
    };

    return (StatusCode::OK, Json(json!(result)));
}

fn create_random_password() -> Option<String> {
    let mut rng = rand::thread_rng();
    let random_number: u32 = rng.gen_range(0..100_000_000);
    let password = format!("fate{:08}", random_number);
    let _pwd_hash = match pvpgn_hash_rs::get_hash_string(&password) {
        Ok(hash) => hash,
        Err(_) => return None,
    };

    return Some(password);
}

fn write_user_data(username: &str, pwd: &str, state: &AppState) -> bool {
    let folder_path = match state.config.get_string("USER_DATA_PATH") {
        Ok(path) => path,
        Err(_) => {
            println!("not found folder path in config");
            return false;
        }
    };

    let mut template_data: String = match fs::read_to_string("./template/user_template_data.dat") {
        Ok(data) => data,
        Err(_) => {
            println!("user not template data");
            return false;
        }
    };

    let uid_offset = state.config.get_int("UID_OFFSET").unwrap_or_else(|_| 0);

    let file_path = Path::new(&folder_path).join(username);

    let mut uid = 1 + uid_offset;

    let entries = match fs::read_dir(folder_path) {
        Ok(entries) => { entries }
        Err(_) => {
            println!("can't load users folder");
            return false;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => { entry }
            Err(_) => {
                println!("can't load users file");
                continue;
            }
        };

        let metadata = match entry.metadata() {
            Ok(metadata) => { metadata }
            Err(_) => {
                println!("can't get file metadata");
                continue;
            }
        };

        if metadata.is_file() {
            uid += 1;
            let file_name = entry.file_name();
            if file_name.to_string_lossy().to_lowercase() == username.to_lowercase() {
                println!("file is exist");
                return false;
            }
        }
    }

    let pwd_hash = match pvpgn_hash_rs::get_hash_string(pwd) {
        Ok(pwd) => pwd,
        Err(_) => {
            println!("can't create hash password");
            return false;
        }
    };

    template_data = template_data.replace("{{ userid }}", &uid.to_string());
    template_data = template_data.replace("{{ username }}", username);
    template_data = template_data.replace("{{ password }}", &pwd_hash);

    return match fs::write(file_path, template_data.as_bytes()) {
        Ok(_) => true,
        Err(_) => {
            println!("can't write file");
            false
        }
    };
}

struct AppState {
    config: Config,
}

#[derive(Deserialize, Debug)]
struct AddUserReq {
    username: String,
    valid_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AddUserResp {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RoomInfo {
    room_id: u16,
    room_name: String,
    player_count: u8,
}