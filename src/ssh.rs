use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use ssh_key::{HashAlg, PublicKey};

#[derive(Debug, Clone)]
pub struct SshKeyInfo {
    pub name: String,
    pub private_path: PathBuf,
    pub public_path: PathBuf,
    pub key_type: String,
    pub fingerprint: String,
    pub public_key_content: String,
    pub comment: String,
    pub has_private: bool,
}

pub fn ssh_dir() -> PathBuf {
    let dir = dirs::home_dir()
        .expect("cannot determine home directory")
        .join(".ssh");
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("failed to create ~/.ssh");
    }
    dir
}

pub fn list_keys() -> Vec<SshKeyInfo> {
    let dir = ssh_dir();
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut keys = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Only process .pub files
        if !name.ends_with(".pub") {
            continue;
        }

        let stem = name.trim_end_matches(".pub").to_string();
        let private_path = dir.join(&stem);

        let pub_content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let pubkey = match PublicKey::from_openssh(&pub_content) {
            Ok(k) => k,
            Err(_) => continue,
        };

        let key_type = format!("{:?}", pubkey.algorithm());
        let fingerprint = pubkey.fingerprint(HashAlg::Sha256).to_string();
        let comment = pubkey.comment().to_string();
        let has_private = private_path.exists();

        keys.push(SshKeyInfo {
            name: stem,
            private_path,
            public_path: path,
            key_type,
            fingerprint,
            public_key_content: pub_content.trim_end().to_string(),
            comment,
            has_private,
        });
    }

    keys.sort_by(|a, b| a.name.cmp(&b.name));
    keys
}

#[derive(PartialEq, Clone, Copy)]
pub enum KeyType {
    Ed25519,
    Rsa4096,
}

impl KeyType {
    pub fn as_str(&self) -> &str {
        match self {
            KeyType::Ed25519 => "ed25519",
            KeyType::Rsa4096 => "rsa",
        }
    }

    pub fn as_arg_bits(&self) -> Option<&str> {
        match self {
            KeyType::Ed25519 => None,
            KeyType::Rsa4096 => Some("4096"),
        }
    }
}

pub fn generate_key(
    name: &str,
    key_type: KeyType,
    passphrase: &str,
    comment: &str,
) -> Result<(), String> {
    let path = ssh_dir().join(name);

    if path.exists() || path.with_extension("pub").exists() {
        return Err(format!("key '{}' already exists", name));
    }

    let mut args = vec![
        "-t".to_string(),
        key_type.as_str().to_string(),
        "-f".to_string(),
        path.to_string_lossy().to_string(),
        "-C".to_string(),
        comment.to_string(),
        "-N".to_string(),
        passphrase.to_string(),
    ];

    if let Some(bits) = key_type.as_arg_bits() {
        args.push("-b".to_string());
        args.push(bits.to_string());
    }

    let status = Command::new("ssh-keygen")
        .args(&args)
        .status()
        .map_err(|e| format!("failed to run ssh-keygen: {}", e))?;

    if !status.success() {
        return Err("ssh-keygen failed".to_string());
    }

    // Set permissions
    set_key_permissions(&path);
    Ok(())
}

pub fn delete_key(key: &SshKeyInfo) -> Result<(), String> {
    if key.public_path.exists() {
        fs::remove_file(&key.public_path)
            .map_err(|e| format!("failed to delete public key: {}", e))?;
    }
    if key.private_path.exists() {
        fs::remove_file(&key.private_path)
            .map_err(|e| format!("failed to delete private key: {}", e))?;
    }
    Ok(())
}

pub fn set_key_permissions(private_path: &Path) {
    if private_path.exists() {
        let _ = fs::set_permissions(private_path, Permissions::from_mode(0o600));
    }
    let pub_path = private_path.with_extension("pub");
    if pub_path.exists() {
        let _ = fs::set_permissions(pub_path, Permissions::from_mode(0o644));
    }
}
