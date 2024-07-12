use std::net::SocketAddr;
use std::path::Path;

use axum_server::tls_rustls::RustlsConfig;

use crate::settings::CONFIG;

mod model;
mod handler;
mod routes;
mod util;
mod settings;

#[tokio::main]
async fn main() {
    settings::init_config();

    let runtime_mode = CONFIG.runtime_mode.as_str();
    let certs_path = CONFIG.certs_path.as_str();

    let app = routes::root::routes();

    match runtime_mode {
        "dev" => {
            let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
            axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
        }
        "prod" => {
            let tls_config = RustlsConfig::from_pem_file(
                Path::new(&certs_path).join("cert.pem"),
                Path::new(&certs_path).join("key.pem"),
            ).await.unwrap();

            let addr = SocketAddr::from(([0, 0, 0, 0], 443));
            axum_server::bind_rustls(addr, tls_config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        _ => {
            panic!("not supported runtime mode")
        }
    }
}