use std::process::Command as ProcessCommand;
use crate::db::models::Command;

pub struct ScanResult {
    pub tool: String,
    pub commands: Vec<Command>,
    pub error: Option<String>,
}

pub fn scan_tool(tool: &str) -> ScanResult {
    let output = ProcessCommand::new(tool).arg("--help").output();
    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            let text = if text.is_empty() {
                String::from_utf8_lossy(&out.stderr).to_string()
            } else {
                text
            };
            let commands = parse_help_text(tool, &text);
            ScanResult { tool: tool.to_string(), commands, error: None }
        }
        Err(e) => ScanResult {
            tool: tool.to_string(),
            commands: Vec::new(),
            error: Some(format!("Failed: {}", e)),
        },
    }
}

fn parse_help_text(tool: &str, text: &str) -> Vec<Command> {
    use regex::Regex;
    let mut commands = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Common non-command words that Pattern B must skip (git/cargo help
    // prose lines are often indented and would otherwise be false positives).
    let skip_words: &[&str] = &[
        "usage", "options", "commands", "see", "use", "for", "and", "the", "or",
        "if", "this", "that", "with", "when", "will", "can", "be", "is", "are",
        "was", "were", "have", "has", "had", "do", "does", "did",
    ];

    // Helper: build + push a command if its id hasn't been seen yet.
    let mut add_command =
        |tool: &str, sub: &str, args: &str, desc: &str,
         commands: &mut Vec<Command>,
         seen: &mut std::collections::HashSet<String>| -> bool {
            let id = format!("scanned-{}-{}", tool, sub);
            if seen.contains(&id) {
                return false;
            }
            seen.insert(id.clone());

            let command_str = if args.is_empty() {
                format!("{} {}", tool, sub)
            } else {
                format!("{} {} {}", tool, sub, args)
            };

            let name = format!("{} {}", tool, sub);

            let mut cmd = Command::new(id, name.clone(), name, command_str, tool.to_string());
            cmd.description_en = Some(desc.to_string());
            cmd.description = Some(desc.to_string());
            cmd.category = Some("scanned".to_string());
            cmd.tags = vec!["scanned".to_string(), tool.to_string()];
            commands.push(cmd);
            true
        };

    // Pattern A: `tool subcommand [<arg>]  description` (pi/hermes/npm style).
    let pattern_a = format!(
        r"(?m)^\s*{}\s+([a-z][a-z0-9_-]*)(\s+<[^>]+>)?\s{{2,}}(.+?)$",
        regex::escape(tool)
    );

    if let Ok(re) = Regex::new(&pattern_a) {
        for cap in re.captures_iter(text) {
            let sub = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let args = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            let desc = cap.get(3).map(|m| m.as_str().trim()).unwrap_or("");

            if sub.is_empty() || sub == "help" {
                continue;
            }

            add_command(tool, sub, args, desc, &mut commands, &mut seen);
        }
    }

    // Pattern B: `  subcommand  description` (git/cargo style: subcommand
    // indented 2+ spaces, then 2+ spaces, then the description. No args).
    let pattern_b = r"(?m)^\s{2,}([a-z][a-z0-9_-]*)\s{2,}(.+?)$";

    if let Ok(re) = Regex::new(pattern_b) {
        for cap in re.captures_iter(text) {
            let sub = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let desc = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");

            if sub.is_empty() || sub == "help" {
                continue;
            }
            // Skip common non-command words (prose, not subcommands).
            if skip_words.contains(&sub) {
                continue;
            }
            // Skip subcommands shorter than 2 chars or longer than 30 chars.
            if sub.len() < 2 || sub.len() > 30 {
                continue;
            }
            if desc.is_empty() {
                continue;
            }

            add_command(tool, sub, "", desc, &mut commands, &mut seen);
        }
    }

    commands
}

pub fn default_tools() -> Vec<&'static str> {
    vec!["pi", "hermes", "git", "cargo", "npm"]
}
