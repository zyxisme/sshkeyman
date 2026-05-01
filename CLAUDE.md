# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`sshkeyman` is a web-based SSH key & config manager built with Rust. Backend: Axum + Askama templates. Manages `~/.ssh/` keys and config. Edition 2024. UI is i18n-ready: zh-CN (default) + en, detected via `Accept-Language` header.

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
| `main.rs` | Tokio entry, clap CLI args (`--host`, `--port`, `--bind`), calls `i18n::init()` |
| `ssh.rs` | SSH key filesystem ops: `list_keys()`, `generate_key()`, `delete_key()`, permissions |
| `config.rs` | SSH config parser/writer: `parse_config()`, `write_config()`, `find_hosts_using_key()`, raw edit |
| `export.rs` | Key pair export/import + full `~/.ssh/` backup/restore (`backup_all`, `restore_all`) |
| `i18n.rs` | Locale detection (`Accept-Language` header), TOML-based translation lookup, flash message resolution |
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
- `locales/zh-CN.toml` — Chinese translations (default language)
- `locales/en.toml` — English translations (fallback)

### Key types

- `ssh::SshKeyInfo` — key metadata
- `config::SshConfigEntry` — parsed config block (host_pattern + fields)
- `ssh::KeyType` — `Ed25519` / `Rsa4096` (PartialEq, Clone, Copy)

### Design decisions

- `~/.ssh/` is the single source of truth — no cached state
- `ssh-key` for parsing `.pub` files; `ssh-keygen` subprocess for generation
- SSH config parsed line-by-line, key names normalized to Title-Case
- Flash messages via query params — use translation keys (`?flash=flash_deleted`, `?flash=flash_saved&flash_param=host`); backend errors passthrough as `?flash_error=...`
- Flash resolution: handler extracts `Accept-Language` → builds `Locale` → calls `resolve_flash(key, param)` → passes translated string to template
- i18n: TOML locale files loaded at startup into `LazyLock<LocaleMap>`, `&'static LocaleMap` passed to all Askama templates, accessed with `{{ t["key"] }}` bracket syntax
- Locale detection: parses `Accept-Language` header, matches `zh*`/`en*` prefix, defaults to zh-CN, falls back to en for missing keys
- JS-side strings in `config_edit.html` injected via `js_locale_json` template field
- Clipboard copy via browser `navigator.clipboard.writeText()` JS
- Config edit form uses JS to dynamically add/remove field rows
- IdentityFile field renders as `<select>` dropdown populated from existing keys, with translated "Custom path..." fallback
- Config save uses `RawForm` + manual URL-decode parsing (not `Form<T>`) to handle repeated field names that may arrive as single value or array
- Backup = tar.gz of keys + config; restore refuses if files exist
- Default config fields: HostName, User, Port, IdentityFile — always shown in edit form (empty if missing)

### Dependencies

`axum` 0.8 (multipart), `tokio` 1, `askama` 0.15, `serde` 1, `serde_json` 1, `tower-http` 0.6, `ssh-key` 0.6, `dirs` 6, `tar` 0.4, `flate2` 1, `clap` 4 (derive), `toml` 0.8, `unic-langid` 0.9

## Packaging & CI

### GitHub Actions (`.github/workflows/release.yml`)

- **On every push to main**: build + upload nightly binary artifact
- **On tag push** (`v*`): build + package `.tar.gz` (binary + `static/` + `locales/`) + upload to GitHub Release
- Release product: `sshkeyman-x86_64-linux.tar.gz` — contains `sshkeyman` binary, `static/style.css`, `locales/{en,zh-CN}.toml`. Extract and run from the extracted directory so the binary finds resource files at `.`.

### PKGBUILD (AUR)

VCS-based (`git+https://`) Arch Linux package. `build()` runs `cargo build --release` on user machine. Installs binary to `/usr/share/sshkeyman/` with a wrapper script in `/usr/bin/sshkeyman` that `cd`s before exec so `static/` and `locales/` are found at runtime. `pkgver()` auto-generated via `git describe`.

**AUR repo**: `/home/zyx/projs/aur-sshkeyman/` (separate from project repo, remote `ssh://aur@aur.archlinux.org/sshkeyman.git`, SSH key `~/.ssh/aur`)

**Update flow**:
```bash
cp PKGBUILD /home/zyx/projs/aur-sshkeyman/PKGBUILD
cd /home/zyx/projs/aur-sshkeyman
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "<message>"
git push origin master
```

