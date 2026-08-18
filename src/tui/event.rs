use crate::export::Exporter;
use crate::fetcher::MetadataFetcher;
use crate::models::{ExportFormat, NewClip, UpdateClip};
use crate::tui::app::{App, InputMode};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

pub enum EventResult {
    Continue,
    Exit,
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    app.clear_status();

    match app.mode {
        InputMode::Normal => handle_normal_key(app, key),
        InputMode::Search => handle_search_key(app, key),
        InputMode::Command => handle_command_key(app, key),
        InputMode::AddClip => handle_add_clip_key(app, key),
        InputMode::EditTags => handle_edit_tags_key(app, key),
        InputMode::Export => handle_export_key(app, key),
        InputMode::ConfirmDelete => handle_confirm_delete_key(app, key),
        InputMode::HelpModal => handle_help_key(app, key),
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    match key.code {
        KeyCode::Char('q') => return Ok(EventResult::Exit),
        KeyCode::Char('j') | KeyCode::Down => app.next_clip(),
        KeyCode::Char('k') | KeyCode::Up => app.previous_clip(),
        KeyCode::Char('J') => app.scroll_detail_down(),
        KeyCode::Char('K') => app.scroll_detail_up(),
        KeyCode::Char('/') => {
            app.mode = InputMode::Search;
        }
        KeyCode::Char(':') => {
            app.mode = InputMode::Command;
            app.command_input.clear();
        }
        KeyCode::Char('n') => {
            app.mode = InputMode::AddClip;
            app.add_form = Default::default();
        }
        KeyCode::Char('e') => {
            app.mode = InputMode::Export;
        }
        KeyCode::Char('t') => {
            if let Some(clip) = app.selected_clip() {
                app.tag_input = clip.tags.join(", ");
                app.mode = InputMode::EditTags;
            } else {
                app.set_status("No clip selected to tag", true);
            }
        }
        KeyCode::Char('d') | KeyCode::Char('x') => {
            if app.selected_clip().is_some() {
                app.mode = InputMode::ConfirmDelete;
            } else {
                app.set_status("No clip selected to delete", true);
            }
        }
        KeyCode::Char('r') => {
            app.reload_clips()?;
            app.reload_tags()?;
            app.set_status("Clips refreshed", false);
        }
        KeyCode::Char('?') => {
            app.previous_mode = app.mode.clone();
            app.mode = InputMode::HelpModal;
        }
        KeyCode::Enter => {
            if let Some(clip) = app.selected_clip() {
                let url = clip.url.clone();
                if let Err(e) = open::that(&url) {
                    app.set_status(format!("Failed to open browser: {}", e), true);
                } else {
                    app.set_status(format!("Opened {}", url), false);
                }
            }
        }
        KeyCode::Esc => {
            if !app.search_query.is_empty() {
                app.search_query.clear();
                app.reload_clips()?;
                app.set_status("Search filter cleared", false);
            }
        }
        _ => {}
    }
    Ok(EventResult::Continue)
}

fn handle_search_key(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
        }
        KeyCode::Enter => {
            app.mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            let _ = app.reload_clips();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            let _ = app.reload_clips();
        }
        _ => {}
    }
    Ok(EventResult::Continue)
}

fn handle_command_key(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
            app.command_input.clear();
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        KeyCode::Enter => {
            let cmd = app.command_input.trim().to_string();
            app.command_input.clear();
            app.mode = InputMode::Normal;
            return execute_command(app, &cmd);
        }
        _ => {}
    }
    Ok(EventResult::Continue)
}

