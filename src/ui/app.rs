use crate::db::models::Command;
use crate::db::storage::save_commands;

/// Current input mode of the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
    Confirm,
    Output,
    Add,
    Edit,
    Help,
    Input,
    Scan,
    AiSearch,
}

/// Which search engine is active.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchMode {
    Fuzzy,
    Ai,
}

/// UI display language (not persisted; always starts as English).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    English,
    Arabic,
}

/// Form state for adding or editing a command.
pub struct AddForm {
    pub name_en: String,
    pub command: String,
    pub tool: String,
    pub description_en: String,
    pub category: String,
    pub tags: String,
    pub field_cursor: usize,
    pub char_cursor: usize,
    pub editing_id: Option<String>,
}

impl Default for AddForm {
    fn default() -> Self {
        Self {
            name_en: String::new(),
            command: String::new(),
            tool: String::new(),
            description_en: String::new(),
            category: String::new(),
            tags: String::new(),
            field_cursor: 0,
            char_cursor: 0,
            editing_id: None,
        }
    }
}

/// Top-level application state.
pub struct App {
    pub commands: Vec<Command>,
    pub filtered: Vec<usize>,
    pub search_query: String,
    pub selected_index: usize,
    pub should_quit: bool,
    pub mode: Mode,
    pub language: Language,
    pub status_message: Option<String>,
    pub show_logo: bool,
    pub pending_command: Option<usize>,
    pub output_result: Option<crate::executor::ExecResult>,
    pub output_scroll: u16,
    pub input_values: Vec<String>,
    pub input_cursor: usize,
    pub input_field_cursor: usize,
    pub add_form: AddForm,
    pub scan_results: Vec<crate::scanner::ScanResult>,
    pub scan_selected: usize,
    pub scan_scroll: u16,
    pub scan_total_commands: usize,
    pub ai_query: String,
    pub ai_suggestions: Vec<crate::ai::Suggestion>,
    pub ai_selected: usize,
    pub ai_loading: bool,
    pub ai_error: Option<String>,
    pub search_mode: SearchMode,
    pub spinner_frame: usize,
    pub spinner_last_tick: std::time::Instant,
}

impl App {
    pub fn new(commands: Vec<Command>) -> Self {
        let filtered: Vec<usize> = (0..commands.len()).collect();
        Self {
            commands,
            filtered,
            search_query: String::new(),
            selected_index: 0,
            should_quit: false,
            mode: Mode::Normal,
            language: Language::English,
            status_message: None,
            show_logo: false,
            pending_command: None,
            output_result: None,
            output_scroll: 0,
            input_values: Vec::new(),
            input_cursor: 0,
            input_field_cursor: 0,
            add_form: AddForm::default(),
            scan_results: Vec::new(),
            scan_selected: 0,
            scan_scroll: 0,
            scan_total_commands: 0,
            ai_query: String::new(),
            ai_suggestions: Vec::new(),
            ai_selected: 0,
            ai_loading: false,
            ai_error: None,
            search_mode: SearchMode::Fuzzy,
            spinner_frame: 0,
            spinner_last_tick: std::time::Instant::now(),
        }
    }

    pub fn open_help(&mut self) {
        self.mode = Mode::Help;
    }

    pub fn close_help(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn tick_spinner(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.spinner_last_tick).as_millis() >= 100 {
            self.spinner_frame = (self.spinner_frame + 1) % 4;
            self.spinner_last_tick = now;
        }
    }

