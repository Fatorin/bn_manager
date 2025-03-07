use crate::settings::CONFIG;
use serenity::all::GatewayIntents;
use serenity::Client;
use sqlx::migrate::Migrator;
use std::path::Path;

pub struct Bot {
    pub database: sqlx::SqlitePool,
}

pub async fn start_discord_bot() -> Result<(), Box<dyn std::error::Error>> {
    let token = &CONFIG.discord_token;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&CONFIG.db_path)
                .create_if_missing(true),
        )
        .await
        .map_err(|e| format!("Couldn't connect to database: {}", e))?;

    let migrations = Migrator::new(Path::new("./migrations"))
        .await
        .map_err(|e| format!("Couldn't load migrations: {}", e))?;

    migrations
        .run(&database)
        .await
        .map_err(|e| format!("Couldn't run database migrations :{}", e))?;

    let bot = Bot { database };

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .map_err(|e| format!("Error creating client: {}", e))?;

    println!("Discord Bot starting...");

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            eprintln!("iscord Bot start failed, error: {:?}", why);
        }
    });

    Ok(())
}
