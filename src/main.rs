mod app;
mod cli;
mod clipboard;
mod components;
mod config;
mod database;
mod event;
mod key_bind;
mod tree;
mod ui;
mod version;

#[macro_use]
mod log;

use crate::app::App;
use crate::config::Config;
use crate::event::{Event, Key};
use anyhow::Result;
use crossterm::execute;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use std::panic::{set_hook, take_hook};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let value = crate::cli::parse();
    let config = Config::new(&value.config)?;
    setup_terminal()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let events = event::Events::new(250);
    let mut app = App::new(config.clone());
    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            if let Err(err) = app.draw(f) {
                shutdown_terminal();
                let mut source = err.source();
                while let Some(err) = source {
                    eprintln!("Caused by: {}", err);
                    source = err.source();
                }
                eprintln!("Failed by: {}", err);

                std::process::exit(1);
            }
        })?;
        match events.next()? {
            Event::Input(key) => match app.event(key).await {
                Ok(state) => {
                    if !state.is_consumed()
                        && (key == app.config.key_config.quit || key == app.config.key_config.exit)
                    {
                        break;
                    }
                }
                Err(err) => app.error.set(err.to_string())?,
            },
            Event::Tick => (),
        }
    }

    shutdown_terminal();
    terminal.show_cursor()?;
    Ok(())
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    init_panic_hook();
    io::stdout().execute(EnterAlternateScreen)?;
    Ok(())
}

pub fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

pub fn restore_tui() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn shutdown_terminal() {
    let leave_screen = io::stdout().execute(LeaveAlternateScreen).map(|_f| ());

    if let Err(e) = leave_screen {
        eprintln!("leave_screen failed:\n{}", e);
    }

    let leave_raw_mode = disable_raw_mode();

    if let Err(e) = leave_raw_mode {
        eprintln!("leave_raw_mode failed:\n{}", e);
    }
}