    pub fn spinner_char(&self) -> &'static str {
        match self.spinner_frame {
            0 => "⣾",
            1 => "⣽",
            2 => "⣻",
            _ => "⣷",
        }
    }

    pub fn configure_for_terminal(&mut self, height: u16) {
        self.show_logo = height >= 35;
    }

    /// Re-filter commands based on `search_query`.
    /// Simple substring match against name + command + description + tags.
    pub fn update_filter(&mut self) {
        let q = self.search_query.trim().to_lowercase();
        self.filtered = self
            .commands
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                q.is_empty()
                    || c.name.to_lowercase().contains(&q)
                    || c.command.to_lowercase().contains(&q)
                    || c.description
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&q)
                    || c.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&q))
            })
            .map(|(i, _)| i)
            .collect();
        if self.selected_index >= self.filtered.len() {
            self.selected_index = 0;
        }
    }

    /// Move selection down (wrap around).
    pub fn next(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.filtered.len();
    }

    /// Move selection up (wrap around).
    pub fn previous(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.filtered.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// The currently selected command, if any.
    pub fn selected_command(&self) -> Option<&Command> {
        self.filtered
            .get(self.selected_index)
            .map(|&i| &self.commands[i])
    }

    /// Begin a confirmation flow for the selected command.
    pub fn request_confirmation(&mut self) -> bool {
        if let Some(_) = self.selected_command() {
            if let Some(&cmd_idx) = self.filtered.get(self.selected_index) {
                self.pending_command = Some(cmd_idx);
                self.mode = Mode::Confirm;
                return true;
            }
        }
        false
    }

    /// Cancel the pending confirmation and return to normal mode.
    pub fn cancel_confirmation(&mut self) {
        self.pending_command = None;
        self.mode = Mode::Normal;
    }

    /// Confirm the pending command (for now: just a status message).
    pub fn confirm_pending(&mut self) {
        if let Some(idx) = self.pending_command {
            let cmd = &self.commands[idx];
            self.status_message = Some(format!("Would run: {}", cmd.command));
        }
        self.pending_command = None;
        self.mode = Mode::Normal;
    }

    /// The command awaiting confirmation, if any.
    pub fn pending_command_ref(&self) -> Option<&Command> {
        self.pending_command.map(|idx| &self.commands[idx])
    }

    /// Execute the pending command and show the result in the Output screen.
    pub fn execute_pending(&mut self) {
        if let Some(idx) = self.pending_command {
            let cmd_str = self.commands[idx].command.clone();

            // Use ~/work as the working directory if it exists, otherwise inherit
            // the current directory (None).
            let cwd = {
                let home = std::env::var("HOME").unwrap_or_default();
                let work_path = std::path::PathBuf::from(home).join("work");
                if work_path.exists() {
                    Some(work_path.to_string_lossy().to_string())
                } else {
                    None
                }
            };

            match crate::executor::execute(&cmd_str, cwd.as_deref(), 60) {
                Ok(result) => {
                    self.output_result = Some(result);
                    self.output_scroll = 0;
                    self.mode = Mode::Output;

                    // Update usage stats and persist.
                    self.commands[idx].touch();
                    let _ = crate::db::storage::update_command(self.commands[idx].clone());
                }
                Err(e) => {
                    self.status_message = Some(format!("Error: {}", e));
                    self.mode = Mode::Normal;
                }
            }

            self.pending_command = None;
        }
    }

    /// Close the output screen and return to normal mode.
    pub fn close_output(&mut self) {
        self.output_result = None;
        self.output_scroll = 0;
        self.mode = Mode::Normal;
    }

    /// Scroll the output view by `delta` lines (clamped to zero).
    pub fn scroll_output(&mut self, delta: i32) {
        let new = (self.output_scroll as i32 + delta).max(0) as u16;
        self.output_scroll = new;
    }

    /// Append a character to the search query and re-filter.
    pub fn push_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filter();
    }

    /// Remove the last character of the search query and re-filter.
    pub fn pop_char(&mut self) {
        self.search_query.pop();
        self.update_filter();
    }

    /// Clear the search query and re-filter.
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.update_filter();
    }

    /// Toggle between English and Arabic UI language.
    pub fn toggle_language(&mut self) {
        self.language = match self.language {
            Language::English => Language::Arabic,
            Language::Arabic => Language::English,
        };
    }

    /// Pick the string matching the current language.
    pub fn t(&self, en: &'static str, ar: &'static str) -> &'static str {
        match self.language {
            Language::English => en,
            Language::Arabic => ar,
        }
    }

    /// Begin the input flow for the selected command (only if it has placeholders).
    pub fn start_input(&mut self) -> bool {
        if let Some(&cmd_idx) = self.filtered.get(self.selected_index) {
            if self.commands[cmd_idx].has_placeholders() {
                self.pending_command = Some(cmd_idx);
                let count = self.commands[cmd_idx].placeholders.len();
                self.input_values = vec![String::new(); count];
                self.input_cursor = 0;
                self.input_field_cursor = 0;
                self.mode = Mode::Input;
                return true;
            }
        }
        false
    }

    /// Move focus to the next placeholder field (wraps around).
    pub fn next_field(&mut self) {
        if let Some(idx) = self.pending_command {
            let count = self.commands[idx].placeholders.len();
            self.input_cursor = (self.input_cursor + 1) % count;
            self.input_field_cursor = self.input_values[self.input_cursor].len();
        }
    }

    /// Move focus to the previous placeholder field (wraps around).
    pub fn prev_field(&mut self) {
        if let Some(idx) = self.pending_command {
            let count = self.commands[idx].placeholders.len();
            self.input_cursor = if self.input_cursor == 0 { count - 1 } else { self.input_cursor - 1 };
            self.input_field_cursor = self.input_values[self.input_cursor].len();
        }
    }

    /// Insert a character into the focused field at the field cursor position.
    pub fn input_char(&mut self, c: char) {
        if self.input_cursor < self.input_values.len() {
            let field = &mut self.input_values[self.input_cursor];
            let pos = self.input_field_cursor.min(field.len());
            field.insert(pos, c);
            self.input_field_cursor += 1;
        }
    }

    /// Remove the character before the field cursor in the focused field.
    pub fn input_backspace(&mut self) {
        if self.input_cursor < self.input_values.len() && self.input_field_cursor > 0 {
            let field = &mut self.input_values[self.input_cursor];
            let pos = (self.input_field_cursor - 1).min(field.len() - 1);
            if !field.is_empty() {
                field.remove(pos);
            }
            self.input_field_cursor = self.input_field_cursor.saturating_sub(1);
        }
    }

    /// Cancel the input flow and return to normal mode.
    pub fn cancel_input(&mut self) {
        self.pending_command = None;
        self.input_values.clear();
        self.input_cursor = 0;
        self.input_field_cursor = 0;
        self.mode = Mode::Normal;
    }

    /// Build the command string with placeholder values substituted.
    pub fn build_command_with_values(&self) -> Option<String> {
        let idx = self.pending_command?;
        let cmd = &self.commands[idx];
        let mut result = cmd.command.clone();
        for (i, ph) in cmd.placeholders.iter().enumerate() {
            let value = self.input_values.get(i).cloned().unwrap_or_default();
            result = result.replace(&format!("<{}>", ph.name), &value);
        }
        Some(result)
    }

    /// Execute the pending command with the entered placeholder values.
    pub fn execute_with_input(&mut self) {
        if let Some(cmd_str) = self.build_command_with_values() {
            let cwd = {
                let home = std::env::var("HOME").unwrap_or_default();
                let work_path = std::path::PathBuf::from(home).join("work");
                if work_path.exists() {
                    Some(work_path.to_string_lossy().to_string())
                } else {
                    None
                }
            };
            match crate::executor::execute(&cmd_str, cwd.as_deref(), 60) {
                Ok(result) => {
                    self.output_result = Some(result);
                    self.output_scroll = 0;
                    self.mode = Mode::Output;
                    if let Some(idx) = self.pending_command {
                        self.commands[idx].touch();
                        let _ = crate::db::storage::update_command(self.commands[idx].clone());
                    }
                }
                Err(e) => {
                    self.status_message = Some(format!("Error: {}", e));
                    self.mode = Mode::Normal;
                }
            }
            self.pending_command = None;
            self.input_values.clear();
            self.input_cursor = 0;
            self.input_field_cursor = 0;
        }
    }

    /// Save the add/edit form to the DB (add a new command or update the edited one).
    pub fn save_form(&mut self) -> anyhow::Result<()> {
        if self.add_form.command.trim().is_empty() {
            self.status_message = Some("Command cannot be empty".to_string());
            return Ok(());
        }

        let tags: Vec<String> = self.add_form.tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if let Some(id) = self.add_form.editing_id.clone() {
            // Update existing command
            if let Some(cmd) = self.commands.iter_mut().find(|c| c.id == id) {
                cmd.name_en = self.add_form.name_en.clone();
                cmd.name = self.add_form.name_en.clone();
                cmd.command = self.add_form.command.clone();
                cmd.tool = self.add_form.tool.clone();
                cmd.description_en = if self.add_form.description_en.is_empty() {
                    None
                } else {
                    Some(self.add_form.description_en.clone())
                };
                cmd.description = cmd.description_en.clone();
                cmd.category = if self.add_form.category.is_empty() {
                    None
                } else {
                    Some(self.add_form.category.clone())
                };
                cmd.tags = tags;
                let _ = crate::db::storage::update_command(cmd.clone());
                self.status_message = Some(format!("Updated: {}", cmd.name_en));
            }
        } else {
            // Add new command
            let new_id = format!("custom-{}", chrono::Utc::now().timestamp());
            let mut cmd = crate::db::models::Command::new(
                new_id,
                self.add_form.name_en.clone(),
                self.add_form.name_en.clone(),
                self.add_form.command.clone(),
                self.add_form.tool.clone(),
            );
            cmd.description_en = if self.add_form.description_en.is_empty() {
                None
            } else {
                Some(self.add_form.description_en.clone())
            };
            cmd.description = cmd.description_en.clone();
            cmd.category = if self.add_form.category.is_empty() {
                None
            } else {
                Some(self.add_form.category.clone())
            };
            cmd.tags = tags;
            let _ = crate::db::storage::add_command(cmd.clone());
            self.commands.push(cmd.clone());
            self.status_message = Some(format!("Added: {}", cmd.name_en));
        }

        self.update_filter();
        self.add_form = AddForm::default();
        self.mode = Mode::Normal;
        Ok(())
    }

    /// Delete the selected command from the DB and refresh the list.
    pub fn delete_selected(&mut self) {
        let Some(&cmd_idx) = self.filtered.get(self.selected_index) else {
            return;
        };
        let id = self.commands[cmd_idx].id.clone();
        let name = self.commands[cmd_idx].name_en.clone();
        // Defensive check: only custom commands (created via the Add form)
        // may be deleted. Built-in/seed commands must never be removed.
        if !id.starts_with("custom-") {
            self.status_message = Some(format!(
                "Cannot delete built-in [{}]: {}",
                id, name
            ));
            return;
        }
        self.commands.remove(cmd_idx);
        let _ = crate::db::storage::remove_command(&id);
        self.status_message = Some(format!("Deleted: {}", name));
        self.update_filter();
    }

    /// Toggle between Fuzzy and Ai search modes; resets AI state when leaving Ai.
    pub fn toggle_search_mode(&mut self) {
        self.search_mode = match self.search_mode {
            SearchMode::Fuzzy => SearchMode::Ai,
            SearchMode::Ai => SearchMode::Fuzzy,
        };
        // If switching to Fuzzy, clear AI state
        if self.search_mode == SearchMode::Fuzzy {
            self.ai_query.clear();
            self.ai_suggestions.clear();
            self.ai_selected = 0;
            self.ai_error = None;
        }
    }

    /// Append a character to the AI query.
    pub fn ai_push_char(&mut self, c: char) {
        self.ai_query.push(c);
    }

    /// Remove the last character of the AI query.
    pub fn ai_pop_char(&mut self) {
        self.ai_query.pop();
    }

    /// Move AI suggestion selection down (no wrap).
    pub fn ai_next(&mut self) {
        if !self.ai_suggestions.is_empty() && self.ai_selected < self.ai_suggestions.len() - 1 {
            self.ai_selected += 1;
        }
    }

    /// Move AI suggestion selection up (no wrap).
    pub fn ai_prev(&mut self) {
        if self.ai_selected > 0 {
            self.ai_selected -= 1;
        }
    }

    /// Submit the AI query and fetch suggestions (blocking network call).
    pub fn ai_submit(&mut self) {
        if self.ai_query.trim().is_empty() {
            return;
        }

        self.ai_loading = true;
        self.ai_error = None;
        self.ai_suggestions.clear();
        self.ai_selected = 0;

        // Build the command catalog from current commands
        let catalog: Vec<(String, String, String, String)> = self.commands.iter()
            .map(|c| (
                c.id.clone(),
                c.name_en.clone(),
                c.command.clone(),
                c.description_en.clone().unwrap_or_default(),
            ))
            .collect();

        match crate::ai::ask_ai(&self.ai_query, &catalog) {
            Ok(suggestions) => {
                self.ai_suggestions = suggestions;
                self.ai_error = None;
            }
            Err(e) => {
                self.ai_error = Some(format!("{}", e));
                self.ai_suggestions.clear();
            }
        }
        self.ai_loading = false;
    }

    /// Reset all AI search state.
    pub fn ai_cancel(&mut self) {
        self.ai_query.clear();
        self.ai_suggestions.clear();
        self.ai_selected = 0;
        self.ai_loading = false;
        self.ai_error = None;
    }

    /// Find the index of a command by id (returns index into self.commands).
    pub fn find_command_index(&self, id: &str) -> Option<usize> {
        self.commands.iter().position(|c| c.id == id)
    }

    /// Get the selected AI suggestion's command index (if valid).
    pub fn ai_selected_command_index(&self) -> Option<usize> {
        let suggestion = self.ai_suggestions.get(self.ai_selected)?;
        self.find_command_index(&suggestion.command_id)
    }

    /// Start an auto-discovery scan of the default tools.
    pub fn start_scan(&mut self) {
        self.scan_results.clear();
        self.scan_selected = 0;
        self.scan_scroll = 0;
        self.scan_total_commands = 0;
        self.mode = Mode::Scan;

        let tools = crate::scanner::default_tools();
        for tool in tools {
            let result = crate::scanner::scan_tool(tool);
            self.scan_total_commands += result.commands.len();
            self.scan_results.push(result);
        }
    }

    /// Cancel the scan and return to normal mode.
    pub fn cancel_scan(&mut self) {
        self.scan_results.clear();
        self.scan_selected = 0;
        self.scan_scroll = 0;
        self.scan_total_commands = 0;
        self.mode = Mode::Normal;
    }

    /// Add all discovered commands to the DB, skipping existing ids.
    pub fn scan_add_all(&mut self) -> anyhow::Result<()> {
        let mut added = 0;
        let mut skipped = 0;

        for result in &self.scan_results {
            for cmd in &result.commands {
                // Skip if a command with this id already exists
                let exists = self.commands.iter().any(|c| c.id == cmd.id);
                if exists {
                    skipped += 1;
                    continue;
                }
                let _ = crate::db::storage::add_command(cmd.clone());
                self.commands.push(cmd.clone());
                added += 1;
            }
        }

        self.update_filter();
        self.status_message = Some(format!("Scan: added {} commands ({} skipped, already existed)", added, skipped));
        self.scan_results.clear();
        self.scan_selected = 0;
        self.scan_scroll = 0;
        self.scan_total_commands = 0;
        self.mode = Mode::Normal;
        Ok(())
    }

    /// Move the scan selection down (no wrap).
    pub fn scan_next(&mut self) {
        if !self.scan_results.is_empty() && self.scan_selected < self.scan_results.len() - 1 {
            self.scan_selected += 1;
        }
    }

    /// Move the scan selection up (no wrap).
    pub fn scan_prev(&mut self) {
        if self.scan_selected > 0 {
            self.scan_selected -= 1;
        }
    }

    /// Start the add form (empty fields).
    pub fn start_add(&mut self) {
        self.add_form = AddForm::default();
        self.mode = Mode::Add;
    }

    /// Start the edit form, pre-filled with the selected command.
    pub fn start_edit(&mut self) {
        let Some(&cmd_idx) = self.filtered.get(self.selected_index) else {
            return;
        };
        let cmd = &self.commands[cmd_idx];
        self.add_form = AddForm {
            name_en: cmd.name_en.clone(),
            command: cmd.command.clone(),
            tool: cmd.tool.clone(),
            description_en: cmd.description_en.clone().unwrap_or_default(),
            category: cmd.category.clone().unwrap_or_default(),
            tags: cmd.tags.join(", "),
            field_cursor: 0,
            char_cursor: cmd.name_en.len(),
            editing_id: Some(cmd.id.clone()),
        };
        self.mode = Mode::Edit;
    }

    /// Cancel the add/edit form and return to normal mode.
    pub fn cancel_form(&mut self) {
        self.add_form = AddForm::default();
        self.mode = Mode::Normal;
    }

    /// Get a mutable reference to the field at the current field cursor.
    fn form_field(&mut self) -> &mut String {
        match self.add_form.field_cursor {
            0 => &mut self.add_form.name_en,
            1 => &mut self.add_form.command,
            2 => &mut self.add_form.tool,
            3 => &mut self.add_form.description_en,
            4 => &mut self.add_form.category,
            _ => &mut self.add_form.tags,
        }
    }

    /// Move focus to the next form field (wraps around).
    pub fn form_next_field(&mut self) {
        self.add_form.field_cursor = (self.add_form.field_cursor + 1) % 6;
        let len = self.form_field().len();
        self.add_form.char_cursor = len;
    }

    /// Move focus to the previous form field (wraps around).
    pub fn form_prev_field(&mut self) {
        self.add_form.field_cursor = if self.add_form.field_cursor == 0 {
            5
        } else {
            self.add_form.field_cursor - 1
        };
        let len = self.form_field().len();
        self.add_form.char_cursor = len;
    }

    /// Insert a character into the active form field at the char cursor.
    pub fn form_char(&mut self, c: char) {
        let pos = self.add_form.char_cursor;
        let field = self.form_field();
        let pos = pos.min(field.len());
        field.insert(pos, c);
        self.add_form.char_cursor += 1;
    }

    /// Remove the character before the char cursor in the active form field.
    pub fn form_backspace(&mut self) {
        if self.add_form.char_cursor > 0 {
            let pos = self
                .add_form
                .char_cursor
                .saturating_sub(1);
            let field = self.form_field();
            if !field.is_empty() {
                let pos = pos.min(field.len() - 1);
                field.remove(pos);
            }
            self.add_form.char_cursor -= 1;
        }
    }

    /// Toggle favorite on the selected command and persist to DB.
    pub fn toggle_favorite(&mut self) {
        if self.selected_command().is_some() {
            let idx = self.filtered[self.selected_index];
            self.commands[idx].favorite = !self.commands[idx].favorite;
            self.commands[idx].touch();
            if save_commands(&self.commands).is_ok() {
                self.status_message = Some(format!(
                    "{} التفضيل",
                    if self.commands[idx].favorite { "⭐ تم تفعيل" } else { "إلغاء" }
                ));
            } else {
                self.status_message = Some("تعذر الحفظ".to_string());
            }
        }
    }
}
