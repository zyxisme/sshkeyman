# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`sshkeyman` is a web-based SSH key & config manager built with Rust. Backend: Axum + Askama templates. Manages `~/.ssh/` keys and config. Edition 2024.

## Build & Run

```bash
cargo build                    # compile
cargo run                      # start at http://127.0.0.1:3000
cargo run -- --port 8080       # custom port
cargo run -- --bind 0.0.0.0:9000  # custom bind address
cargo clippy                   # lint (must pass clean)
cargo fmt --check              # check formatting
```

## Architecture

| Module | Role |
|---|---|
| `main.rs` | Tokio entry, clap CLI args (`--host`, `--port`, `--bind`) |
| `ssh.rs` | SSH key filesystem ops: `list_keys()`, `generate_key()`, `delete_key()`, permissions |
| `config.rs` | SSH config parser/writer: `parse_config()`, `write_config()`, `find_hosts_using_key()`, raw edit |
| `export.rs` | Key pair export/import + full `~/.ssh/` backup/restore (`backup_all`, `restore_all`) |
| `routes/mod.rs` | Router: all routes + static file serving |
| `routes/index.rs` | `GET /` — key list + detail + key-host association |
| `routes/keys.rs` | `POST /generate`, `POST /delete` — key operations |
| `routes/transfer.rs` | `GET /export/:name`, `POST /import` — single key backup |
| `routes/config.rs` | Config CRUD, raw editor, backup/restore endpoints |

### Routes

| Method | Path | Function |
|--------|------|----------|
| GET | `/` | Key list + detail |
| POST | `/generate` | Generate new SSH key |
| POST | `/delete` | Delete key |
| GET | `/export/:name` | Download key tar.gz |
| POST | `/import` | Upload key tar.gz |
| GET | `/config` | Config host list |
| GET | `/config/edit?host=xxx` | Edit host form |
| POST | `/config/save` | Save host config |
| POST | `/config/add` | New host |
| POST | `/config/delete` | Delete host |
| GET | `/config/raw` | Raw config editor |
| POST | `/config/raw/save` | Save raw config |
| GET | `/backup` | Download full ~/.ssh backup |
| POST | `/restore` | Upload full restore |

### Templates

- `templates/index.html` — key list sidebar, detail panel, generate/import forms
- `templates/config.html` — config host cards, restore form
- `templates/config_edit.html` — host edit form with dynamic fields (JS)
- `templates/config_raw.html` — raw config textarea editor
- `static/style.css` — all styling

### Key types

- `ssh::SshKeyInfo` — key metadata
- `config::SshConfigEntry` — parsed config block (host_pattern + fields)
- `ssh::KeyType` — `Ed25519` / `Rsa4096` (PartialEq, Clone, Copy)

### Design decisions

- `~/.ssh/` is the single source of truth — no cached state
- `ssh-key` for parsing `.pub` files; `ssh-keygen` subprocess for generation
- SSH config parsed line-by-line, key names normalized to Title-Case
- Flash messages via query params (`?flash=...` / `?flash_error=...`)
- Clipboard copy via browser `navigator.clipboard.writeText()` JS
- Config edit form uses JS to dynamically add/remove field rows
- IdentityFile field renders as `<select>` dropdown populated from existing keys, with "Custom path..." fallback
- Config save uses `RawForm` + manual URL-decode parsing (not `Form<T>`) to handle repeated field names that may arrive as single value or array
- Backup = tar.gz of keys + config; restore refuses if files exist
- Default config fields: HostName, User, Port, IdentityFile — always shown in edit form (empty if missing)

### Dependencies

`axum` 0.8 (multipart), `tokio` 1, `askama` 0.15, `serde` 1, `serde_json` 1, `tower-http` 0.6, `ssh-key` 0.6, `dirs` 6, `tar` 0.4, `flate2` 1, `clap` 4 (derive)
