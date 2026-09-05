use crate::model::user::User;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

fn map_user(row: &SqliteRow) -> User {
    User {
        id: row.get("id"),
        discord_id: row.get("discord_id"),
        username: row.get("username"),
        created_at: row.get("created_at"),
    }
}

pub async fn get_user_by_discord_id(
    pool: &SqlitePool,
    discord_id: &str,
) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM users WHERE discord_id = ?")
        .bind(discord_id)
        .fetch_optional(pool)
        .await?;

    Ok(row.as_ref().map(map_user))
}

pub async fn get_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    // NOCASE matches how util::file::check_exist compares account file names, so a
    // lookup does not miss an account purely because of capitalisation.
    let row = sqlx::query("SELECT * FROM users WHERE username = ? COLLATE NOCASE")
        .bind(username)
        .fetch_optional(pool)
        .await?;

    Ok(row.as_ref().map(map_user))
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

    Ok(map_user(&row))
}

pub async fn create_admin_created_account(
    pool: &SqlitePool,
    username: &str,
    admin_discord_id: &str,
    admin_username: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO admin_created_accounts (username, admin_discord_id, admin_username) VALUES (?, ?, ?)",
    )
    .bind(username)
    .bind(admin_discord_id)
    .bind(admin_username)
    .execute(pool)
    .await?;

    Ok(())
}
