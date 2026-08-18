# 🔖 Clipper — Keyboard-Driven Terminal Web Clipper

> **Clipper** is a fast, privacy-focused, keyboard-driven web clipper and bookmark manager for the terminal. Built with Rust, SQLite FTS5, and Ratatui.

---

## ✨ Features

- **⚡ Instant Web Clipping**: Clip any URL directly from CLI or TUI with automatic web page metadata extraction (OpenGraph titles, descriptions, favicons, author, estimated reading time, and readable body text).
- **⌨️ Vim Keybindings**: Full keyboard control with `/` for live search, `:` for commands, `j/k` for navigation, `Enter` to open in browser, `n` to clip new URLs, `t` to edit tags, and `d` to delete.
- **🔍 SQLite FTS5 Full-Text Search**: Ultra-fast searching across titles, descriptions, notes, extracted body text, tags, and URLs.
- **🏷️ Flexible Tag Management**: Categorize clips with tags, perform tag filtering, rename tags globally, or list clip statistics per tag.
- **📦 Portable Multi-Format Exports**: Export saved clips to **JSON** (schema-compliant), **CSV** (for Notion or Excel import), or **HTML** (a responsive, standalone offline web gallery with light/dark theme toggle and instant in-browser search).
- **📂 XDG Base Directory Compliant**: Keeps data safely stored in `~/.local/share/clipper/clips.db` and configuration in `~/.config/clipper/`.

---

## 🚀 Quick Start

### Installation

```bash
# Build and install locally via Cargo
cargo install --path .
```

### Basic CLI Commands

```bash
# 1. Clip a web page (automatically extracts metadata, body text, and reading time)
clipper clip https://rust-lang.org --tags "rust,programming" --notes "Official Rust language homepage"

# 2. Search clips by keyword (uses SQLite FTS5 index)
clipper search "multi-paradigm"

# 3. Filter clips by tag
clipper list --tag rust

# 4. Open clip in default browser by ID
clipper open 1

# 5. Show full clip details and extracted article text
clipper show 1

# 6. Edit tags or notes
clipper edit 1 --add-tag "cli,systems" --notes "Updated notes"

# 7. Manage tags
clipper tag list
clipper tag rename "programming" "dev"

# 8. Export clips (JSON, CSV, or standalone HTML)
clipper export --format html --output collection.html
clipper export --format json --output clips.json
```

---

## 🎮 Interactive Vim-Driven Terminal UI (TUI)

Launch the interactive TUI by running:

```bash
clipper tui
# or simply
clipper
```

### Vim Keybindings Reference

| Key | Mode | Description |
| :--- | :--- | :--- |
| `j` / `Down` | Normal | Navigate down the clip list |
| `k` / `Up` | Normal | Navigate up the clip list |
| `J` / `Shift+Down` | Normal | Scroll detail inspector down |
| `K` / `Shift+Up` | Normal | Scroll detail inspector up |
| `Enter` | Normal | Open selected clip URL in default web browser |
| `/` | Normal -> Search | Enter live search filter mode |
| `:` | Normal -> Command | Enter command-line mode |
| `n` | Normal -> New | Open modal to clip a new URL |
| `e` | Normal -> Export | Open export modal (JSON/CSV/HTML) |
| `t` | Normal -> Tag | Edit tags for selected clip |
| `d` / `x` | Normal -> Delete | Confirm deletion of selected clip |
| `r` | Normal | Refresh clip list & tag statistics |
| `?` | Normal -> Help | Toggle help overlay window |
| `q` | Normal | Quit Clipper |

### Vim Command-Line Commands (`:`)

In TUI, press `:` to open the command line prompt:

- `:new <URL>` or `:clip <URL>` — Clip a new URL directly from TUI.
- `:export <json|csv|html> [filename]` — Export clips to file.
- `:tag <tag1,tag2>` — Update tags on selected clip.
- `:del` or `:delete` — Delete selected clip.
- `:help` — Show help dialog.
- `:q` or `:quit` — Exit TUI.

---

## 🗄️ Database & Storage Architecture

Data is stored in accordance with the XDG Base Directory specification:

- **Database**: `~/.local/share/clipper/clips.db`
- **Assets**: `~/.local/share/clipper/assets/`
- **Config**: `~/.config/clipper/config.toml`

### SQLite Schema

```sql
CREATE TABLE clips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    tags TEXT NOT NULL, -- Comma separated tags
    notes TEXT,
    content_text TEXT, -- Extracted article body text
    screenshot_path TEXT,
    favicon_url TEXT,
    author TEXT,
    site_name TEXT,
    reading_time_mins INTEGER,
    date_saved TEXT NOT NULL,
    date_updated TEXT NOT NULL
);

CREATE TABLE clip_tags (
    clip_id INTEGER NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (clip_id, tag),
    FOREIGN KEY (clip_id) REFERENCES clips(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE clips_fts USING fts5(
    clip_id UNINDEXED,
    title,
    description,
    content_text,
    tags,
    notes,
    url
);
```

---

## 🧪 Testing

Run the full automated integration test suite:

```bash
cargo test
```

---

## 📄 License

Distributed under the MIT License. See `LICENSE` for details.
