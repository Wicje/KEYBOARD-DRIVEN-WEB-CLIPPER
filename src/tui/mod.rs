pub mod app;
pub mod event;
pub mod ui;

use crate::db::Database;
use anyhow::Result;
use app::App;
use crossterm::{
    event::Event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::{handle_key_event, EventResult};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;
use ui::draw_ui;

pub fn run_tui(db: &Database) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(db)?;

    let res = run_loop(&mut terminal, &mut app);

    // Restore terminal settings
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    match handle_key_event(app, key)? {
                        EventResult::Exit => break,
                        EventResult::Continue => {}
                    }
                }
            }
        }
    }
    Ok(())
}
