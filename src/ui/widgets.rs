use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::db::models::Command;
use crate::ui::app::{Language, Mode};

pub struct Colors;

impl Colors {
    pub const PRIMARY: Color = Color::Rgb(0, 200, 255);
    pub const PRIMARY_DARK: Color = Color::Rgb(0, 120, 180);
    pub const ACCENT: Color = Color::Rgb(255, 105, 180);
    pub const ACCENT_ALT: Color = Color::Rgb(180, 60, 220);
    pub const SUCCESS: Color = Color::Rgb(80, 220, 120);
    pub const WARNING: Color = Color::Rgb(255, 200, 50);
    pub const ERROR: Color = Color::Rgb(255, 80, 80);
    pub const TEXT_PRIMARY: Color = Color::Rgb(240, 245, 250);
    pub const TEXT_SECONDARY: Color = Color::Rgb(140, 160, 180);
    pub const TEXT_DIM: Color = Color::Rgb(90, 110, 130);
    pub const BG_SELECTED: Color = Color::Rgb(255, 200, 50);
    pub const BG_BORDER: Color = Color::Rgb(0, 150, 200);
}

/// Language toggle indicator for the header.
pub fn render_language_indicator(language: Language) -> String {
    match language {
        Language::English => "[EN]/AR".to_string(),
        Language::Arabic => "EN/[AR]".to_string(),
    }
}

/// Search box paragraph for the header.
pub fn render_search_box<'a>(
    query: &'a str,
    mode: &Mode,
    language: Language,
) -> Paragraph<'a> {
    let title = if *mode == Mode::Search {
        Span::styled(" 🔍 ", Style::default().fg(Color::Cyan))
    } else {
        Span::styled(" 🔍 ", Style::default())
    };

    let empty_hint = match language {
        Language::English => "Press / to search",
        Language::Arabic => "اضغط / للبحث",
    };

    let text_span = if *mode == Mode::Search {
        if query.is_empty() {
            Span::styled("▌", Style::default().fg(Color::White))
        } else {
            Span::raw(format!("{}▌", query))
        }
    } else {
        if query.is_empty() {
            Span::styled(empty_hint, Style::default().fg(Color::Gray))
        } else {
            Span::raw(query)
        }
    };

    let border_color = if *mode == Mode::Search {
        Colors::PRIMARY
    } else {
        Colors::TEXT_DIM
    };

    Paragraph::new(Line::from(vec![title, text_span]))
        .block(
            Block::bordered()
                .border_style(Style::default().fg(border_color)),
        )
}

