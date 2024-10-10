use std::net::SocketAddr;

mod model;
mod handler;
mod routes;
mod util;
mod settings;

#[tokio::main]
async fn main() {
    settings::init_config();

    let app = routes::root::routes();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}