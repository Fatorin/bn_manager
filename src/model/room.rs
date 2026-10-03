use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomInfo {
    pub room_id: u16,
    pub room_name: String,
    pub player_count: u8,
}

/// `GET /status` body served by a ghostpp-rs instance.
#[derive(Deserialize, Debug)]
pub struct BotStatus {
    pub bot: String,
    #[serde(default)]
    pub games: Vec<BotGame>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotGame {
    pub name: String,
    #[serde(default)]
    pub map: String,
    /// `lobby` / `loading` / `playing`
    pub phase: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub started_at: Option<i64>,
    #[serde(default)]
    pub open_slots: u8,
    #[serde(default)]
    pub players: Vec<BotPlayer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BotPlayer {
    pub name: String,
    #[serde(default)]
    pub observer: bool,
    #[serde(default)]
    pub reconnecting: bool,
}

#[derive(Serialize, Debug)]
pub struct BotRooms {
    pub bot: String,
    pub online: bool,
    pub games: Vec<BotGame>,
}

#[derive(Serialize, Debug)]
pub struct RoomStatus {
    /// Unix seconds, so clients can show elapsed game time without trusting their own clock.
    pub server_time: i64,
    /// `None` when no `bot_status_urls` are configured.
    pub bots: Option<Vec<BotRooms>>,
    /// `None` when `bn_log_path` is blank.
    pub pvpgn: Option<Vec<RoomInfo>>,
    pub pvpgn_error: bool,
}