/// One command block: two lines + a separator line.
/// Line A: marker + name (bold) + favorite star
/// Line B: indented command string (dim)
pub fn render_command_block(cmd: &Command, is_selected: bool) -> Vec<Line<'_>> {
    let marker = if is_selected { "▶ " } else { "  " };
    let mut line_a = Line::from(vec![
        Span::raw(marker),
        Span::styled(
            cmd.name_en.clone(),
            Style::default()
                .fg(Colors::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    if cmd.favorite {
        line_a.push_span(Span::raw(" ⭐"));
    }

    let line_b = Line::from(Span::styled(
        format!("    {}", cmd.command),
        Style::default().fg(Colors::TEXT_DIM),
    ));

    let mut lines = vec![line_a, line_b, Line::default()];

    if is_selected {
        let sel_bg = Color::Rgb(60, 50, 20);
        let sel = Style::default().bg(sel_bg).fg(Colors::WARNING);
        for line in lines.iter_mut().take(2) {
            for span in &mut line.spans {
                span.style = span.style.patch(sel);
            }
            line.spans.insert(
                0,
                Span::styled(
                    "▐",
                    Style::default().fg(Colors::PRIMARY).bg(sel_bg),
                ),
            );
        }
    }

    lines
}

/// Content of the details panel for the selected command.
pub fn render_detail_panel<'a>(cmd: Option<&'a Command>, language: Language) -> Paragraph<'a> {
    let mut lines: Vec<Line> = vec![];

    if let Some(c) = cmd {
        // English name and description only (Arabic fields are ignored).
        let opposite_name = match language {
            Language::English => c.name_en.as_str(),
            Language::Arabic => c.name_en.as_str(),
        };
        let opposite_desc = match language {
            Language::English => c.description_en.as_deref().unwrap_or(""),
            Language::Arabic => c.description_en.as_deref().unwrap_or(""),
        };

        lines.push(Line::from(Span::styled(
            opposite_name,
            Style::default()
                .fg(Colors::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            opposite_desc,
            Style::default().fg(Colors::TEXT_SECONDARY),
        )));

        if c.has_placeholders() {
            let names = c
                .placeholders
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(Line::from(Span::styled(
                format!("⚠️  Requires: {}", names),
                Style::default().fg(Colors::WARNING),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "✅ No inputs required",
                Style::default().fg(Colors::SUCCESS),
            )));
        }
    } else {
        lines.push(Line::from("—"));
    }

    Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(Line::from(Span::styled(
                    "📋 Details",
                    Style::default()
                        .fg(Colors::ACCENT)
                        .add_modifier(Modifier::BOLD),
                )))
                .border_style(Style::default().fg(Colors::ACCENT)),
        )
        .style(Style::default())
}

/// Bottom status bar with app version, command count, and current time.
pub fn render_status_bar(command_count: usize) -> Paragraph<'static> {
    let time = chrono::Local::now().format("%H:%M");
    let text = format!("  EASY v0.1.0  │  Commands: {}  │  {}  ", command_count, time);
    Paragraph::new(text).style(
        Style::default()
            .bg(Colors::PRIMARY_DARK)
            .fg(Colors::TEXT_PRIMARY),
    )
}

/// Bottom navigation bar with 5 fixed buttons.
pub fn render_nav_bar() -> Paragraph<'static> {
    let label = || Style::default().fg(Colors::TEXT_PRIMARY);
    let key = |bg: Color| Style::default().bg(bg).add_modifier(Modifier::BOLD);
    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled("[S]", key(Colors::PRIMARY)),
        Span::styled(" SEARCH", label()),
        Span::raw("   "),
        Span::styled("[C]", key(Colors::SUCCESS)),
        Span::styled(" CONFIG", label()),
        Span::raw("   "),
        Span::styled("[H]", key(Colors::WARNING)),
        Span::styled(" HELP", label()),
        Span::raw("   "),
        Span::styled("[F]", key(Colors::ACCENT)),
        Span::styled(" FAVORITES", label()),
        Span::raw("   "),
        Span::styled("[R]", key(Colors::PRIMARY_DARK)),
        Span::styled(" RUN", label()),
        Span::raw("  "),
    ]);
    Paragraph::new(line)
}

