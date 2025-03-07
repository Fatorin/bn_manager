use crate::model::user::User;
use sqlx::{Row, SqlitePool};

pub async fn get_user_by_discord_id(
    pool: &SqlitePool,
    discord_id: &str,
) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM users WHERE discord_id = ?")
        .bind(discord_id)
        .fetch_optional(pool)
        .await?;

    let user = row.map(|row| User {
        id: row.get("id"),
        discord_id: row.get("discord_id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
    });

    Ok(user)
}

pub async fn create_user(
    pool: &SqlitePool,
    discord_id: &str,
    username: &str,
) -> Result<User, sqlx::Error> {
    let row = sqlx::query("INSERT INTO users (discord_id, username) VALUES (?, ?) RETURNING *")
        .bind(discord_id)
        .bind(username)
        .fetch_one(pool)
        .await?;

    let user = User {
        id: row.get("id"),
        discord_id: row.get("discord_id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
    };

    Ok(user)
}
