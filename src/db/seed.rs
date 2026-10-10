use crate::db::models::{Command, Placeholder};
use chrono::Utc;

fn make(
    id: &str,
    name: &str,
    name_en: &str,
    command: &str,
    tool: &str,
    category: Option<&str>,
    description: Option<&str>,
    description_en: Option<&str>,
    tags: Vec<&str>,
    placeholders: Vec<Placeholder>,
) -> Command {
    Command {
        id: id.to_string(),
        name: name.to_string(),
        name_en: name_en.to_string(),
        command: command.to_string(),
        tool: tool.to_string(),
        category: category.map(|s| s.to_string()),
        subcategory: None,
        description: description.map(|s| s.to_string()),
        description_en: description_en.map(|s| s.to_string()),
        tags: tags.into_iter().map(|s| s.to_string()).collect(),
        placeholders,
        favorite: false,
        usage_count: 0,
        last_used: None,
        created_at: Utc::now(),
    }
}

/// Load builtin commands from assets/builtin_commands.json.
/// On any error (missing file, parse error), returns an empty Vec so startup doesn't fail.
pub fn seed_builtin_safe() -> Vec<crate::db::models::Command> {
    use crate::db::builtin;

    let db = match builtin::load_builtin() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[easy] builtin_commands.json not loaded: {}", e);
            return Vec::new();
        }
    };

    let mut out = Vec::new();
    for category in db.categories {
        for bc in category.commands {
            let placeholders = bc.placeholders.into_iter()
                .map(|p| crate::db::models::Placeholder {
                    name: p.name,
                    description: p.description,
                    default: p.default,
                })
                .collect();

            let mut cmd = crate::db::models::Command::new(
                bc.id,
                bc.name_en.clone(),
                bc.name_en.clone(),
                bc.command,
                bc.tool,
            );
            cmd.category = Some(bc.category);
            cmd.subcategory = bc.subcategory;
            cmd.description_en = bc.description_en.clone();
            cmd.description = bc.description_en;
            cmd.tags = bc.tags;
            cmd.placeholders = placeholders;
            out.push(cmd);
        }
    }
    out
}

fn ph(name: &str, description: &str) -> Placeholder {
    Placeholder {
        name: name.to_string(),
        description: description.to_string(),
        default: None,
    }
}

/// Default seed commands for first run.
pub fn seed_commands() -> Vec<Command> {
    vec![
        make(
            "pi-model",
            "تغيير موديل Pi",
            "Change Pi Model",
            "pi /model",
            "pi",
            Some("الموديلات"),
            Some("تغيير موديل Pi المستخدم حالياً"),
            Some("Change the model Pi is currently using"),
            vec!["pi", "model", "settings"],
            vec![],
        ),
        make(
            "pi-install",
            "تثبيت إضافة",
            "Install Extension",
            "pi install <source>",
            "pi",
            Some("الإضافات"),
            Some("تثبيت إضافة من مصدر معين"),
            Some("Install an extension from a given source"),
            vec!["pi", "install", "extensions"],
            vec![ph("source", "المصدر (رابط أو معرف)")],
        ),
        make(
            "pi-remove",
            "إزالة إضافة",
            "Remove Extension",
            "pi remove <source>",
            "pi",
            Some("الإضافات"),
            Some("إزالة إضافة مثبتة"),
            Some("Remove an installed extension"),
            vec!["pi", "remove", "extensions"],
            vec![ph("source", "المصدر (رابط أو معرف)")],
        ),
        make(
            "pi-update",
            "تحديث Pi",
            "Update Pi",
            "pi update",
            "pi",
            Some("الصيانة"),
            Some("تحديث Pi إلى أحدث إصدار"),
            Some("Update Pi to the latest version"),
            vec!["pi", "update"],
            vec![],
        ),
        make(
            "pi-list",
            "عرض الإضافات",
            "List Extensions",
            "pi list",
            "pi",
            Some("الإضافات"),
            Some("عرض قائمة الإضافات المثبتة"),
            Some("List all installed extensions"),
            vec!["pi", "list", "extensions"],
            vec![],
        ),
        make(
            "hermes",
            "تشغيل Hermes",
            "Run Hermes",
            "hermes",
            "hermes",
            Some("Hermes"),
            Some("تشغيل Hermes"),
            Some("Run the Hermes agent"),
            vec!["hermes", "run"],
            vec![],
        ),
        make(
            "hermes-model",
            "تغيير موديل Hermes",
            "Change Hermes Model",
            "hermes /model",
            "hermes",
            Some("Hermes"),
            Some("تغيير موديل Hermes"),
            Some("Change the model Hermes is using"),
            vec!["hermes", "model"],
            vec![],
        ),
        make(
            "git-clone",
            "استنساخ مستودع",
            "Clone Repository",
            "git clone <url>",
            "git",
            Some("Git"),
            Some("استنساخ مستودع Git"),
            Some("Clone a Git repository"),
            vec!["git", "clone", "repo"],
            vec![ph("url", "رابط المستودع")],
        ),
        make(
            "git-status",
            "حالة المستودع",
            "Repository Status",
            "git status",
            "git",
            Some("Git"),
            Some("عرض حالة المستودع"),
            Some("Show current repository status"),
            vec!["git", "status"],
            vec![],
        ),
        make(
            "git-pull",
            "تحديث من السيرفر",
            "Pull from Remote",
            "git pull",
            "git",
            Some("Git"),
            Some("جلب آخر تحديثات من السيرفر"),
            Some("Fetch latest updates from remote"),
            vec!["git", "pull"],
            vec![],
        ),
    ]
}
