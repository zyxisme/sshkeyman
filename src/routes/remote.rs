use askama::Template;
use axum::extract::{Form, Query};
use axum::http::HeaderMap;
use axum::response::{Html, Redirect};
use serde::Deserialize;

use crate::i18n::{Locale, LocaleMap};
use crate::remote;

#[derive(Template)]
#[template(path = "remote_copy.html")]
pub struct RemoteCopyTemplate {
    pub key_name: String,
    pub flash: Option<String>,
    pub flash_is_error: bool,
    pub t: &'static LocaleMap,
}

#[derive(Deserialize)]
pub struct CopyQuery {
    pub flash_error: Option<String>,
    pub flash_param: Option<String>,
}

pub async fn copy_page(
    axum::extract::Path(name): axum::extract::Path<String>,
    Query(query): Query<CopyQuery>,
    headers: HeaderMap,
) -> Html<String> {
    let locale =
        Locale::from_accept_language(headers.get("accept-language").and_then(|v| v.to_str().ok()));

    let flash_is_error = query.flash_error.is_some();
    let flash = query
        .flash_error
        .map(|msg| locale.resolve_flash(&msg, query.flash_param.as_deref()));

    let tmpl = RemoteCopyTemplate {
        key_name: name,
        flash,
        flash_is_error,
        t: locale.map,
    };

    Html(
        tmpl.render()
            .unwrap_or_else(|e| format!("Template error: {}", e)),
    )
}

#[derive(Deserialize)]
pub struct CopyForm {
    pub key_name: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub auth_method: String,
    pub password: String,
    pub key_path: String,
    pub passphrase: String,
}

pub async fn copy_execute(Form(form): Form<CopyForm>) -> Redirect {
    let port: u16 = form.port.parse().unwrap_or(22);

    let password = if form.password.is_empty() {
        None
    } else {
        Some(form.password.as_str())
    };
    let key_path = if form.key_path.is_empty() {
        None
    } else {
        Some(form.key_path.as_str())
    };
    let passphrase = if form.passphrase.is_empty() {
        None
    } else {
        Some(form.passphrase.as_str())
    };

    // Client-side validation fallback — show errors inline
    if form.host.trim().is_empty() {
        return Redirect::to(&format!(
            "/copy/{}?flash_error=flash_validation_host_required",
            form.key_name
        ));
    }
    if form.username.trim().is_empty() {
        return Redirect::to(&format!(
            "/copy/{}?flash_error=flash_validation_username_required",
            form.key_name
        ));
    }
    if form.auth_method == "password" && form.password.is_empty() {
        return Redirect::to(&format!(
            "/copy/{}?flash_error=flash_validation_password_required",
            form.key_name
        ));
    }
    if form.auth_method == "key" && form.key_path.is_empty() {
        return Redirect::to(&format!(
            "/copy/{}?flash_error=flash_validation_key_path_required",
            form.key_name
        ));
    }

    match remote::copy_key_to_remote(&remote::RemoteCopyConfig {
        key_name: &form.key_name,
        host: form.host.trim(),
        port,
        username: form.username.trim(),
        auth_method: &form.auth_method,
        password,
        key_path,
        passphrase,
    }) {
        Ok(()) => Redirect::to(&format!(
            "/?selected={}&flash=flash_copy_success&flash_param={}",
            form.key_name, form.host
        )),
        Err(e) => Redirect::to(&format!(
            "/copy/{}?flash_error={}",
            form.key_name,
            e.replace(' ', "+")
        )),
    }
}
