use axum::body::Body;
use axum::extract::Multipart;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};

use crate::export;
use crate::ssh;

pub async fn export(axum::extract::Path(name): axum::extract::Path<String>) -> Response {
    let keys = ssh::list_keys();
    let key = match keys.iter().find(|k| k.name == name) {
        Some(k) => k.clone(),
        None => {
            return (StatusCode::NOT_FOUND, "key not found".to_string()).into_response();
        }
    };

    let tmp_dir = std::env::temp_dir();
    let file_name = format!("{}.tar.gz", key.name);
    let dest = tmp_dir.join(&file_name);

    if let Err(e) = export::export_key(&key, &dest) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
    }

    let data = match std::fs::read(&dest) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to read export: {}", e),
            )
                .into_response();
        }
    };

    let _ = std::fs::remove_file(&dest);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/gzip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file_name),
        )
        .body(Body::from(data))
        .unwrap()
}

pub async fn import(mut multipart: Multipart) -> Redirect {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name != "archive" {
            continue;
        }

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                return Redirect::to(&format!(
                    "/?flash_error=flash_upload_failed&flash_param={}",
                    e.to_string().replace(' ', "+")
                ));
            }
        };

        let tmp_dir = std::env::temp_dir();
        let tmp_path = tmp_dir.join("sshkeyman_import.tar.gz");
        if let Err(e) = std::fs::write(&tmp_path, &data) {
            return Redirect::to(&format!(
                "/?flash_error=flash_write_failed&flash_param={}",
                e.to_string().replace(' ', "+")
            ));
        }

        let result = export::import_key(&tmp_path);
        let _ = std::fs::remove_file(&tmp_path);

        return match result {
            Ok(name) => Redirect::to(&format!(
                "/?selected={}&flash=flash_imported_key&flash_param={}",
                name, name
            )),
            Err(e) => Redirect::to(&format!("/?flash_error={}", e.replace(' ', "+"))),
        };
    }

    Redirect::to("/?flash_error=flash_no_file_uploaded")
}
