use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub user_data_path: String,
    pub bn_log_path: String,
    pub valid_code: String,
    pub map_path: String,
    pub map_valid_code: String,
    pub db_path: String,
    pub discord_token: String,
    pub discord_server_id: u64,
    pub discord_report_channel_id: u64,
    pub uid_offset: i32,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_str = fs::read_to_string("settings.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse config file");

    if !validate_config(&config) {
        eprintln!("Invalid configuration");
        std::process::exit(1);
    }

    config
});

fn validate_config(config: &Config) -> bool {
    if !Path::new(&config.user_data_path).exists() {
        println!("USER_DATA_PATH is empty");
        return false;
    }
    if config.bn_log_path.is_empty() {
        println!("BN_LOG_PATH is empty");
        return false;
    }
    if config.valid_code.is_empty() {
        println!("VALID_CODE is empty");
        return false;
    }
    if !Path::new(&config.map_path).exists() {
        println!("MAP_PATH is empty");
        return false;
    }
    if config.map_valid_code.is_empty() {
        println!("MAP_VALID_CODE is empty");
        return false;
    }
    if !Path::new(&config.db_path).exists() {
        println!("DB_PATH is empty");
        return false;
    }
    if config.discord_token.is_empty() {
        println!("DISCORD_TOKEN cannot be negative is empty");
        return false;
    }
    if config.discord_server_id <= 0 {
        println!("DISCORD_SERVER_ID cannot be negative is empty");
        return false;
    }
    if config.discord_report_channel_id <= 0 {
        println!("DISCORD_REPORT_CHANNEL_ID cannot be negative is empty");
        return false;
    }
    if config.uid_offset < 0 {
        println!("UID_OFFSET cannot be negative is empty");
        return false;
    }
    true
}

pub fn init_config() {
    Lazy::force(&CONFIG);
    println!("Configuration initialized successfully");
}