/// Confirmation dialog shown before running a command.
pub fn render_confirm_dialog(cmd: &Command) -> Paragraph<'static> {
    let lines = vec![
        Line::default(),
        Line::from(Span::styled(
            "  \u{26a0}\u{fe0f}  Confirm Execution",
            Style::default()
                .fg(Colors::WARNING)
                .add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from(Span::styled(
            "  Command:",
            Style::default().fg(Colors::TEXT_SECONDARY),
        )),
        Line::from(Span::styled(
            format!("    {}", cmd.command),
            Style::default()
                .fg(Colors::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[Enter]",
                Style::default()
                    .fg(Colors::SUCCESS)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Run    "),
            Span::styled(
                "[Esc]",
                Style::default()
                    .fg(Colors::ERROR)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ]),
        Line::default(),
    ];

    Paragraph::new(lines).block(
        Block::bordered()
            .border_style(Style::default().fg(Colors::WARNING))
            .title(Line::from(Span::styled(
                " \u{26a0}\u{fe0f}  Confirmation ",
                Style::default()
                    .fg(Colors::WARNING)
                    .add_modifier(Modifier::BOLD),
            ))),
    )
}

/// Input dialog shown before running a command that has placeholders.
pub fn render_input_dialog(
    cmd: &Command,
    values: &[String],
    active_field: usize,
    field_cursor: usize,
) -> Paragraph<'static> {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  \u{1f4dd} Input Required",
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  Command: ",
        Style::default().fg(Colors::TEXT_SECONDARY),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {}", cmd.command),
        Style::default().fg(Colors::TEXT_PRIMARY),
    )));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  Fill in the placeholders below:",
        Style::default().fg(Colors::TEXT_SECONDARY),
    )));
    lines.push(Line::default());

    for (i, ph) in cmd.placeholders.iter().enumerate() {
        let value = values.get(i).cloned().unwrap_or_default();
        let is_active = i == active_field;

        let mut spans: Vec<Span> = Vec::new();
        let label_style = if is_active {
            Style::default().fg(Colors::TEXT_PRIMARY).bg(Color::Rgb(40, 60, 80))
        } else {
            Style::default().fg(Colors::TEXT_SECONDARY).bg(Color::Rgb(40, 60, 80))
        };
        spans.push(Span::styled(format!("  <{}>: ", ph.name), label_style));

        if is_active {
            // Value with an inserted cursor at the field cursor position.
            let pos = field_cursor.min(value.len());
            let (before, after) = value.split_at(pos);
            spans.push(Span::styled(
                before.to_string(),
                Style::default().fg(Colors::TEXT_PRIMARY).bg(Color::Rgb(40, 60, 80)),
            ));
            spans.push(Span::styled(
                "\u{258c}".to_string(),
                Style::default().fg(Colors::PRIMARY).bg(Color::Rgb(40, 60, 80)),
            ));
            spans.push(Span::styled(
                after.to_string(),
                Style::default().fg(Colors::TEXT_PRIMARY).bg(Color::Rgb(40, 60, 80)),
            ));
        } else {
            spans.push(Span::styled(
                if value.is_empty() { "(empty)".to_string() } else { value },
                Style::default().fg(Colors::TEXT_SECONDARY).bg(Color::Rgb(40, 60, 80)),
            ));
        }

        if !ph.description.is_empty() {
            spans.push(Span::styled(
                format!("  \u{2014}  {}", ph.description),
                Style::default()
                    .fg(Colors::TEXT_DIM)
                    .add_modifier(Modifier::DIM),
            ));
        }

        lines.push(Line::from(spans));
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled(
            "[Tab]",
            Style::default()
                .fg(Colors::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Next  "),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(Colors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Run  "),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(Colors::ERROR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Cancel"),
    ]));

    Paragraph::new(lines).block(
        Block::bordered()
            .border_style(Style::default().fg(Colors::PRIMARY))
            .title(Line::from(Span::styled(
                " \u{1f4dd} Input ",
                Style::default()
                    .fg(Colors::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ))),
    )
}

/// Output dialog showing the result of an executed command.
pub fn render_output_dialog(
    result: &crate::executor::ExecResult,
    scroll: u16,
) -> Paragraph<'static> {
    let mut lines: Vec<Line> = Vec::new();

    // Header: $ <command>
    lines.push(Line::from(Span::styled(
        format!("$ {}", result.command),
        Style::default()
            .fg(Colors::TEXT_PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));

    // Exit-code line.
    let code = result
        .exit_code
        .map(|c| c.to_string())
        .unwrap_or_else(|| "?".to_string());
    let exit_line = if result.success {
        Span::styled(
            format!("\u{2713} Exit code: {} ({}ms)", code, result.duration_ms),
            Style::default()
                .fg(Colors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!("\u{2717} Exit code: {} ({}ms)", code, result.duration_ms),
            Style::default()
                .fg(Colors::ERROR)
                .add_modifier(Modifier::BOLD),
        )
    };
    lines.push(Line::from(exit_line));

    // Separator.
    lines.push(Line::from(Span::styled(
        "\u{2500}".repeat(50),
        Style::default().fg(Colors::TEXT_DIM),
    )));

    // stdout lines.
    if result.stdout.trim().is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no output)",
            Style::default().fg(Colors::TEXT_DIM),
        )));
    } else {
        for line in result.stdout.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(Colors::TEXT_PRIMARY),
            )));
        }
    }

    // stderr section.
    if !result.stderr.trim().is_empty() {
        lines.push(Line::from(Span::styled(
            "\u{2500}".repeat(50),
            Style::default().fg(Colors::TEXT_DIM),
        )));
        lines.push(Line::from(Span::styled(
            "STDERR:",
            Style::default()
                .fg(Colors::ERROR)
                .add_modifier(Modifier::BOLD),
        )));
        for line in result.stderr.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(Colors::ERROR),
            )));
        }
    }

    // Footer hint.
    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(Colors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Close", Style::default().fg(Colors::TEXT_SECONDARY)),
        Span::styled("  \u{2022}  ", Style::default().fg(Colors::TEXT_DIM)),
        Span::styled(
            "[\u{2191}\u{2193}]",
            Style::default()
                .fg(Colors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Scroll", Style::default().fg(Colors::TEXT_SECONDARY)),
    ]));

    Paragraph::new(lines).scroll((scroll, 0))
}

