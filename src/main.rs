use std::net::SocketAddr;
use tokio::sync::broadcast;
use tracing::{error, info, Level};
mod bot;
mod handler;
mod i18n;
mod model;
mod routes;
mod settings;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting application...");
    settings::init_config();

    let (shutdown_tx, _) = broadcast::channel(1);

    let mut bot_shutdown_rx = shutdown_tx.subscribe();

    info!("Starting Discord bot...");
    let bot_async_task = bot::start_discord_bot(&mut bot_shutdown_rx);

    let axum_shutdown_tx = shutdown_tx.clone();
    let app = routes::root::routes();
    info!("Starting Axum server on 0.0.0.0:3000...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let axum_server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        let _ = axum_shutdown_tx.subscribe().recv().await;
        info!("Axum server received shutdown signal");
    });
    let axum_async_task = async { axum_server.await };

    // Handle OS signals
    let signal_shutdown_tx = shutdown_tx.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C signal, initiating shutdown...");
                let _ = signal_shutdown_tx.send(());
            }
            Err(err) => {
                error!("Failed to listen for Ctrl+C: {}", err);
            }
        }
    });

    tokio::select! {
        res = bot_async_task => {
            match res {
                Ok(_) => info!("Discord Bot shutdown complete"),
                Err(err) => {
                    error!("Discord Bot error: {:?}", err);
                    // Log to file as well
                    std::fs::write("discord_bot_error.log", format!("{:?}", err))
                        .unwrap_or_else(|e| error!("Failed to write error log: {}", e));
                }
            }
        },
        res = axum_async_task => {
            match res {
                Ok(_) => info!("Axum Server shutdown complete"),
                Err(err) => {
                    error!("Axum Server error: {:?}", err);
                    // Log to file as well
                    std::fs::write("axum_server_error.log", format!("{:?}", err))
                        .unwrap_or_else(|e| error!("Failed to write error log: {}", e));
                }
            }
        },
    }



    info!("Sending final shutdown signal...");
    let _ = shutdown_tx.send(());
    info!("Application shutdown complete");

    Ok(())
}
