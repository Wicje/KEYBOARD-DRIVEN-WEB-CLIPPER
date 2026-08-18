# 🔖 Clipper — Keyboard-Driven Terminal Web Clipper

[![CI & Release](https://github.com/Wicje/KEYBOARD-DRIVEN-WEB-CLIPPER/actions/workflows/release.yml/badge.svg)](https://github.com/Wicje/KEYBOARD-DRIVEN-WEB-CLIPPER/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/Wicje/KEYBOARD-DRIVEN-WEB-CLIPPER)](https://github.com/Wicje/KEYBOARD-DRIVEN-WEB-CLIPPER/releases)

> **Clipper** is a fast, privacy-focused, keyboard-driven web clipper and bookmark manager for the terminal. Built with Rust, SQLite FTS5, and Ratatui.

---

## 🚀 One-Line Installation

Install `clipper` along with the full Josephan tools suite:

```bash
curl -fsSL install.josephan.dev | bash
```

Or build and install locally via Cargo:

```bash
cargo install --path .
```

---

## ✨ Features

- **⚡ Instant Web Clipping**: Clip any URL directly from CLI or TUI with automatic web page metadata extraction (OpenGraph titles, descriptions, favicons, author, estimated reading time, and readable body text).
- **⌨️ Vim Keybindings**: Full keyboard control with `/` for live search, `:` for commands, `j/k` for navigation, `Enter` to open in browser, `n` to clip new URLs, `t` to edit tags, and `d` to delete.
- **🔍 SQLite FTS5 Full-Text Search**: Ultra-fast searching across titles, descriptions, notes, extracted body text, tags, and URLs.
- **🏷️ Flexible Tag Management**: Categorize clips with tags, perform tag filtering, rename tags globally, or list clip statistics per tag.
- **📦 Portable Multi-Format Exports**: Export saved clips to **JSON**, **CSV** (for Notion or Excel import), or **HTML** (standalone offline web gallery).
- **📂 XDG Base Directory Compliant**: Stores data in `~/.local/share/clipper/clips.db` and configuration in `~/.config/clipper/`.

---

## 💻 Quick Commands

```bash
# Clip a web page
clipper clip https://rust-lang.org --tags "rust,programming"

# Search clips by keyword (SQLite FTS5)
clipper search "multi-paradigm"

# Launch interactive Vim TUI
clipper tui
```

---

## 🎮 Vim Keybindings Reference

| Key | Mode | Description |
| :--- | :--- | :--- |
| `j` / `Down` | Normal | Navigate down clip list |
| `k` / `Up` | Normal | Navigate up clip list |
| `Enter` | Normal | Open clip in browser |
| `/` | Normal -> Search | Enter live search filter mode |
| `:` | Normal -> Command | Enter command prompt mode |
| `n` | Normal -> New | Open clip URL modal |
| `e` | Normal -> Export | Open export modal (JSON/CSV/HTML) |
| `q` | Normal | Quit Clipper |

---

## 🧪 Testing

```bash
cargo test
```
