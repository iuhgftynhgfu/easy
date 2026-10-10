mod ai;
mod db;
mod executor;
mod scanner;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{poll, read};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use db::storage::{load_commands, save_commands};
use ui::app::App;
use ui::events::handle_event;
use ui::layout::draw_ui;

/// Owns the terminal; restores it on drop even if an error unwinds.
struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn init_tui() -> Result<Tui> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    Ok(Tui { terminal })
}

fn run(mut tui: Tui) -> Result<()> {
    let mut app = App::new(load_commands()?);

    loop {
        tui.terminal.draw(|f| draw_ui(f, &mut app))?;

        app.tick_spinner();

        if poll(Duration::from_millis(100))? {
            let event = read()?;
            handle_event(&mut app, event)?;
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Handle --help / -h
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    // Handle --version / -v
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!("easy {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Ensure the DB has seed data.
    if load_commands()?.is_empty() {
        let seeded = db::seed::seed_commands();
        save_commands(&seeded)?;
    }

    // Merge builtin commands on startup (idempotent — does not duplicate)
    if let Ok(n) = db::storage::merge_builtin_commands() {
        if n > 0 {
            eprintln!("[easy] Added {} builtin commands", n);
        }
    }

    run(init_tui()?)
}

fn print_help() {
    println!("easy {} - A unified TUI command hub for Termux", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("    easy [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print this help message and exit");
    println!("    -v, --version    Print version and exit");
    println!();
    println!("KEYBINDINGS (inside the TUI):");
    println!("    /                Search commands (Fuzzy)");
    println!("    Tab              Switch to AI Search (from Search mode)");
    println!("    ↑ / ↓ or j / k   Navigate list");
    println!("    Enter            Run selected command / Confirm");
    println!("    Esc              Cancel / Exit current dialog");
    println!("    F                Toggle favorite");
    println!("    A                Add new command");
    println!("    E                Edit selected command");
    println!("    D                Delete selected command (custom only)");
    println!("    S                Auto-discovery scan");
    println!("    Q or Ctrl+C      Quit");
    println!();
    println!("CONFIG:");
    println!("    ~/.config/easy/commands.json    Commands database");
    println!();
    println!("For more info: https://github.com/AhmedAdel/easy");
}
