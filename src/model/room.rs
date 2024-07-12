use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RoomInfo {
    pub room_id: u16,
    pub room_name: String,
    pub player_count: u8,
}