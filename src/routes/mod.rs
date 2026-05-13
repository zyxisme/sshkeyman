pub mod config;
pub mod index;
pub mod keys;
pub mod remote;
pub mod transfer;

use crate::assets::StaticAssets;
use axum::Router;
use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    match StaticAssets::get(&path) {
        Some(file) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], file.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/", axum::routing::get(index::index))
        .route("/generate", axum::routing::post(keys::generate))
        .route("/delete", axum::routing::post(keys::delete))
        .route("/export/{name}", axum::routing::get(transfer::export))
        .route("/import", axum::routing::post(transfer::import))
        // Remote key copy
        .route("/copy/{name}", axum::routing::get(remote::copy_page))
        .route("/copy", axum::routing::post(remote::copy_execute))
        // Config management
        .route("/config", axum::routing::get(config::config_page))
        .route("/config/edit", axum::routing::get(config::config_edit))
        .route("/config/save", axum::routing::post(config::config_save))
        .route("/config/add", axum::routing::post(config::config_add))
        .route("/config/delete", axum::routing::post(config::config_delete))
        .route("/config/raw", axum::routing::get(config::config_raw))
        .route(
            "/config/raw/save",
            axum::routing::post(config::config_raw_save),
        )
        // Backup & restore
        .route("/backup", axum::routing::get(config::backup))
        .route("/restore", axum::routing::post(config::restore))
        // Static files
        .route("/static/{*path}", axum::routing::get(static_handler))
}
