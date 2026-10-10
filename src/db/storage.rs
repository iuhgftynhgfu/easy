use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::db::models::Command;

/// Path to the commands database file: ~/.config/easy/commands.json
pub fn get_db_path() -> PathBuf {
    let home = std::env::var("HOME")
        .expect("HOME environment variable is not set");
    PathBuf::from(&home).join(".config").join("easy").join("commands.json")
}

fn ensure_dir() -> Result<()> {
    let path = get_db_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {}", parent.display()))?;
    }
    Ok(())
}

/// Load all commands from disk. Returns an empty vec if the file doesn't exist.
/// Merge builtin commands into the DB. Existing commands (by id) are preserved.
/// Returns the number of NEW commands added.
pub fn merge_builtin_commands() -> anyhow::Result<usize> {
    let mut existing = load_commands()?;
    let existing_ids: std::collections::HashSet<String> =
        existing.iter().map(|c| c.id.clone()).collect();

    let builtin = crate::db::seed::seed_builtin_safe();
    let mut added = 0;
    for cmd in builtin {
        if !existing_ids.contains(&cmd.id) {
            existing.push(cmd);
            added += 1;
        }
    }

    if added > 0 {
        save_commands(&existing)?;
    }
    Ok(added)
}

pub fn load_commands() -> Result<Vec<Command>> {
    let path = get_db_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let commands: Vec<Command> = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(commands)
}

/// Save all commands to disk.
pub fn save_commands(commands: &[Command]) -> Result<()> {
    ensure_dir()?;
    let path = get_db_path();
    let raw = serde_json::to_string_pretty(commands)
        .context("failed to serialize commands")?;
    fs::write(&path, raw).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

/// Add a new command to the database.
pub fn add_command(cmd: Command) -> Result<()> {
    let mut commands = load_commands()?;
    commands.push(cmd);
    save_commands(&commands)?;
    Ok(())
}

/// Remove a command by id. Returns true if it was found and removed.
pub fn remove_command(id: &str) -> Result<bool> {
    let mut commands = load_commands()?;
    let before = commands.len();
    commands.retain(|c| c.id != id);
    let removed = commands.len() < before;
    if removed {
        save_commands(&commands)?;
    }
    Ok(removed)
}

/// Find a command by id.
pub fn find_by_id(id: &str) -> Result<Option<Command>> {
    let commands = load_commands()?;
    Ok(commands.into_iter().find(|c| c.id == id))
}

/// Replace an existing command (matched by id) with the given one.
/// Returns true if a match was found and updated.
pub fn update_command(cmd: Command) -> Result<bool> {
    let mut commands = load_commands()?;
    let mut updated = false;
    for c in &mut commands {
        if c.id == cmd.id {
            *c = cmd;
            updated = true;
            break;
        }
    }
    if updated {
        save_commands(&commands)?;
    }
    Ok(updated)
}
