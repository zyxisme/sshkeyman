use axum::extract::Form;
use axum::response::Redirect;
use serde::Deserialize;

use crate::ssh::{self, KeyType};

#[derive(Deserialize)]
pub struct GenerateForm {
    pub name: String,
    pub key_type: String,
    pub comment: String,
    pub passphrase: String,
}

pub async fn generate(Form(form): Form<GenerateForm>) -> Redirect {
    let key_type = match form.key_type.as_str() {
        "rsa" => KeyType::Rsa4096,
        _ => KeyType::Ed25519,
    };

    match ssh::generate_key(&form.name, key_type, &form.passphrase, &form.comment) {
        Ok(()) => Redirect::to(&format!("/?selected={}", form.name)),
        Err(e) => Redirect::to(&format!("/?flash_error={}", e.replace(' ', "+"))),
    }
}

#[derive(Deserialize)]
pub struct DeleteForm {
    pub name: String,
}

pub async fn delete(Form(form): Form<DeleteForm>) -> Redirect {
    let keys = ssh::list_keys();
    if let Some(key) = keys.iter().find(|k| k.name == form.name) {
        match ssh::delete_key(key) {
            Ok(()) => Redirect::to("/?flash=deleted"),
            Err(e) => Redirect::to(&format!("/?flash_error={}", e.replace(' ', "+"))),
        }
    } else {
        Redirect::to("/?flash_error=key+not+found")
    }
}
