use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::error;
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
    pub bn_server: String,
    pub bn_username: String,
    pub bn_password: String,
    pub mysql_user: String,
    pub mysql_password: String,
    pub mysql_host: String,
    pub mysql_port: u16,
    pub mysql_db_name: String,
}

impl Config {
    pub fn mysql_connection_string(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.mysql_user, self.mysql_password, self.mysql_host, self.mysql_port, self.mysql_db_name
        )
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_str = fs::read_to_string("settings.toml").expect("Failed to read config file");
    let config: Config = toml::from_str(&config_str).expect("Failed to parse config file");

    if !validate_config(&config) {
        error!("Invalid configuration");
        std::process::exit(1);
    }

    config
});

fn validate_config(config: &Config) -> bool {
    if !Path::new(&config.user_data_path).exists() {
        error!("USER_DATA_PATH is empty");
        return false;
    }
    if config.bn_log_path.is_empty() {
        error!("BN_LOG_PATH is empty");
        return false;
    }
    if config.valid_code.is_empty() {
        error!("VALID_CODE is empty");
        return false;
    }
    if !Path::new(&config.map_path).exists() {
        error!("MAP_PATH is empty");
        return false;
    }
    if config.map_valid_code.is_empty() {
        error!("MAP_VALID_CODE is empty");
        return false;
    }
    if config.db_path.is_empty() {
        error!("DB_PATH is empty");
        return false;
    }
    if config.discord_token.is_empty() {
        error!("DISCORD_TOKEN cannot be negative is empty");
        return false;
    }
    if config.discord_server_id <= 0 {
        error!("DISCORD_SERVER_ID cannot be negative is empty");
        return false;
    }
    if config.discord_report_channel_id <= 0 {
        error!("DISCORD_REPORT_CHANNEL_ID cannot be negative is empty");
        return false;
    }
    if config.uid_offset < 0 {
        error!("UID_OFFSET cannot be negative is empty");
        return false;
    }
    if config.bn_server.is_empty() {
        error!("BN_SERVER is empty");
        return false;
    }
    if config.bn_username.is_empty() {
        error!("BN_USERNAME is empty");
        return false;
    }
    if config.bn_password.is_empty() {
        error!("BN_PASSWORD is empty");
        return false;
    }
    if config.mysql_user.is_empty() {
        error!("MYSQL_USER is empty");
        return false;
    }
    if config.mysql_host.is_empty() {
        error!("MYSQL_HOST is empty");
        return false;
    }
    if config.mysql_port == 0 {
        error!("MYSQL_PORT is invalid");
        return false;
    }
    if config.mysql_db_name.is_empty() {
        error!("MYSQL_DB_NAME is empty");
        return false;
    }
    true
}

pub fn init_config() {
    Lazy::force(&CONFIG);
    error!("Configuration initialized successfully");
}
