use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct BuiltinDb {
    pub version: u32,
    pub categories: Vec<BuiltinCategory>,
}

#[derive(Debug, Deserialize)]
pub struct BuiltinCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub commands: Vec<BuiltinCommand>,
}

#[derive(Debug, Deserialize)]
pub struct BuiltinCommand {
    pub id: String,
    pub name_en: String,
    pub command: String,
    pub tool: String,
    pub category: String,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub description_en: Option<String>,
    #[serde(default)]
    pub placeholders: Vec<BuiltinPlaceholder>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct BuiltinPlaceholder {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub default: Option<String>,
}

/// Locate the builtin_commands.json file.
pub fn builtin_path() -> Option<PathBuf> {
    // 1. Relative to current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("assets").join("builtin_commands.json");
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Absolute fallback (installed project location)
    let p = PathBuf::from("/storage/emulated/0/Easy/assets/builtin_commands.json");
    if p.exists() {
        return Some(p);
    }

    // 3. Relative to HOME
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home)
            .join("storage")
            .join("shared")
            .join("Easy")
            .join("assets")
            .join("builtin_commands.json");
        if p.exists() {
            return Some(p);
        }
    }

    None
}

pub fn load_builtin() -> Result<BuiltinDb> {
    let path = builtin_path()
        .context("Could not find assets/builtin_commands.json")?;
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {:?}", path))?;
    let db: BuiltinDb = serde_json::from_str(&content)
        .context("Failed to parse builtin_commands.json")?;
    Ok(db)
}
