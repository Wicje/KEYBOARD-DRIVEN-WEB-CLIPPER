use crate::models::ExportFormat;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "clipper",
    author = "Clipper Contributors",
    version = "0.1.0",
    about = "⚡ Keyboard-driven web clipper and bookmark manager for your terminal",
    long_about = "A fast, privacy-focused terminal web clipper with SQLite FTS5 search, Vim keybindings, metadata extraction, offline HTML previews, and JSON/CSV/HTML export."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Launch interactive Terminal UI directly
    #[arg(short, long)]
    pub tui: bool,

    /// Specify custom database path
    #[arg(long, global = true)]
    pub db: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Clip a new URL, fetch metadata, and save to SQLite
    Clip(ClipArgs),

    /// Search clips by keyword, tag, or date range
    Search(SearchArgs),

    /// List saved clips in terminal or TUI
    List(ListArgs),

    /// Display details and extracted text of a single clip
    Show(ShowArgs),

    /// Open clip URL in default web browser
    Open(OpenArgs),

    /// Edit clip metadata, tags, or notes
    Edit(EditArgs),

    /// Delete a clip by ID
    Delete(DeleteArgs),

    /// Tag management subcommands (list, add, remove, rename, delete)
    Tag(TagArgs),

    /// Export saved clips to JSON, CSV, or offline HTML collection
    Export(ExportArgs),

    /// Launch interactive Vim-driven Terminal UI
    Tui,
}

#[derive(Args, Debug)]
pub struct ClipArgs {
    /// The URL of the web page to clip
    pub url: String,

    /// Override extracted web page title
    #[arg(short, long)]
    pub title: Option<String>,

    /// Comma-separated tags (e.g. --tags "design,rust,article")
    #[arg(short = 'T', long)]
    pub tags: Option<String>,

    /// Personal notes for this clip
    #[arg(short, long)]
    pub notes: Option<String>,

    /// Skip HTTP network fetch and save URL directly
    #[arg(long)]
    pub no_fetch: bool,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search query (matches title, description, content text, tags, notes, URL)
    pub query: Option<String>,

    /// Filter clips by specific tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Filter clips saved from date (YYYY-MM-DD)
    #[arg(long)]
    pub from: Option<String>,

    /// Filter clips saved up to date (YYYY-MM-DD)
    #[arg(long)]
    pub to: Option<String>,

    /// Limit number of results returned
    #[arg(short, long)]
    pub limit: Option<usize>,

    /// Open matching search results directly inside interactive TUI
    #[arg(long)]
    pub tui: bool,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter clips by specific tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Max number of clips to list
    #[arg(short, long, default_value = "25")]
    pub limit: usize,

    /// Launch TUI directly
    #[arg(long)]
    pub tui: bool,
}

#[derive(Args, Debug)]
pub struct ShowArgs {
    /// ID of the clip to display
    pub id: i64,
}

#[derive(Args, Debug)]
pub struct OpenArgs {
    /// ID of the clip to open in browser
    pub id: i64,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// ID of the clip to update
    pub id: i64,

    /// Update title
    #[arg(short, long)]
    pub title: Option<String>,

    /// Set/replace all tags (comma separated)
    #[arg(short, long)]
    pub tags: Option<String>,

    /// Add tags to existing clip (comma separated)
    #[arg(long)]
    pub add_tag: Option<String>,

    /// Remove tags from existing clip (comma separated)
    #[arg(long)]
    pub remove_tag: Option<String>,

    /// Update notes
    #[arg(short, long)]
    pub notes: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// ID of the clip to delete
    pub id: i64,

    /// Skip confirmation prompt
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct TagArgs {
    #[command(subcommand)]
    pub command: TagSubcommands,
}

#[derive(Subcommand, Debug)]
pub enum TagSubcommands {
    /// List all tags with clip counts
    List,

    /// Add tags to a clip by ID
    Add {
        /// Clip ID
        id: i64,
        /// Comma separated tags to add
        tags: String,
    },

    /// Remove tags from a clip by ID
    Remove {
        /// Clip ID
        id: i64,
        /// Comma separated tags to remove
        tags: String,
    },

    /// Rename a tag across all clips
    Rename {
        /// Existing tag name
        old_tag: String,
        /// New tag name
        new_tag: String,
    },

    /// Delete a tag from all clips
    Delete {
        /// Tag name to delete
        tag: String,
    },
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Export format: json, csv, or html
    #[arg(short, long, value_enum, default_value = "json")]
    pub format: ExportFormat,

    /// File path to save exported data (prints to stdout if omitted)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Filter exported clips by tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Filter exported clips by search query
    #[arg(short, long)]
    pub query: Option<String>,
}
