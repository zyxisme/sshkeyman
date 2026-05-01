use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs::File;
use std::path::Path;

use crate::ssh::{self, SshKeyInfo};

pub fn export_key(key: &SshKeyInfo, dest_path: &Path) -> Result<(), String> {
    let tar_gz = File::create(dest_path).map_err(|e| format!("failed to create file: {}", e))?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    // Add private key
    if key.private_path.exists() {
        tar.append_path_with_name(&key.private_path, &key.name)
            .map_err(|e| format!("failed to add private key: {}", e))?;
    }

    // Add public key
    if key.public_path.exists() {
        let pub_name = format!("{}.pub", key.name);
        tar.append_path_with_name(&key.public_path, pub_name)
            .map_err(|e| format!("failed to add public key: {}", e))?;
    }

    tar.finish()
        .map_err(|e| format!("failed to finalize archive: {}", e))?;
    Ok(())
}

pub fn import_key(archive_path: &Path) -> Result<String, String> {
    let file = File::open(archive_path).map_err(|e| format!("failed to open archive: {}", e))?;
    let dec = GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);

    let ssh_dir = ssh::ssh_dir();
    let mut imported_name = String::new();

    let entries: Vec<_> = archive
        .entries()
        .map_err(|e| format!("failed to read archive: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("failed to read entry: {}", e))?;

    // Safety check: refuse if target files already exist
    for entry in &entries {
        let entry_path = entry.path().map_err(|e| format!("bad entry path: {}", e))?;
        let file_name = entry_path
            .file_name()
            .ok_or("entry has no filename")?
            .to_string_lossy()
            .to_string();
        let target = ssh_dir.join(&file_name);
        if target.exists() {
            return Err(format!("{} already exists in ~/.ssh/, aborting", file_name));
        }
        if imported_name.is_empty() || !file_name.ends_with(".pub") {
            imported_name = file_name.trim_end_matches(".pub").to_string();
        }
    }

    // Extract
    let file = File::open(archive_path).map_err(|e| format!("failed to reopen archive: {}", e))?;
    let dec = GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);

    archive
        .unpack(&ssh_dir)
        .map_err(|e| format!("failed to extract: {}", e))?;

    // Set permissions
    let private_path = ssh_dir.join(&imported_name);
    ssh::set_key_permissions(&private_path);

    Ok(imported_name)
}

/// Backup all key pairs and config from ~/.ssh into a single tar.gz
pub fn backup_all(dest_path: &Path) -> Result<(), String> {
    let ssh_dir = ssh::ssh_dir();

    let tar_gz = File::create(dest_path).map_err(|e| format!("failed to create file: {}", e))?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);

    let entries =
        std::fs::read_dir(&ssh_dir).map_err(|e| format!("failed to read ~/.ssh: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip known_hosts, known_hosts.old, authorized_keys, etc.
        // Include: config, *.pub, and private key files (files without extension that have a .pub counterpart)
        let is_pub = file_name.ends_with(".pub");
        let is_config = file_name == "config";
        let is_private_key =
            !file_name.contains('.') && ssh_dir.join(format!("{}.pub", file_name)).exists();

        if is_pub || is_config || is_private_key {
            tar.append_path_with_name(&path, &file_name)
                .map_err(|e| format!("failed to add {}: {}", file_name, e))?;
        }
    }

    tar.finish()
        .map_err(|e| format!("failed to finalize archive: {}", e))?;
    Ok(())
}

/// Restore all files from a backup tar.gz into ~/.ssh
/// Skips files that already exist
pub fn restore_all(archive_path: &Path) -> Result<Vec<String>, String> {
    let file = File::open(archive_path).map_err(|e| format!("failed to open archive: {}", e))?;
    let dec = GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);

    let ssh_dir = ssh::ssh_dir();
    let mut restored = Vec::new();

    let entries: Vec<_> = archive
        .entries()
        .map_err(|e| format!("failed to read archive: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("failed to read entry: {}", e))?;

    // First pass: safety check
    for entry in &entries {
        let entry_path = entry.path().map_err(|e| format!("bad entry path: {}", e))?;
        let file_name = entry_path
            .file_name()
            .ok_or("entry has no filename")?
            .to_string_lossy()
            .to_string();
        let target = ssh_dir.join(&file_name);
        if target.exists() {
            return Err(format!(
                "{} already exists in ~/.ssh/, aborting restore",
                file_name
            ));
        }
    }

    // Second pass: extract
    let file = File::open(archive_path).map_err(|e| format!("failed to reopen archive: {}", e))?;
    let dec = GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);

    archive
        .unpack(&ssh_dir)
        .map_err(|e| format!("failed to extract: {}", e))?;

    // Set permissions on private keys and collect restored names
    for entry in &entries {
        let entry_path = entry.path().map_err(|e| format!("bad entry path: {}", e))?;
        let file_name = entry_path
            .file_name()
            .ok_or("entry has no filename")?
            .to_string_lossy()
            .to_string();

        restored.push(file_name.clone());

        // If it's a private key (no .pub extension and a .pub exists), set permissions
        if !file_name.ends_with(".pub") && !file_name.contains('.') {
            ssh::set_key_permissions(&ssh_dir.join(&file_name));
        }
    }

    Ok(restored)
}
