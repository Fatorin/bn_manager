use chrono::NaiveDateTime;

#[derive(Debug, sqlx::FromRow)]
pub struct Game {
    pub id: i32,
    pub map: String,
    pub datetime: NaiveDateTime,
    pub duration: i32,
}
