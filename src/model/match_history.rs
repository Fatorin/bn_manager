use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MatchHistory {
    pub id: i32,
    pub map: String,
    pub datetime: NaiveDateTime,
    pub duration: i32,
    pub teams: Vec<Team>,
}

#[derive(Debug, Serialize)]
pub struct Team {
    pub index: i32,
    pub name: String,
    pub score: i32,
    pub servants: Vec<Servant>,
}

#[derive(Debug, Serialize)]
pub struct Servant {
    #[serde(rename = "UserName")]
    pub user_name: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Level")]
    pub level: i32,
    #[serde(rename = "Kills")]
    pub kills: i32,
    #[serde(rename = "Deaths")]
    pub deaths: i32,
    #[serde(rename = "Assists")]
    pub assists: i32,
}