/// Add/Edit form dialog.
pub fn render_add_form(
    form: &crate::ui::app::AddForm,
    mode: &crate::ui::app::Mode,
) -> Paragraph<'static> {
    let is_edit = *mode == crate::ui::app::Mode::Edit;

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        if is_edit { "  \u{270f}\u{fe0f} Edit Command" } else { "  \u{2795} Add Command" },
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::default());

    let field_values: Vec<&str> = vec![
        &form.name_en,
        &form.command,
        &form.tool,
        &form.description_en,
        &form.category,
        &form.tags,
    ];
    let labels: Vec<&str> = vec![
        "Name (EN):",
        "Command:",
        "Tool:",
        "Description:",
        "Category:",
        "Tags (comma-sep):",
    ];
    let max_label = labels.iter().map(|l| l.len()).max().unwrap_or(0);

    for (i, label) in labels.iter().enumerate() {
        let active = i == form.field_cursor;
        let value = field_values[i];
        let mut spans: Vec<Span> = vec![Span::styled(
            format!("  {:<width$} ", label, width = max_label),
            Style::default().fg(Colors::TEXT_DIM),
        )];
        if active {
            let pos = form.char_cursor.min(value.len());
            let (before, after) = value.split_at(pos);
            let active_style = Style::default()
                .fg(Colors::TEXT_PRIMARY)
                .bg(Color::Rgb(40, 60, 80));
            spans.push(Span::styled(before.to_string(), active_style));
            spans.push(Span::styled(
                "\u{258c}".to_string(),
                Style::default().fg(Colors::PRIMARY).bg(Color::Rgb(40, 60, 80)),
            ));
            spans.push(Span::styled(after.to_string(), active_style));
        } else {
            spans.push(Span::styled(
                value.to_string(),
                Style::default().fg(Colors::TEXT_SECONDARY),
            ));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "[Tab]",
            Style::default()
                .fg(Colors::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Next   "),
        Span::styled(
            "[Ctrl+S]",
            Style::default()
                .fg(Colors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Save   "),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(Colors::ERROR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Cancel"),
    ]));

    Paragraph::new(lines).block(
        Block::bordered()
            .border_style(Style::default().fg(Colors::PRIMARY))
            .title(Line::from(Span::styled(
                if is_edit { " \u{270f}\u{fe0f} Form " } else { " \u{2795} Form " },
                Style::default()
                    .fg(Colors::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ))),
    )
}

/// Scan dialog showing auto-discovery results for the default tools.
pub fn render_scan_dialog(
    results: &[crate::scanner::ScanResult],
    selected: usize,
    total: usize,
    _scroll: u16,
) -> Paragraph<'static> {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  \u{1f50d} Auto-Discovery Scan",
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        format!("  Total new commands found: {}", total),
        Style::default().fg(Colors::TEXT_PRIMARY),
    )));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  \u{2500}\u{2500} Tools \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
        Style::default().fg(Colors::TEXT_DIM),
    )));

    for (i, result) in results.iter().enumerate() {
        let is_selected = i == selected;
        if let Some(err) = &result.error {
            let style = if is_selected {
                Style::default()
                    .fg(Colors::TEXT_PRIMARY)
                    .bg(Color::Rgb(40, 60, 80))
            } else {
                Style::default().fg(Colors::ERROR)
            };
            lines.push(Line::from(Span::styled(
                format!("  \u{2717} {}  [error: {}]", result.tool, err),
                style,
            )));
        } else {
            let prefix = if is_selected { "\u{25b6} " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(Colors::TEXT_PRIMARY)
                    .bg(Color::Rgb(40, 60, 80))
            } else {
                Style::default().fg(Colors::TEXT_SECONDARY)
            };
            lines.push(Line::from(Span::styled(
                format!("  {}\u{2713} {}  \u{2192} {} commands", prefix, result.tool, result.commands.len()),
                style,
            )));
        }
    }

    lines.push(Line::default());

    if let Some(result) = results.get(selected) {
        lines.push(Line::from(Span::styled(
            format!("  Commands from {}:", result.tool),
            Style::default().fg(Colors::TEXT_DIM),
        )));
        for cmd in result.commands.iter().take(8) {
            lines.push(Line::from(Span::styled(
                format!("    \u{2022} {}  \u{2014}  {}", cmd.name_en, cmd.command),
                Style::default().fg(Colors::TEXT_SECONDARY),
            )));
        }
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "[\u{2191}\u{2193}]",
            Style::default()
                .fg(Colors::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Browse tools    "),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(Colors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Add all    "),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(Colors::ERROR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Cancel"),
    ]));

    Paragraph::new(lines).block(
        Block::bordered()
            .border_style(Style::default().fg(Colors::PRIMARY))
            .title(Line::from(Span::styled(
                " \u{1f50d} Scan ",
                Style::default()
                    .fg(Colors::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ))),
    )
}

