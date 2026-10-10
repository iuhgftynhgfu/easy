use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// A placeholder inside a command, e.g. `<url>`, `<filename>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placeholder {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
}

/// A command entry in the hub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub name: String,
    pub name_en: String,
    pub command: String,
    pub tool: String,
    pub category: Option<String>,
    #[serde(default)]
    pub subcategory: Option<String>,
    pub description: Option<String>,
    pub description_en: Option<String>,
    pub tags: Vec<String>,
    pub placeholders: Vec<Placeholder>,
    pub favorite: bool,
    pub usage_count: u32,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Command {
    /// Create a new command with default values.
    pub fn new(id: String, name: String, name_en: String, command: String, tool: String) -> Self {
        Self {
            id,
            name,
            name_en,
            command,
            tool,
            category: None,
            subcategory: None,
            description: None,
            description_en: None,
            tags: Vec::new(),
            placeholders: Vec::new(),
            favorite: false,
            usage_count: 0,
            last_used: None,
            created_at: Utc::now(),
        }
    }

    /// Does this command contain any placeholders?
    pub fn has_placeholders(&self) -> bool {
        !self.placeholders.is_empty()
    }

    /// Record a usage: bump the counter and set the timestamp.
    pub fn touch(&mut self) {
        self.usage_count += 1;
        self.last_used = Some(Utc::now());
    }

    pub fn with_subcategory(mut self, sub: Option<String>) -> Self {
        self.subcategory = sub;
        self
    }
}
