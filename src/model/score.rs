use serde::Serialize;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Score {
    pub id: i32,
    pub name: String,
    pub score: f64,
}