/// AI Search dialog showing natural-language query results.
pub fn render_ai_search(
    query: &str,
    suggestions: &[crate::ai::Suggestion],
    selected: usize,
    loading: bool,
    error: Option<&str>,
    commands: &[crate::db::models::Command],
    spinner: &str,
) -> Paragraph<'static> {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        "  \u{1f916} AI Search",
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        format!("  Query: {}\u{258c}", query),
        Style::default().fg(Colors::TEXT_PRIMARY),
    )));
    lines.push(Line::default());

    if loading {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                spinner.to_string(),
                Style::default().fg(Colors::WARNING),
            ),
            Span::raw("  Asking AI... please wait"),
        ]));
    } else if let Some(err) = error {
        lines.push(Line::from(Span::styled(
            format!("  \u{274c} Error: {}", err),
            Style::default().fg(Colors::ERROR),
        )));
    } else if suggestions.is_empty() {
        lines.push(Line::from(Span::styled(
            "  Type a question and press Enter",
            Style::default().fg(Colors::TEXT_DIM),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "  \u{2500}\u{2500} Suggestions \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            Style::default().fg(Colors::TEXT_DIM),
        )));
        for (i, s) in suggestions.iter().enumerate() {
            let name = commands
                .iter()
                .find(|c| c.id == s.command_id)
                .map(|c| c.name_en.as_str())
                .unwrap_or("??");
            let pct = (s.score * 100.0).round() as i32;
            if i == selected {
                lines.push(Line::from(Span::styled(
                    format!("  \u{25b6} {}. {}  ({}%)", i + 1, name, pct),
                    Style::default()
                        .fg(Colors::TEXT_PRIMARY)
                        .bg(Color::Rgb(40, 60, 80)),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    format!("      {}. {}  ({}%)", i + 1, name, pct),
                    Style::default().fg(Colors::TEXT_SECONDARY),
                )));
            }
            lines.push(Line::from(Span::styled(
                format!("     {}", s.reason),
                Style::default().fg(Colors::TEXT_DIM),
            )));
        }
    }

    lines.push(Line::default());
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "[Tab]",
            Style::default()
                .fg(Colors::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Fuzzy   "),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(Colors::SUCCESS)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Run   "),
        Span::styled(
            "[Esc]",
            Style::default()
                .fg(Colors::ERROR)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Cancel"),
    ]));

    Paragraph::new(lines).block(
        Block::bordered()
            .border_style(Style::default().fg(Colors::PRIMARY))
            .title(Line::from(Span::styled(
                " \u{1f916} AI ",
                Style::default()
                    .fg(Colors::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ))),
    )
}

