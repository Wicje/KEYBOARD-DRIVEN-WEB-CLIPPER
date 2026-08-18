use crate::tui::app::{App, InputMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main body (List + Detail)
            Constraint::Length(3), // Status & Input Bar
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    // Draw active modal overlay if any
    match app.mode {
        InputMode::HelpModal => draw_help_modal(f, app),
        InputMode::AddClip => draw_add_clip_modal(f, app),
        InputMode::EditTags => draw_edit_tags_modal(f, app),
        InputMode::Export => draw_export_modal(f, app),
        InputMode::ConfirmDelete => draw_confirm_delete_modal(f, app),
        _ => {}
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let mode_str = match app.mode {
        InputMode::Normal => (" NORMAL ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        InputMode::Search => (" SEARCH ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
        InputMode::Command => (" COMMAND ", Style::default().bg(Color::Magenta).fg(Color::White).add_modifier(Modifier::BOLD)),
        InputMode::AddClip => (" NEW CLIP ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
        InputMode::EditTags => (" EDIT TAGS ", Style::default().bg(Color::Blue).fg(Color::White).add_modifier(Modifier::BOLD)),
        InputMode::Export => (" EXPORT ", Style::default().bg(Color::LightCyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        InputMode::ConfirmDelete => (" DELETE ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
        InputMode::HelpModal => (" HELP ", Style::default().bg(Color::Gray).fg(Color::Black).add_modifier(Modifier::BOLD)),
    };

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25),
            Constraint::Length(12),
            Constraint::Min(20),
        ])
        .split(area);

    let title_widget = Paragraph::new(Span::styled(
        " 🔖 CLIPPER v0.1.0",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ))
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));

    let mode_widget = Paragraph::new(Span::styled(mode_str.0, mode_str.1))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));

    let stats_text = format!(
        " Clips: {} | Tags: {} | Press '?' for Help ",
        app.clips.len(),
        app.tags_summary.len()
    );
    let stats_widget = Paragraph::new(Span::styled(stats_text, Style::default().fg(Color::Gray)))
        .alignment(Alignment::Right)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));

    f.render_widget(title_widget, header_layout[0]);
    f.render_widget(mode_widget, header_layout[1]);
    f.render_widget(stats_widget, header_layout[2]);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_clip_list(f, app, main_chunks[0]);
    draw_clip_detail(f, app, main_chunks[1]);
}

fn draw_clip_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .clips
        .iter()
        .enumerate()
        .map(|(idx, clip)| {
            let is_selected = idx == app.selected_index;
            let prefix = if is_selected { "▶ " } else { "  " };

            let title = clip.display_title();
            let tags_formatted = if clip.tags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", clip.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "))
            };

            let title_line = Line::from(vec![
                Span::styled(prefix, if is_selected { Style::default().fg(Color::Yellow) } else { Style::default() }),
                Span::styled(
                    title,
                    if is_selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    },
                ),
            ]);

            let subtitle_line = Line::from(vec![
                Span::styled(format!("    {}", clip.url), Style::default().fg(Color::DarkGray)),
                Span::styled(tags_formatted, Style::default().fg(Color::Cyan)),
            ]);

            ListItem::new(vec![title_line, subtitle_line, Line::from("")])
        })
        .collect();

    let title_text = if app.search_query.is_empty() {
        format!(" Clips ({}) ", app.clips.len())
    } else {
        format!(" Search Results ({}) ", app.clips.len())
    };

    let list_widget = List::new(items)
        .block(
            Block::default()
                .title(title_text)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list_widget, area);
}

