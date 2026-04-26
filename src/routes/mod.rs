pub mod config;
pub mod index;
pub mod keys;
pub mod transfer;

use axum::Router;
use tower_http::services::ServeDir;

pub fn router() -> Router {
    Router::new()
        .route("/", axum::routing::get(index::index))
        .route("/generate", axum::routing::post(keys::generate))
        .route("/delete", axum::routing::post(keys::delete))
        .route("/export/{name}", axum::routing::get(transfer::export))
        .route("/import", axum::routing::post(transfer::import))
        // Config management
        .route("/config", axum::routing::get(config::config_page))
        .route("/config/edit", axum::routing::get(config::config_edit))
        .route("/config/save", axum::routing::post(config::config_save))
        .route("/config/add", axum::routing::post(config::config_add))
        .route("/config/delete", axum::routing::post(config::config_delete))
        .route("/config/raw", axum::routing::get(config::config_raw))
        .route("/config/raw/save", axum::routing::post(config::config_raw_save))
        // Backup & restore
        .route("/backup", axum::routing::get(config::backup))
        .route("/restore", axum::routing::post(config::restore))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
}
