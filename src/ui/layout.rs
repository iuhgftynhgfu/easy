use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::app::{App, Mode};
use crate::ui::widgets;

/// Draw the full UI for one frame, adapting which zones are shown to the
/// terminal height.
pub fn draw_ui(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let height = area.height;

    // Configure the app based on terminal size before building constraints.
    app.configure_for_terminal(height);

    // Build layout constraints dynamically.
    let mut constraints: Vec<Constraint> = Vec::new();

    // Status bar (always, 1 line)
    constraints.push(Constraint::Length(1));

    // Nav bar (only if height >= 30)
    if height >= 30 {
        constraints.push(Constraint::Length(1));
    }

    // Logo (only if app.show_logo, 9 lines)
    if app.show_logo {
        constraints.push(Constraint::Length(9));
    }

    // Search box (always, 3 lines)
    constraints.push(Constraint::Length(3));

    // Commands list (flexible, min 6)
    constraints.push(Constraint::Min(6));

    // Details panel (only if height >= 30, 6 lines)
    if height >= 30 {
        constraints.push(Constraint::Length(6));
    }

    // Footer (always, 1 line)
    constraints.push(Constraint::Length(1));

    let chunks = Layout::vertical(constraints).split(area);

    let mut idx = 0;

    // Status bar
    let status = widgets::render_status_bar(app.commands.len());
    frame.render_widget(status, chunks[idx]);
    idx += 1;

    // Nav bar
    if height >= 30 {
        let nav = widgets::render_nav_bar();
        frame.render_widget(nav, chunks[idx]);
        idx += 1;
    }

    // Logo
    if app.show_logo {
        let logo = widgets::render_logo();
        let logo_paragraph = Paragraph::new(logo);
        frame.render_widget(logo_paragraph, chunks[idx]);
        idx += 1;
    }

    // Search box
    let search = widgets::render_search_box(&app.search_query, &app.mode, app.language);
    frame.render_widget(search, chunks[idx]);
    idx += 1;

    // Commands list
    let list_area = chunks[idx];
    idx += 1;
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(widgets::Colors::BG_BORDER))
        .title(format!(" Commands ({}) ", app.filtered.len()))
        .title_style(
            Style::default()
                .fg(widgets::Colors::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );
    let inner = list_block.inner(list_area);
    frame.render_widget(list_block, list_area);

    if app.filtered.is_empty() {
        let empty = Paragraph::new(app.t("No results", "لا توجد نتائج"))
            .style(Style::default().fg(widgets::Colors::TEXT_DIM));
        frame.render_widget(empty, inner);
    } else {
        // Build lines for all filtered commands.
        let mut all_lines: Vec<Line> = Vec::new();
        let mut selected_line_start = 0;
        let mut selected_line_count = 0;

        for (display_idx, &cmd_idx) in app.filtered.iter().enumerate() {
            let cmd = &app.commands[cmd_idx];
            let is_selected = display_idx == app.selected_index;
            if is_selected {
                selected_line_start = all_lines.len();
                selected_line_count = 3; // name + command + blank
            }
            all_lines.extend(widgets::render_command_block(cmd, is_selected));
            // Blank line between commands.
            all_lines.push(Line::from(""));
        }

        // If there's room, show a dim hint at the bottom of the list.
        if (all_lines.len() as u16) < inner.height {
            all_lines.push(Line::from(Span::styled(
                format!(
                    "  \u{00b7}  {} commands available  \u{00b7}  Press / to search  \u{00b7}  \u{2191}\u{2193} to navigate",
                    app.filtered.len()
                ),
                Style::default().fg(widgets::Colors::TEXT_DIM),
            )));
        }

        // Scroll offset to keep the selection visible.
        let inner_height = inner.height as usize;
        let scroll_offset = if selected_line_start + selected_line_count > inner_height {
            (selected_line_start + selected_line_count).saturating_sub(inner_height)
        } else {
            0
        };

        let list_paragraph = Paragraph::new(all_lines).scroll((scroll_offset as u16, 0));
        frame.render_widget(list_paragraph, inner);
    }

    // Details panel
    if height >= 30 {
        let details = widgets::render_detail_panel(app.selected_command(), app.language);
        frame.render_widget(details, chunks[idx]);
        idx += 1;
    }

    // Output dialog (drawn on top of everything).
    if app.mode == Mode::Output {
        if let Some(result) = &app.output_result {
            let popup_area = centered_rect(80, 70, frame.area());
            frame.render_widget(Clear, popup_area);
            let content = widgets::render_output_dialog(result, app.output_scroll);
            let dialog = content.block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(widgets::Colors::PRIMARY))
                    .title(" \u{1f4e4} Output ")
                    .title_style(
                        Style::default()
                            .fg(widgets::Colors::PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ),
            );
            frame.render_widget(dialog, popup_area);
        }
    }

    // AI Search dialog (drawn on top of everything).
    if app.mode == Mode::AiSearch {
        let popup_area = centered_rect(80, 60, frame.area());
        frame.render_widget(Clear, popup_area);
        let dialog = widgets::render_ai_search(
            &app.ai_query,
            &app.ai_suggestions,
            app.ai_selected,
            app.ai_loading,
            app.ai_error.as_deref(),
            &app.commands,
            app.spinner_char(),
        );
        frame.render_widget(dialog, popup_area);
    }

    // Help overlay (drawn on top of everything).
    if app.mode == Mode::Help {
        let popup_area = centered_rect(70, 75, frame.area());
        frame.render_widget(Clear, popup_area);
        let dialog = widgets::render_help_screen();
        frame.render_widget(dialog, popup_area);
    }

    // Scan dialog (drawn on top of everything).
    if app.mode == Mode::Scan {
        let popup_area = centered_rect(80, 70, frame.area());
        frame.render_widget(Clear, popup_area);
        let dialog = widgets::render_scan_dialog(
            &app.scan_results,
            app.scan_selected,
            app.scan_total_commands,
            app.scan_scroll,
        );
        let dialog = dialog.scroll((app.scan_scroll, 0));
        frame.render_widget(dialog, popup_area);
    }

    // Add/Edit form dialog (drawn on top of everything).
    if app.mode == Mode::Add || app.mode == Mode::Edit {
        let popup_area = centered_rect(75, 60, frame.area());
        frame.render_widget(Clear, popup_area);
        let dialog = widgets::render_add_form(&app.add_form, &app.mode);
        frame.render_widget(dialog, popup_area);
    }

    // Input dialog (drawn on top of everything).
    if app.mode == Mode::Input {
        if let Some(cmd) = app.pending_command_ref() {
            let popup_area = centered_rect(70, 50, frame.area());
            frame.render_widget(Clear, popup_area);
            let dialog =
                widgets::render_input_dialog(cmd, &app.input_values, app.input_cursor, app.input_field_cursor);
            frame.render_widget(dialog, popup_area);
        }
    }

    // Confirmation dialog (drawn on top of everything).
    if app.mode == Mode::Confirm {
        if let Some(cmd) = app.pending_command_ref() {
            let popup_area = centered_rect(60, 40, frame.area());
            frame.render_widget(Clear, popup_area);
            let dialog = widgets::render_confirm_dialog(cmd);
            frame.render_widget(dialog, popup_area);
        }
    }

    // Footer
    let footer = if let Some(msg) = &app.status_message {
        Paragraph::new(Line::from(Span::styled(
            msg.as_str(),
            Style::default().fg(widgets::Colors::WARNING),
        )))
    } else {
        Paragraph::new(Line::from(app.t(
            "↑↓ Navigate • Enter Run • F Favorite • L Language • / Search • Esc Exit",
            "↑↓ تنقل • Enter تشغيل • F مفضل • L اللغة • / بحث • Esc خروج",
        )))
    };
    frame.render_widget(footer, chunks[idx]);
}

/// Center a popup rectangle (percentage of width/height) within `r`.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let vertical = Layout::default()
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
        .split(vertical[1])[1]
}