fn draw_clip_detail(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Clip Inspector & Content ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    if let Some(clip) = app.selected_clip() {
        let mut lines = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Title: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&clip.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("URL:   ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&clip.url, Style::default().fg(Color::Cyan)),
        ]));

        let tags_str = if clip.tags.is_empty() {
            "None".to_string()
        } else {
            clip.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ")
        };
        lines.push(Line::from(vec![
            Span::styled("Tags:  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(tags_str, Style::default().fg(Color::LightGreen)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("Saved: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(clip.date_saved.format("%Y-%m-%d %H:%M:%S UTC").to_string(), Style::default().fg(Color::Gray)),
            Span::styled(" | Reading Time: ", Style::default().fg(Color::Yellow)),
            Span::styled(format!("~{} min", clip.reading_time_mins.unwrap_or(1)), Style::default().fg(Color::Gray)),
        ]));

        if let Some(desc) = &clip.description {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Description:", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(Span::styled(desc, Style::default().fg(Color::White))));
        }

        if let Some(notes) = &clip.notes {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Personal Notes:", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD))));
            lines.push(Line::from(Span::styled(notes, Style::default().fg(Color::Yellow))));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Article Content / Text Preview:", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(Span::styled("----------------------------------------", Style::default().fg(Color::DarkGray))));

        if let Some(body) = &clip.content_text {
            for paragraph in body.split("\n\n") {
                lines.push(Line::from(Span::styled(paragraph, Style::default().fg(Color::Gray))));
                lines.push(Line::from(""));
            }
        } else {
            lines.push(Line::from(Span::styled("(No extracted readable text available)", Style::default().fg(Color::DarkGray))));
        }

        let detail_paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((app.detail_scroll, 0));

        f.render_widget(detail_paragraph, area);
    } else {
        let empty_paragraph = Paragraph::new(Span::styled(
            "No clips saved yet. Press 'n' or type ':new' to clip a web page!",
            Style::default().fg(Color::DarkGray),
        ))
        .block(block)
        .alignment(Alignment::Center);

        f.render_widget(empty_paragraph, area);
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let content = match app.mode {
        InputMode::Search => Line::from(vec![
            Span::styled("/ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(&app.search_query, Style::default().fg(Color::White)),
        ]),
        InputMode::Command => Line::from(vec![
            Span::styled(": ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(&app.command_input, Style::default().fg(Color::White)),
        ]),
        _ => {
            if let Some((msg, is_error)) = &app.status_message {
                Line::from(Span::styled(
                    msg,
                    if *is_error {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Green)
                    },
                ))
            } else {
                Line::from(vec![
                    Span::styled(" [j/k] ", Style::default().fg(Color::Yellow)),
                    Span::raw("Nav "),
                    Span::styled(" [/] ", Style::default().fg(Color::Yellow)),
                    Span::raw("Search "),
                    Span::styled(" [:] ", Style::default().fg(Color::Magenta)),
                    Span::raw("Cmd "),
                    Span::styled(" [Enter] ", Style::default().fg(Color::Green)),
                    Span::raw("Open "),
                    Span::styled(" [n] ", Style::default().fg(Color::Green)),
                    Span::raw("New "),
                    Span::styled(" [t] ", Style::default().fg(Color::Cyan)),
                    Span::raw("Tag "),
                    Span::styled(" [d] ", Style::default().fg(Color::Red)),
                    Span::raw("Del "),
                    Span::styled(" [?] ", Style::default().fg(Color::Gray)),
                    Span::raw("Help "),
                    Span::styled(" [q] ", Style::default().fg(Color::Red)),
                    Span::raw("Quit"),
                ])
            }
        }
    };

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn draw_help_modal(f: &mut Frame, _app: &App) {
    let area = centered_rect(65, 70, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("KEYBOARD SHORTCUTS & VIM BINDINGS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled(" Navigation:", Style::default().fg(Color::Yellow))]),
        Line::from("   j / Down     : Move selection down"),
        Line::from("   k / Up       : Move selection up"),
        Line::from("   J / Shift+Dn : Scroll detail inspector down"),
        Line::from("   K / Shift+Up : Scroll detail inspector up"),
        Line::from("   Enter        : Open selected clip in default browser"),
        Line::from(""),
        Line::from(vec![Span::styled(" Modes & Commands:", Style::default().fg(Color::Yellow))]),
        Line::from("   /            : Live Search Mode"),
        Line::from("   :            : Command Line Mode"),
        Line::from("   n or :new    : Open clip new URL modal"),
        Line::from("   e or :export : Export clips to JSON / CSV / HTML"),
        Line::from("   t or :tag    : Edit tags on selected clip"),
        Line::from("   d or :del    : Delete selected clip"),
        Line::from("   r            : Refresh list"),
        Line::from("   Esc          : Return to Normal mode / Clear search"),
        Line::from("   q or :q      : Quit Clipper"),
        Line::from(""),
        Line::from(vec![Span::styled(" Command Line Examples:", Style::default().fg(Color::Yellow))]),
        Line::from("   :new https://rust-lang.org"),
        Line::from("   :export json"),
        Line::from("   :tag design,rust"),
        Line::from("   :q"),
        Line::from(""),
        Line::from(Span::styled(" Press Esc or 'q' to close this help window ", Style::default().fg(Color::DarkGray))),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help & Keyboard Bindings ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, area);
}

fn draw_add_clip_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // URL
            Constraint::Length(3), // Tags
            Constraint::Length(3), // Notes
            Constraint::Min(2),    // Actions hint
        ])
        .split(area);

    let block_url = Block::default()
        .title(" URL (Required) ")
        .borders(Borders::ALL)
        .border_style(if app.add_form.active_field == 0 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) });

    let block_tags = Block::default()
        .title(" Tags (Comma separated, e.g. rust,article) ")
        .borders(Borders::ALL)
        .border_style(if app.add_form.active_field == 1 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) });

    let block_notes = Block::default()
        .title(" Notes (Optional) ")
        .borders(Borders::ALL)
        .border_style(if app.add_form.active_field == 2 { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::DarkGray) });

    let p_url = Paragraph::new(app.add_form.url.as_str()).block(block_url);
    let p_tags = Paragraph::new(app.add_form.tags.as_str()).block(block_tags);
    let p_notes = Paragraph::new(app.add_form.notes.as_str()).block(block_notes);

    let hint_text = Line::from(vec![
        Span::styled("[Tab] ", Style::default().fg(Color::Yellow)),
        Span::raw("Next field | "),
        Span::styled("[Enter] ", Style::default().fg(Color::Green)),
        Span::raw("Save Clip | "),
        Span::styled("[Esc] ", Style::default().fg(Color::Red)),
        Span::raw("Cancel"),
    ]);
    let p_hint = Paragraph::new(hint_text).alignment(Alignment::Center);

    f.render_widget(p_url, chunks[0]);
    f.render_widget(p_tags, chunks[1]);
    f.render_widget(p_notes, chunks[2]);
    f.render_widget(p_hint, chunks[3]);
}

