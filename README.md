# SSHKeyman
[![Release](https://github.com/zyxisme/sshkeyman/actions/workflows/release.yml/badge.svg)](https://github.com/zyxisme/sshkeyman/actions/workflows/release.yml)
[English](#english) | [中文](#中文)

---

## English

A web-based SSH key and config manager, built with Rust + Axum + Askama.

### Features

- **Key Management** — List, generate (Ed25519 / RSA-4096), delete SSH keys
- **Copy & Export** — One-click copy public key to clipboard, export/import key pairs as `.tar.gz`
- **Config Management** — View, edit, delete `~/.ssh/config` Host blocks via form UI
- **Key-Config Association** — See which Hosts reference a given key
- **Raw Editor** — Directly edit the raw config file text
- **Full Backup** — Backup and restore entire `~/.ssh/` (keys + config) as `.tar.gz`
- **CLI Options** — Customizable listen address via `--host`, `--port`, `--bind`

### Quick Start

```bash
cargo run
```

Open http://127.0.0.1:3000 in your browser.

### CLI Usage

```bash
sshkeyman                        # default: 127.0.0.1:3000
sshkeyman --port 8080            # custom port
sshkeyman --host 0.0.0.0         # listen on all interfaces
sshkeyman --bind 0.0.0.0:9000    # full bind address
```

### Build

```bash
cargo build --release
# binary at target/release/sshkeyman
```

### Tech Stack

- **Backend**: Axum + Tokio
- **Templates**: Askama (server-side rendered)
- **SSH Parsing**: `ssh-key` crate, `ssh-keygen` subprocess
- **Archive**: `tar` + `flate2`
- **CLI**: `clap`

### License

MIT

---

## 中文

一个基于 Web 的 SSH 密钥和配置管理工具，使用 Rust + Axum + Askama 构建。

### 功能

- **密钥管理** — 查看、生成（Ed25519 / RSA-4096）、删除 SSH 密钥
- **复制与导出** — 一键复制公钥到剪贴板，导入/导出密钥对为 `.tar.gz`
- **配置管理** — 通过表单界面查看、编辑、删除 `~/.ssh/config` 中的 Host 配置块
- **密钥关联** — 查看哪些 Host 配置引用了指定密钥
- **原始编辑** — 直接编辑 config 文件原始文本
- **全量备份** — 将整个 `~/.ssh/`（密钥 + 配置）打包备份或恢复
- **命令行参数** — 通过 `--host`、`--port`、`--bind` 自定义监听地址

### 快速开始

```bash
cargo run
```

浏览器打开 http://127.0.0.1:3000

### 命令行用法

```bash
sshkeyman                        # 默认: 127.0.0.1:3000
sshkeyman --port 8080            # 自定义端口
sshkeyman --host 0.0.0.0         # 监听所有网卡
sshkeyman --bind 0.0.0.0:9000    # 完整绑定地址
```

### 构建

```bash
cargo build --release
# 二进制文件位于 target/release/sshkeyman
```

### 技术栈

- **后端**: Axum + Tokio
- **模板**: Askama（服务端渲染）
- **SSH 解析**: `ssh-key` crate + `ssh-keygen` 子进程
- **归档**: `tar` + `flate2`
- **命令行**: `clap`

### 许可证

MIT
