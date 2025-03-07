use crate::settings::CONFIG;
use serenity::all::GatewayIntents;
use serenity::Client;
use sqlx::migrate::Migrator;
use std::path::Path;
use tokio::sync::broadcast::Receiver;

pub struct Bot {
    pub database: sqlx::SqlitePool,
}

pub async fn start_discord_bot(
    shutdown: &mut Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
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
        .map_err(|e| format!("Couldn't run database migrations: {}", e))?;

    let bot = Bot { database };

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .map_err(|e| format!("Error creating client: {}", e))?;

    println!("Discord Bot starting...");

    tokio::select! {
        res = client.start() => {
            if let Err(why) = res {
                eprintln!("Discord Bot starting failed, ex:{:?}", why);
            }
        },
        _ = shutdown.recv() => {
            println!("Shutting down Discord Bot...");
            client.shard_manager.shutdown_all().await;
        }
    }

    Ok(())
}
