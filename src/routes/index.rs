use askama::Template;
use axum::extract::Query;
use axum::response::Html;
use serde::Deserialize;

use crate::config;
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
}

#[derive(Deserialize)]
pub struct IndexQuery {
    pub selected: Option<String>,
    pub flash: Option<String>,
    pub flash_error: Option<String>,
}

pub async fn index(Query(query): Query<IndexQuery>) -> Html<String> {
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
        (Some(msg), true)
    } else if let Some(msg) = query.flash {
        (Some(msg), false)
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
    };

    Html(tmpl.render().unwrap_or_else(|e| format!("Template error: {}", e)))
}
