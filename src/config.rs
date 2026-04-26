use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SshConfigEntry {
    pub host_pattern: String,
    pub fields: Vec<(String, String)>,
}

pub fn config_path() -> PathBuf {
    crate::ssh::ssh_dir().join("config")
}

pub fn parse_config() -> Vec<SshConfigEntry> {
    let path = config_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut entries = Vec::new();
    let mut current: Option<SshConfigEntry> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // New Host block
        if trimmed.to_lowercase().starts_with("host ") {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            let pattern = trimmed[5..].trim().to_string();
            current = Some(SshConfigEntry {
                host_pattern: pattern,
                fields: Vec::new(),
            });
            continue;
        }

        // Indented field within a Host block
        if (line.starts_with(' ') || line.starts_with('\t'))
            && let Some(ref mut entry) = current
            && let Some((key, value)) = parse_field(trimmed)
        {
            entry.fields.push((key, value));
        }
    }

    if let Some(entry) = current {
        entries.push(entry);
    }

    entries
}

fn parse_field(line: &str) -> Option<(String, String)> {
    let eq_pos = line.find('=');
    let space_pos = line.find(' ');

    let sep = match (eq_pos, space_pos) {
        (Some(e), Some(s)) => {
            if e < s {
                e
            } else {
                s
            }
        }
        (Some(e), None) => e,
        (None, Some(s)) => s,
        (None, None) => return None,
    };

    let key = line[..sep].trim().to_string();
    let value = line[sep + 1..].trim().to_string();

    if key.is_empty() {
        return None;
    }

    // Normalize key: Title-Case
    let key = normalize_key(&key);

    Some((key, value))
}

fn normalize_key(key: &str) -> String {
    let lower = key.to_lowercase();
    match lower.as_str() {
        "hostname" => "HostName".to_string(),
        "user" => "User".to_string(),
        "port" => "Port".to_string(),
        "identityfile" => "IdentityFile".to_string(),
        "proxyjump" => "ProxyJump".to_string(),
        "proxycommand" => "ProxyCommand".to_string(),
        "forwardagent" => "ForwardAgent".to_string(),
        "serveraliveinterval" => "ServerAliveInterval".to_string(),
        "serveralivecountmax" => "ServerAliveCountMax".to_string(),
        "stricthostkeychecking" => "StrictHostKeyChecking".to_string(),
        "userknownhostsfile" => "UserKnownHostsFile".to_string(),
        "addkeystoagent" => "AddKeysToAgent".to_string(),
        "identitiesonly" => "IdentitiesOnly".to_string(),
        "localforward" => "LocalForward".to_string(),
        "remoteforward" => "RemoteForward".to_string(),
        "dynamicforward" => "DynamicForward".to_string(),
        "compression" => "Compression".to_string(),
        "tcpkeepalive" => "TCPKeepAlive".to_string(),
        "connecttimeout" => "ConnectTimeout".to_string(),
        "batchmode" => "BatchMode".to_string(),
        "passwordauthentication" => "PasswordAuthentication".to_string(),
        "pubkeyauthentication" => "PubkeyAuthentication".to_string(),
        "requesttty" => "RequestTTY".to_string(),
        _ => key.to_string(),
    }
}

pub fn write_config(entries: &[SshConfigEntry]) -> Result<(), String> {
    let mut content = String::new();

    for entry in entries {
        content.push_str(&format!("Host {}\n", entry.host_pattern));
        for (key, value) in &entry.fields {
            content.push_str(&format!("    {} {}\n", key, value));
        }
        content.push('\n');
    }

    let path = config_path();
    fs::write(&path, content).map_err(|e| format!("failed to write config: {}", e))
}

pub fn read_raw_config() -> String {
    let path = config_path();
    fs::read_to_string(&path).unwrap_or_default()
}

pub fn write_raw_config(content: &str) -> Result<(), String> {
    let path = config_path();
    fs::write(&path, content).map_err(|e| format!("failed to write config: {}", e))
}

pub fn find_hosts_using_key(key_name: &str) -> Vec<String> {
    let entries = parse_config();
    let ssh_dir = crate::ssh::ssh_dir();
    let full_path = ssh_dir.join(key_name).to_string_lossy().to_string();
    let full_path_tilde = format!("~/.ssh/{}", key_name);

    entries
        .iter()
        .filter(|e| {
            e.fields.iter().any(|(k, v)| {
                k == "IdentityFile"
                    && (v == &full_path || v == &full_path_tilde || v == key_name)
            })
        })
        .map(|e| e.host_pattern.clone())
        .collect()
}

pub fn get_entry_fields(entry: &SshConfigEntry) -> Vec<(String, String)> {
    // Return fields with common ones first, in a nice order
    let priority = [
        "HostName",
        "User",
        "Port",
        "IdentityFile",
        "ProxyJump",
        "ProxyCommand",
        "ForwardAgent",
        "ServerAliveInterval",
    ];

    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Priority fields first
    for &pkey in &priority {
        for (k, v) in &entry.fields {
            if k.eq_ignore_ascii_case(pkey) && seen.insert(k.to_lowercase()) {
                result.push((k.clone(), v.clone()));
            }
        }
    }

    // Remaining fields
    for (k, v) in &entry.fields {
        if seen.insert(k.to_lowercase()) {
            result.push((k.clone(), v.clone()));
        }
    }

    result
}
