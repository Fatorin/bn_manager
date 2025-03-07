use crate::settings::CONFIG;
use serenity::all::GatewayIntents;
use serenity::Client;
use sqlx::migrate::Migrator;
use std::path::Path;

pub struct Bot {
    pub database: sqlx::SqlitePool,
}

pub async fn start_discord_bot() {
    let token = &CONFIG.discord_token;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("./app/db/database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");

    let migrations = Migrator::new(Path::new("./migrations")).await.unwrap();
    migrations
        .run(&database)
        .await
        .expect("Couldn't run database migrations");

    let bot = Bot { database };

    let mut client = Client::builder(&token, intents)
        .event_handler(bot)
        .await
        .expect("Error creating client");

    println!("Discord Bot starting...");

    if let Err(why) = client.start().await {
        eprintln!("Discord Bot start failed, error: {:?}", why);
    }
}
