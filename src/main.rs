use std::net::SocketAddr;

mod bot;
mod handler;
mod i18n;
mod model;
mod routes;
mod settings;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    settings::init_config();

    bot::start_discord_bot().await?;

    let app = routes::root::routes();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
