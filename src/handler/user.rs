use std::fs;
use std::path::Path;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use rand::Rng;
use serde_json::json;
use crate::util;
use crate::model::user::{AddUserReq, AddUserResp};
use crate::settings::CONFIG;

pub async fn add_user(req: Json<AddUserReq>) -> impl IntoResponse {
    if req.valid_code != CONFIG.valid_code {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid valid code"})));
    }

    let password = match create_random_password() {
        Some(password) => password,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Server can't create password"}))),
    };

    let write_result = write_user_data(&req.username, &password);
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

fn write_user_data(username: &str, pwd: &str) -> bool {
    let mut template_data: String = match fs::read_to_string("./template/user_template_data.dat") {
        Ok(data) => data,
        Err(_) => {
            println!("user not template data");
            return false;
        }
    };

    let uid_offset = CONFIG.uid_offset;

    let file_path = Path::new(&CONFIG.user_data_path).join(username);

    let mut uid: usize = 1 + uid_offset as usize;

    let exist_result = util::file::check_exist(&CONFIG.user_data_path, username);
    if !exist_result.1 {
        return false;
    }
    uid += exist_result.0;

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