fn draw_edit_tags_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Edit Tags (Comma Separated) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let hint = Line::from(vec![
        Span::styled("Tags: ", Style::default().fg(Color::Yellow)),
        Span::raw(&app.tag_input),
    ]);

    let text = vec![
        hint,
        Line::from(""),
        Line::from(Span::styled("Press Enter to save tags, Esc to cancel.", Style::default().fg(Color::DarkGray))),
    ];

    let p = Paragraph::new(text).block(block);
    f.render_widget(p, area);
}

fn draw_export_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(55, 35, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Export Clips ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let fmt_str = format!("Format: {} (Press Tab to toggle json/csv/html)", app.export_format);
    let path_str = format!("Output Path: {}", app.export_path_input);

    let text = vec![
        Line::from(Span::styled(fmt_str, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled(path_str, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("Press Enter to export, Esc to cancel.", Style::default().fg(Color::Green))),
    ];

    let p = Paragraph::new(text).block(block);
    f.render_widget(p, area);
}

fn draw_confirm_delete_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(45, 25, f.area());
    f.render_widget(Clear, area);

    let title_clip = app.selected_clip().map(|c| c.title.as_str()).unwrap_or("clip");

    let block = Block::default()
        .title(" Confirm Deletion ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(Span::styled(
            format!("Delete clip '{}'?", title_clip),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default()),
            Span::styled("y / Enter ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("to confirm, ", Style::default()),
            Span::styled("n / Esc ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("to cancel.", Style::default()),
        ]),
    ];

    let p = Paragraph::new(text).block(block).alignment(Alignment::Center);
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
