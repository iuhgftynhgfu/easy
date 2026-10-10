use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::ui::app::{App, Mode};

/// Handle a single keyboard event, mutating app state.
pub fn handle_event(app: &mut App, event: Event) -> Result<()> {
    if let Event::Key(key) = event {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Confirmation mode: Enter runs, Esc cancels, everything else ignored.
        if app.mode == Mode::Confirm {
            match key.code {
                KeyCode::Enter => {
                    app.execute_pending();
                }
                KeyCode::Esc => app.cancel_confirmation(),
                _ => {}
            }
            return Ok(());
        }

        // Input mode: Tab/BackTab move fields, Enter runs, Esc cancels.
        if app.mode == Mode::Input {
            match key.code {
                KeyCode::Enter => app.execute_with_input(),
                KeyCode::Esc => app.cancel_input(),
                KeyCode::Tab => app.next_field(),
                KeyCode::BackTab => app.prev_field(),
                KeyCode::Backspace => app.input_backspace(),
                KeyCode::Char(c) => app.input_char(c),
                _ => {}
            }
            return Ok(());
        }

        // Add/Edit form mode: Tab/BackTab move fields, Enter/Ctrl+S save, Esc cancels.
        if app.mode == Mode::Add || app.mode == Mode::Edit {
            match key.code {
                KeyCode::Esc => app.cancel_form(),
                KeyCode::Tab => app.form_next_field(),
                KeyCode::BackTab => app.form_prev_field(),
                KeyCode::Backspace => app.form_backspace(),
                KeyCode::Enter => {
                    let _ = app.save_form();
                }
                KeyCode::Char('s') if ctrl => {
                    let _ = app.save_form();
                }
                KeyCode::Char(c) => app.form_char(c),
                _ => {}
            }
            return Ok(());
        }

        // Scan mode: arrows browse, Enter adds all, Esc cancels.
        if app.mode == Mode::Scan {
            match key.code {
                KeyCode::Esc => app.cancel_scan(),
                KeyCode::Enter => {
                    let _ = app.scan_add_all();
                }
                KeyCode::Up | KeyCode::Char('k') => app.scan_prev(),
                KeyCode::Down | KeyCode::Char('j') => app.scan_next(),
                KeyCode::PageUp => {
                    app.scan_scroll = app.scan_scroll.saturating_sub(10)
                }
                KeyCode::PageDown => {
                    app.scan_scroll = app.scan_scroll.saturating_add(10)
                }
                _ => {}
            }
            return Ok(());
        }

        // AI Search mode: Tab toggles back to Fuzzy, Enter runs/submits, Esc cancels.
        if app.mode == Mode::AiSearch {
            match key.code {
                KeyCode::Esc => {
                    app.ai_cancel();
                    app.mode = Mode::Normal;
                }
                KeyCode::Tab => {
                    app.toggle_search_mode();
                    app.mode = Mode::Search;
                }
                KeyCode::Enter => {
                    if app.ai_suggestions.is_empty() {
                        app.ai_submit();
                    } else {
                        if let Some(idx) = app.ai_selected_command_index() {
                            app.pending_command = Some(idx);
                            app.ai_cancel();
                            app.mode = Mode::Confirm;
                        }
                    }
                }
                KeyCode::Backspace => app.ai_pop_char(),
                KeyCode::Up | KeyCode::Char('k') => app.ai_prev(),
                KeyCode::Down | KeyCode::Char('j') => app.ai_next(),
                KeyCode::Char(c) => app.ai_push_char(c),
                _ => {}
            }
            return Ok(());
        }

        // Help mode: Esc/q/h close, everything else ignored.
        if app.mode == Mode::Help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => app.close_help(),
                _ => {}
            }
            return Ok(());
        }

        // Output mode: Esc/q close, arrows scroll.
        if app.mode == Mode::Output {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => app.close_output(),
                KeyCode::Up | KeyCode::Char('k') => app.scroll_output(-1),
                KeyCode::Down | KeyCode::Char('j') => app.scroll_output(1),
                KeyCode::PageUp => app.scroll_output(-10),
                KeyCode::PageDown => app.scroll_output(10),
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            // Quit / mode handling
            KeyCode::Esc if app.mode == Mode::Search => {
                app.mode = Mode::Normal;
                app.clear_search();
            }
            KeyCode::Esc => {
                app.should_quit = true;
            }
            KeyCode::Char('q') if app.mode == Mode::Normal => {
                app.should_quit = true;
            }
            // Help overlay
            KeyCode::Char('h') | KeyCode::Char('H') if app.mode == Mode::Normal => {
                app.open_help();
            }

            // Enter search mode (fuzzy by default)
            KeyCode::Char('/') if app.mode == Mode::Normal => {
                app.mode = Mode::Search;
                app.search_mode = super::app::SearchMode::Fuzzy;
            }

            // Tab in search mode: switch to AI search
            KeyCode::Tab if app.mode == Mode::Search => {
                app.toggle_search_mode();
                if app.search_mode == super::app::SearchMode::Ai {
                    app.mode = Mode::AiSearch;
                }
            }
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => app.next(),
            KeyCode::Char('k') | KeyCode::Up => app.previous(),
            // Request confirmation for the selected command,
            // or start the input dialog if it has placeholders.
            KeyCode::Enter => {
                if !app.start_input() {
                    app.request_confirmation();
                }
            }
            // Favorite toggle
            KeyCode::Char('f') | KeyCode::Char('F') => {
                app.toggle_favorite();
            }
            // Add / edit / delete in Normal mode
            KeyCode::Char('a') | KeyCode::Char('A') if app.mode == Mode::Normal => {
                app.start_add();
            }
            KeyCode::Char('e') | KeyCode::Char('E') if app.mode == Mode::Normal => {
                app.start_edit();
            }
            KeyCode::Char('d') | KeyCode::Char('D') if app.mode == Mode::Normal => {
                app.delete_selected();
            }
            // Start an auto-discovery scan (Normal mode only)
            KeyCode::Char('s') | KeyCode::Char('S') if app.mode == Mode::Normal => {
                app.start_scan();
            }
            // Language toggle (Normal mode only)
            KeyCode::Char('l') | KeyCode::Char('L') if app.mode == Mode::Normal => {
                app.toggle_language();
            }
            // Search-mode editing
            KeyCode::Backspace if app.mode == Mode::Search => {
                app.pop_char();
            }
            KeyCode::Char(c) if app.mode == Mode::Search => {
                app.push_char(c);
            }
            // Ctrl+C quits too (raw mode intercepts it)
            KeyCode::Char('c') if ctrl => {
                app.should_quit = true;
            }
            _ => {}
        }
    }
    Ok(())
}
