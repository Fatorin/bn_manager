use std::fs;
use serde::Deserialize;
use once_cell::sync::Lazy;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub user_data_path: String,
    pub certs_path: String,
    pub bn_log_path: String,
    pub valid_code: String,
    pub map_path: String,
    pub map_valid_code: String,
    pub runtime_mode: String,
    pub uid_offset: i32,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_str = fs::read_to_string("settings.toml")
        .expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str)
        .expect("Failed to parse config file");

    if !validate_config(&config) {
        eprintln!("Invalid configuration");
        std::process::exit(1);
    }

    config
});

fn validate_config(config: &Config) -> bool {
    if config.user_data_path.is_empty() {
        println!("USER_DATA_PATH is empty");
        return false;
    }
    if config.certs_path.is_empty() {
        println!("CERTS_PATH is empty");
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
    if config.map_path.is_empty() {
        println!("MAP_PATH is empty");
        return false;
    }
    if config.map_valid_code.is_empty() {
        println!("MAP_VALID_CODE is empty");
        return false;
    }
    if config.runtime_mode.is_empty() {
        println!("RUNTIME_MODE is empty");
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