/// Startup logo (EASY ASCII art, one color per letter) plus centered tagline.
pub fn render_logo() -> Vec<Line<'static>> {
    // Each letter is a separate column-aligned set of glyphs,
    // padded to 8 chars wide, joined with 2 spaces.
    let e_lines = [
        "███████",
        "██     ",
        "█████  ",
        "██     ",
        "███████",
    ];
    let a_lines = [
        " █████ ",
        "██   ██",
        "███████",
        "██   ██",
        "██   ██",
    ];
    let s_lines = [
        "███████",
        "██     ",
        "███████",
        "     ██",
        "███████",
    ];
    let y_lines = [
        "██   ██",
        " ██ ██ ",
        "  ███  ",
        "   █   ",
        "   █   ",
    ];

    let mut lines: Vec<Line<'static>> = Vec::new();
    for row in 0..5 {
        lines.push(Line::from(vec![
            Span::styled(format!("{:<8}", e_lines[row]), Style::default().fg(Colors::PRIMARY)),
            Span::raw("  ".to_string()),
            Span::styled(format!("{:<8}", a_lines[row]), Style::default().fg(Colors::SUCCESS)),
            Span::raw("  ".to_string()),
            Span::styled(format!("{:<8}", s_lines[row]), Style::default().fg(Colors::WARNING)),
            Span::raw("  ".to_string()),
            Span::styled(format!("{:<8}", y_lines[row]), Style::default().fg(Colors::ACCENT)),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "COMMAND HUB FOR TERMUX",
        Style::default().fg(Colors::TEXT_DIM),
    )));
    lines
}

fn help_row(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {:<18}", key),
            Style::default().fg(Colors::TEXT_PRIMARY),
        ),
        Span::raw(desc.to_string()),
    ])
}

/// Help overlay showing all keybindings.
pub fn render_help_screen() -> Paragraph<'static> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(Line::from(Span::styled(
        "\u{2753} Help / \u{0645}\u{0633}\u{0627}\u{0639}\u{062f}\u{0629}",
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::default());

    // Section 1: Navigation
    lines.push(Line::from(Span::styled(
        "Navigation",
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(help_row("↑ / ↓ / j / k", "Navigate list"));
    lines.push(help_row("g / G", "Go to first / last"));
    lines.push(help_row("PgUp / PgDn", "Scroll"));
    lines.push(Line::default());

    // Section 2: Search
    lines.push(Line::from(Span::styled(
        "Search",
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(help_row("/", "Enter Fuzzy search"));
    lines.push(help_row("Tab", "Toggle to AI search (from Search mode)"));
    lines.push(help_row("Enter", "Submit query (in AI mode)"));
    lines.push(help_row("Esc", "Exit search / cancel"));
    lines.push(Line::default());

    // Section 3: Actions
    lines.push(Line::from(Span::styled(
        "Actions",
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(help_row("Enter", "Run selected command"));
    lines.push(help_row("F", "Toggle favorite"));
    lines.push(help_row("A", "Add new command"));
    lines.push(help_row("E", "Edit selected"));
    lines.push(help_row("D", "Delete selected (custom only)"));
    lines.push(help_row("S", "Auto-discovery scan"));
    lines.push(Line::default());

    // Section 4: General
    lines.push(Line::from(Span::styled(
        "General",
        Style::default()
            .fg(Colors::PRIMARY)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(help_row("H", "Toggle this help"));
    lines.push(help_row("Q / Ctrl+C", "Quit"));
    lines.push(Line::default());

    lines.push(Line::from(Span::styled(
        "[Esc] Close",
        Style::default().fg(Colors::TEXT_SECONDARY),
    )));

    Paragraph::new(lines).block(
        Block::bordered()
            .border_style(Style::default().fg(Colors::PRIMARY))
            .title(Line::from(Span::styled(
                " \u{2753} Help ",
                Style::default()
                    .fg(Colors::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ))),
    )
}
