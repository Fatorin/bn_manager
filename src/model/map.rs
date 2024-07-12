use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MapInfo {
    pub name: String,
    pub map_name: String,
}