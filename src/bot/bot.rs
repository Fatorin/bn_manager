use crate::settings::CONFIG;
use crate::telnet;
use serenity::all::GatewayIntents;
use serenity::Client;
use std::time::Duration;
use tokio::sync::broadcast::Receiver;
use tokio::time::timeout;

pub struct Bot {
    pub database: sqlx::SqlitePool,
    pub telnet: telnet::ApiClient,
}

pub async fn start_discord_bot(
    shutdown: &mut Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = &CONFIG.discord_token;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    println!(
        "username:{}, password:{}",
        CONFIG.bn_username, CONFIG.bn_password
    );

    let telnet_client = timeout(
        Duration::from_secs(30),
        telnet::ApiClient::start(&CONFIG.bn_server, &CONFIG.bn_username, &CONFIG.bn_password),
    )
    .await
    .map_err(|_| "Timeout while starting telnet client")?
    .map_err(|e| format!("Couldn't connect to BN server {:?}", e))?;

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&CONFIG.db_path)
                .create_if_missing(true),
        )
        .await
        .map_err(|e| format!("Couldn't connect to database: {}", e))?;

    // Migrations are embedded at compile time, so the running binary no longer
    // needs a ./migrations directory beside it. build.rs makes cargo rebuild
    // whenever a migration file is added or changed.
    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .map_err(|e| format!("Couldn't run database migrations: {}", e))?;

    let bot = Bot {
        database,
        telnet: telnet_client,
    };

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