fn execute_command(app: &mut App, cmd: &str) -> Result<EventResult> {
    if cmd == "q" || cmd == "quit" || cmd == "exit" {
        return Ok(EventResult::Exit);
    }

    if cmd == "help" {
        app.mode = InputMode::HelpModal;
        return Ok(EventResult::Continue);
    }

    if cmd.starts_with("new ") || cmd.starts_with("clip ") {
        let url_part = if cmd.starts_with("new ") {
            &cmd[4..]
        } else {
            &cmd[5..]
        }
        .trim();

        if url_part.is_empty() {
            app.set_status("Usage: :new <URL>", true);
            return Ok(EventResult::Continue);
        }

        app.set_status(format!("Fetching metadata for {}...", url_part), false);
        match MetadataFetcher::new() {
            Ok(fetcher) => {
                match fetcher.fetch_and_extract(url_part, None, Vec::new(), None) {
                    Ok(new_clip) => match app.db.insert_clip(new_clip) {
                        Ok(clip) => {
                            app.reload_clips()?;
                            app.reload_tags()?;
                            app.set_status(format!("Clipped: {}", clip.title), false);
                        }
                        Err(e) => app.set_status(format!("Failed to save clip: {}", e), true),
                    },
                    Err(e) => app.set_status(format!("Failed to fetch URL: {}", e), true),
                }
            }
            Err(e) => app.set_status(format!("Fetcher error: {}", e), true),
        }
        return Ok(EventResult::Continue);
    }

    if cmd.starts_with("export") {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let fmt = if parts.len() > 1 {
            match parts[1].to_lowercase().as_str() {
                "csv" => ExportFormat::Csv,
                "html" => ExportFormat::Html,
                _ => ExportFormat::Json,
            }
        } else {
            ExportFormat::Json
        };

        let file_path = if parts.len() > 2 {
            parts[2]
        } else {
            match fmt {
                ExportFormat::Json => "clipper_export.json",
                ExportFormat::Csv => "clipper_export.csv",
                ExportFormat::Html => "clipper_export.html",
            }
        };

        match Exporter::export_clips(&app.clips, fmt, Some(std::path::Path::new(file_path))) {
            Ok(_) => app.set_status(format!("Exported {} clips to {}", app.clips.len(), file_path), false),
            Err(e) => app.set_status(format!("Export failed: {}", e), true),
        }
        return Ok(EventResult::Continue);
    }

    if cmd.starts_with("tag ") {
        let tags_part = cmd[4..].trim();
        if let Some(clip) = app.selected_clip() {
            let clip_id = clip.id;
            let tags: Vec<String> = tags_part
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            match app.db.update_clip(UpdateClip {
                id: clip_id,
                title: None,
                description: None,
                tags: Some(tags),
                notes: None,
            }) {
                Ok(_) => {
                    app.reload_clips()?;
                    app.reload_tags()?;
                    app.set_status("Tags updated successfully", false);
                }
                Err(e) => app.set_status(format!("Tag update failed: {}", e), true),
            }
        } else {
            app.set_status("No clip selected", true);
        }
        return Ok(EventResult::Continue);
    }

    if cmd == "del" || cmd == "delete" {
        if let Some(clip) = app.selected_clip() {
            let clip_id = clip.id;
            match app.db.delete_clip(clip_id) {
                Ok(_) => {
                    app.reload_clips()?;
                    app.reload_tags()?;
                    app.set_status("Clip deleted", false);
                }
                Err(e) => app.set_status(format!("Delete failed: {}", e), true),
            }
        }
        return Ok(EventResult::Continue);
    }

    app.set_status(format!("Unknown command: :{}", cmd), true);
    Ok(EventResult::Continue)
}

