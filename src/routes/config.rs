use askama::Template;
use axum::body::Body;
use axum::extract::{Multipart, Query};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;

use crate::config::{self, SshConfigEntry};
use crate::export;

// ─── Config list page ───

#[derive(Template)]
#[template(path = "config.html")]
pub struct ConfigTemplate {
    pub entries: Vec<ConfigEntryView>,
    pub flash: Option<String>,
    pub flash_is_error: bool,
}

pub struct ConfigEntryView {
    pub host_pattern: String,
    pub fields: Vec<(String, String)>,
    pub identity_file: Option<String>,
}

#[derive(Deserialize)]
pub struct ConfigQuery {
    pub flash: Option<String>,
    pub flash_error: Option<String>,
}

pub async fn config_page(Query(query): Query<ConfigQuery>) -> Html<String> {
    let entries = config::parse_config();
    let views: Vec<ConfigEntryView> = entries
        .iter()
        .map(|e| {
            let identity_file = e
                .fields
                .iter()
                .find(|(k, _)| k == "IdentityFile")
                .map(|(_, v)| v.clone());
            ConfigEntryView {
                host_pattern: e.host_pattern.clone(),
                fields: config::get_entry_fields(e),
                identity_file,
            }
        })
        .collect();

    let (flash, flash_is_error) = if let Some(msg) = query.flash_error {
        (Some(msg), true)
    } else if let Some(msg) = query.flash {
        (Some(msg), false)
    } else {
        (None, false)
    };

    let tmpl = ConfigTemplate {
        entries: views,
        flash,
        flash_is_error,
    };
    Html(tmpl.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

// ─── Edit page ───

#[derive(Template)]
#[template(path = "config_edit.html")]
pub struct ConfigEditTemplate {
    pub host_pattern: String,
    pub fields_json: String,       // JSON array of {key, value}
    pub available_keys_json: String, // JSON array of key names
    pub is_new: bool,
}

#[derive(Deserialize)]
pub struct EditQuery {
    pub host: Option<String>,
}

const DEFAULT_FIELDS: &[&str] = &[
    "HostName",
    "User",
    "Port",
    "IdentityFile",
];

/// Merge existing fields with default fields, preserving existing values.
/// Existing fields keep their order; missing defaults are appended.
fn merge_with_defaults(fields: &[(String, String)]) -> Vec<(String, String)> {
    let mut result: Vec<(String, String)> = fields.to_vec();
    let existing_keys: std::collections::HashSet<String> =
        fields.iter().map(|(k, _)| k.to_lowercase()).collect();

    for &default_key in DEFAULT_FIELDS {
        if !existing_keys.contains(&default_key.to_lowercase()) {
            result.push((default_key.to_string(), String::new()));
        }
    }

    result
}

fn available_keys_json() -> String {
    let keys = crate::ssh::list_keys();
    let names: Vec<&str> = keys.iter().map(|k| k.name.as_str()).collect();
    serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string())
}

pub async fn config_edit(Query(query): Query<EditQuery>) -> Html<String> {
    let entries = config::parse_config();
    let akj = available_keys_json();

    if let Some(ref host) = query.host
        && let Some(entry) = entries.iter().find(|e| &e.host_pattern == host)
    {
        let merged = merge_with_defaults(&entry.fields);
        let fields_json = serde_json::to_string(&merged).unwrap_or_else(|_| "[]".to_string());
        let tmpl = ConfigEditTemplate {
            host_pattern: entry.host_pattern.clone(),
            fields_json,
            available_keys_json: akj,
            is_new: false,
        };
        return Html(tmpl.render().unwrap_or_else(|e| format!("Template error: {}", e)));
    }

    // New entry — all defaults with empty values
    let defaults: Vec<(String, String)> = DEFAULT_FIELDS
        .iter()
        .map(|&k| (k.to_string(), String::new()))
        .collect();
    let fields_json = serde_json::to_string(&defaults).unwrap_or_else(|_| "[]".to_string());

    let tmpl = ConfigEditTemplate {
        host_pattern: String::new(),
        fields_json,
        available_keys_json: akj,
        is_new: true,
    };
    Html(tmpl.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

// ─── Save edit ───

/// Extract all values for a repeated form field.
/// Works whether the field appears once (string) or multiple times (array).
fn form_values(form: &axum::extract::RawForm, key: &str) -> Vec<String> {
    let raw = String::from_utf8_lossy(&form.0);
    let mut values = Vec::new();
    for pair in raw.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            let k = urldecode(k);
            let v = urldecode(v);
            if k == key {
                values.push(v);
            }
        }
    }
    values
}

fn form_single(form: &axum::extract::RawForm, key: &str) -> String {
    form_values(form, key)
        .into_iter()
        .next()
        .unwrap_or_default()
}

fn urldecode(s: &str) -> String {
    let s = s.replace('+', " ");
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub async fn config_save(axum::extract::RawForm(form): axum::extract::RawForm) -> Redirect {
    let mut entries = config::parse_config();

    let original_host = form_single(&axum::extract::RawForm(form.clone()), "original_host");
    let host_pattern = form_single(&axum::extract::RawForm(form.clone()), "host_pattern");

    let keys = form_values(&axum::extract::RawForm(form.clone()), "field_keys");
    let vals = form_values(&axum::extract::RawForm(form.clone()), "field_values");

    let fields: Vec<(String, String)> = keys
        .iter()
        .zip(vals.iter())
        .filter(|(k, _)| !k.trim().is_empty())
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .filter(|(_, v)| !v.is_empty())
        .collect();

    if original_host.is_empty() {
        entries.push(SshConfigEntry {
            host_pattern: host_pattern.clone(),
            fields,
        });
    } else if let Some(entry) = entries.iter_mut().find(|e| e.host_pattern == original_host)
    {
        entry.host_pattern = host_pattern.clone();
        entry.fields = fields;
    } else {
        return Redirect::to("/config?flash_error=host+not+found");
    }

    match config::write_config(&entries) {
        Ok(()) => Redirect::to(&format!("/config?flash=saved+'{}'", host_pattern)),
        Err(e) => Redirect::to(&format!(
            "/config?flash_error={}",
            e.replace(' ', "+")
        )),
    }
}

// ─── Add new ───

pub async fn config_add() -> Redirect {
    Redirect::to("/config/edit")
}

// ─── Delete ───

#[derive(Deserialize)]
pub struct DeleteForm {
    pub host: String,
}

pub async fn config_delete(axum::Form(form): axum::Form<DeleteForm>) -> Redirect {
    let mut entries = config::parse_config();
    let before = entries.len();
    entries.retain(|e| e.host_pattern != form.host);

    if entries.len() == before {
        return Redirect::to("/config?flash_error=host+not+found");
    }

    match config::write_config(&entries) {
        Ok(()) => Redirect::to(&format!("/config?flash=deleted+'{}'", form.host)),
        Err(e) => Redirect::to(&format!(
            "/config?flash_error={}",
            e.replace(' ', "+")
        )),
    }
}

// ─── Raw edit ───

#[derive(Template)]
#[template(path = "config_raw.html")]
pub struct ConfigRawTemplate {
    pub content: String,
    pub flash: Option<String>,
    pub flash_is_error: bool,
}

pub async fn config_raw(Query(query): Query<ConfigQuery>) -> Html<String> {
    let content = config::read_raw_config();

    let (flash, flash_is_error) = if let Some(msg) = query.flash_error {
        (Some(msg), true)
    } else if let Some(msg) = query.flash {
        (Some(msg), false)
    } else {
        (None, false)
    };

    let tmpl = ConfigRawTemplate {
        content,
        flash,
        flash_is_error,
    };
    Html(tmpl.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}

#[derive(Deserialize)]
pub struct RawForm {
    pub content: String,
}

pub async fn config_raw_save(axum::Form(form): axum::Form<RawForm>) -> Redirect {
    match config::write_raw_config(&form.content) {
        Ok(()) => Redirect::to("/config/raw?flash=saved"),
        Err(e) => Redirect::to(&format!(
            "/config/raw?flash_error={}",
            e.replace(' ', "+")
        )),
    }
}

// ─── Backup ───

pub async fn backup() -> Response {
    let tmp_dir = std::env::temp_dir();
    let dest = tmp_dir.join("ssh_backup.tar.gz");

    if let Err(e) = export::backup_all(&dest) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
    }

    let data = match std::fs::read(&dest) {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to read backup: {}", e),
            )
                .into_response();
        }
    };

    let _ = std::fs::remove_file(&dest);

    let now = chrono_like_timestamp();
    let filename = format!("ssh_backup_{}.tar.gz", now);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/gzip")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .unwrap()
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

// ─── Restore ───

pub async fn restore(mut multipart: Multipart) -> Redirect {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name != "archive" {
            continue;
        }

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => {
                return Redirect::to(&format!(
                    "/?flash_error=upload+failed:+{}",
                    e.to_string().replace(' ', "+")
                ));
            }
        };

        let tmp_dir = std::env::temp_dir();
        let tmp_path = tmp_dir.join("sshkeyman_restore.tar.gz");
        if let Err(e) = std::fs::write(&tmp_path, &data) {
            return Redirect::to(&format!(
                "/?flash_error=write+failed:+{}",
                e.to_string().replace(' ', "+")
            ));
        }

        let result = export::restore_all(&tmp_path);
        let _ = std::fs::remove_file(&tmp_path);

        return match result {
            Ok(files) => Redirect::to(&format!(
                "/?flash=restored+{}+files",
                files.len()
            )),
            Err(e) => Redirect::to(&format!(
                "/?flash_error={}",
                e.replace(' ', "+")
            )),
        };
    }

    Redirect::to("/?flash_error=no+file+uploaded")
}
