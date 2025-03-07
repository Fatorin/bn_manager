use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use axum::extract::DefaultBodyLimit;
use axum::{Extension, Router};
use axum::routing::{get, post};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use crate::handler::map::*;
use crate::handler::room::room_info;
use crate::model::map::MapInfo;
use crate::settings::CONFIG;
use crate::util;

type Cache = Arc<Mutex<HashMap<String, MapInfo>>>;

pub fn routes() -> Router {
    let hash_map =
        util::file::read_files_in_directory(Path::new(&CONFIG.map_path)).expect("can't read folder");

    let cache: Cache = Arc::new(Mutex::new(hash_map));

    let cors = CorsLayer::permissive();

    let routes_apis = Router::new()
        .route("/room_info", get(room_info));

    let routes_maps = Router::new()
        .route("/get_maps", get(get_maps))
        .route("/upload_map", post(upload_map))
        .layer(Extension(cache))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(128 * 1024 * 1024));

    Router::new()
        .fallback_service(ServeDir::new("static").append_index_html_on_directories(true))
        .merge(routes_apis)
        .merge(routes_maps)
        .layer(cors)
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
}