fn handle_add_clip_key(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
        }
        KeyCode::Tab => {
            app.add_form.active_field = (app.add_form.active_field + 1) % 3;
        }
        KeyCode::Enter => {
            let url = app.add_form.url.trim().to_string();
            if url.is_empty() {
                app.set_status("URL cannot be empty", true);
                return Ok(EventResult::Continue);
            }

            let tags: Vec<String> = app
                .add_form
                .tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let notes = if app.add_form.notes.trim().is_empty() {
                None
            } else {
                Some(app.add_form.notes.trim().to_string())
            };

            app.mode = InputMode::Normal;
            app.set_status("Fetching page metadata...", false);

            if let Ok(fetcher) = MetadataFetcher::new() {
                match fetcher.fetch_and_extract(&url, None, tags, notes.clone()) {
                    Ok(new_clip) => match app.db.insert_clip(new_clip) {
                        Ok(clip) => {
                            app.reload_clips()?;
                            app.reload_tags()?;
                            app.set_status(format!("Clipped: {}", clip.title), false);
                        }
                        Err(e) => app.set_status(format!("Save error: {}", e), true),
                    },
                    Err(e) => {
                        // Fallback clip creation
                        let fallback = NewClip {
                            url: url.clone(),
                            title: url.clone(),
                            description: None,
                            tags: app.add_form.tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                            notes,
                            content_text: None,
                            screenshot_path: None,
                            favicon_url: None,
                            author: None,
                            site_name: None,
                            reading_time_mins: None,
                        };
                        let _ = app.db.insert_clip(fallback);
                        app.reload_clips()?;
                        app.reload_tags()?;
                        app.set_status(format!("Saved clip (Fetch error: {})", e), false);
                    }
                }
            }
        }
        KeyCode::Backspace => match app.add_form.active_field {
            0 => {
                app.add_form.url.pop();
            }
            1 => {
                app.add_form.tags.pop();
            }
            2 => {
                app.add_form.notes.pop();
            }
            _ => {}
        },
        KeyCode::Char(c) => match app.add_form.active_field {
            0 => app.add_form.url.push(c),
            1 => app.add_form.tags.push(c),
            2 => app.add_form.notes.push(c),
            _ => {}
        },
        _ => {}
    }
    Ok(EventResult::Continue)
}

fn handle_edit_tags_key(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.tag_input.pop();
        }
        KeyCode::Char(c) => {
            app.tag_input.push(c);
        }
        KeyCode::Enter => {
            if let Some(clip) = app.selected_clip() {
                let clip_id = clip.id;
                let tags: Vec<String> = app
                    .tag_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                match app.db.update_clip(UpdateClip {
                    id: clip_id,
                    title: None,
                    description: None,
                    tags: Some(tags),
                    notes: None,
                }) {
                    Ok(_) => {
                        app.reload_clips()?;
                        app.reload_tags()?;
                        app.set_status("Tags updated", false);
                    }
                    Err(e) => app.set_status(format!("Tag update failed: {}", e), true),
                }
            }
            app.mode = InputMode::Normal;
        }
        _ => {}
    }
    Ok(EventResult::Continue)
}

fn handle_export_key(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    match key.code {
        KeyCode::Esc => {
            app.mode = InputMode::Normal;
        }
        KeyCode::Tab => {
            app.export_format = match app.export_format {
                ExportFormat::Json => {
                    app.export_path_input = "clipper_export.csv".to_string();
                    ExportFormat::Csv
                }
                ExportFormat::Csv => {
                    app.export_path_input = "clipper_export.html".to_string();
                    ExportFormat::Html
                }
                ExportFormat::Html => {
                    app.export_path_input = "clipper_export.json".to_string();
                    ExportFormat::Json
                }
            };
        }
        KeyCode::Enter => {
            let path = std::path::Path::new(&app.export_path_input);
            match Exporter::export_clips(&app.clips, app.export_format, Some(path)) {
                Ok(_) => app.set_status(format!("Exported {} clips to {}", app.clips.len(), app.export_path_input), false),
                Err(e) => app.set_status(format!("Export failed: {}", e), true),
            }
            app.mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.export_path_input.pop();
        }
        KeyCode::Char(c) => {
            app.export_path_input.push(c);
        }
        _ => {}
    }
    Ok(EventResult::Continue)
}

fn handle_confirm_delete_key(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            if let Some(clip) = app.selected_clip() {
                let clip_id = clip.id;
                match app.db.delete_clip(clip_id) {
                    Ok(_) => {
                        app.reload_clips()?;
                        app.reload_tags()?;
                        app.set_status("Clip deleted", false);
                    }
                    Err(e) => app.set_status(format!("Delete failed: {}", e), true),
                }
            }
            app.mode = InputMode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = InputMode::Normal;
        }
        _ => {}
    }
    Ok(EventResult::Continue)
}

fn handle_help_key(app: &mut App, key: KeyEvent) -> Result<EventResult> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
            app.mode = app.previous_mode.clone();
        }
        _ => {}
    }
    Ok(EventResult::Continue)
}
