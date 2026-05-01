use askama::Template;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::response::Html;
use serde::Deserialize;

use crate::config;
use crate::i18n::{Locale, LocaleMap};
use crate::ssh::{self, SshKeyInfo};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub keys: Vec<SshKeyInfo>,
    pub selected: Option<String>,
    pub selected_key: Option<SshKeyInfo>,
    pub host_groups: Vec<String>,
    pub flash: Option<String>,
    pub flash_is_error: bool,
    pub t: &'static LocaleMap,
}

#[derive(Deserialize)]
pub struct IndexQuery {
    pub selected: Option<String>,
    pub flash: Option<String>,
    pub flash_error: Option<String>,
    pub flash_param: Option<String>,
}

pub async fn index(Query(query): Query<IndexQuery>, headers: HeaderMap) -> Html<String> {
    let locale =
        Locale::from_accept_language(headers.get("accept-language").and_then(|v| v.to_str().ok()));

    let keys = ssh::list_keys();
    let selected_key = query
        .selected
        .as_ref()
        .and_then(|name| keys.iter().find(|k| &k.name == name).cloned());

    let host_groups = if let Some(ref key) = selected_key {
        config::find_hosts_using_key(&key.name)
    } else {
        Vec::new()
    };

    let (flash, flash_is_error) = if let Some(msg) = query.flash_error {
        (
            Some(locale.resolve_flash(&msg, query.flash_param.as_deref())),
            true,
        )
    } else if let Some(msg) = query.flash {
        (
            Some(locale.resolve_flash(&msg, query.flash_param.as_deref())),
            false,
        )
    } else {
        (None, false)
    };

    let tmpl = IndexTemplate {
        keys,
        selected: query.selected,
        selected_key,
        host_groups,
        flash,
        flash_is_error,
        t: locale.map,
    };

    Html(
        tmpl.render()
            .unwrap_or_else(|e| format!("Template error: {}", e)),
    )
}
