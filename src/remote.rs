use std::io::Read;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

use ssh2::Session;

use crate::ssh;

pub struct RemoteCopyConfig<'a> {
    pub key_name: &'a str,
    pub host: &'a str,
    pub port: u16,
    pub username: &'a str,
    pub auth_method: &'a str,
    pub password: Option<&'a str>,
    pub key_path: Option<&'a str>,
    pub passphrase: Option<&'a str>,
}

pub fn copy_key_to_remote(cfg: &RemoteCopyConfig) -> Result<(), String> {
    let keys = ssh::list_keys();
    let key_info = keys
        .iter()
        .find(|k| k.name == cfg.key_name)
        .ok_or_else(|| format!("key '{}' not found", cfg.key_name))?;

    let pub_key = key_info.public_key_content.trim().to_string();

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port)
        .parse()
        .map_err(|e| format!("invalid address '{}:{}': {}", cfg.host, cfg.port, e))?;

    let tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(10))
        .map_err(|e| format!("connection to {} failed: {}", cfg.host, e))?;

    let mut session = Session::new().map_err(|e| format!("failed to create SSH session: {}", e))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;

    match cfg.auth_method {
        "password" => {
            let pwd = cfg.password.ok_or("no password provided")?;
            session
                .userauth_password(cfg.username, pwd)
                .map_err(|e| format!("password authentication failed: {}", e))?;
        }
        "key" => {
            let kp = cfg.key_path.ok_or("no key path provided")?;
            let pp = cfg.passphrase.filter(|s| !s.is_empty());
            session
                .userauth_pubkey_file(cfg.username, None, Path::new(kp), pp)
                .map_err(|e| format!("key authentication failed: {}", e))?;
        }
        other => return Err(format!("unknown auth method: {}", other)),
    }

    if !session.authenticated() {
        return Err("authentication failed".to_string());
    }

    // Escape single quotes in public key content
    let escaped = pub_key.replace('\'', "'\\''");
    let cmd = format!(
        "mkdir -p ~/.ssh && echo '{}' >> ~/.ssh/authorized_keys \
         && chmod 600 ~/.ssh/authorized_keys && chmod 700 ~/.ssh",
        escaped
    );

    let mut channel = session
        .channel_session()
        .map_err(|e| format!("failed to open channel: {}", e))?;
    channel
        .exec(&cmd)
        .map_err(|e| format!("failed to execute remote command: {}", e))?;

    let mut output = String::new();
    let _ = channel.read_to_string(&mut output);
    channel
        .wait_close()
        .map_err(|e| format!("channel close error: {}", e))?;

    let exit_status = channel
        .exit_status()
        .map_err(|e| format!("failed to get exit status: {}", e))?;

    if exit_status != 0 {
        return Err(format!(
            "remote command failed (exit {}): {}",
            exit_status,
            output.trim()
        ));
    }

    Ok(())
}
