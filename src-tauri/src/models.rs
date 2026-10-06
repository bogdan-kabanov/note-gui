use serde::{Deserialize, Serialize};

fn default_theme_mode() -> String {
    "system".into()
}

fn default_accent() -> String {
    "#7c6aef".into()
}

fn default_readable_line_width() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default = "default_readable_line_width")]
    pub readable_line_width: bool,
    #[serde(default)]
    pub update_manifest_url: String,
    #[serde(default)]
    pub last_vault_path: String,
    /// Разрешённые origin для будущего удалённого API. Сейчас список пуст:
    /// локальный IPC Tauri не отвечает по HTTP и CORS не применяет.
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default)]
    pub projects: Vec<VaultProject>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme_mode: default_theme_mode(),
            accent: default_accent(),
            readable_line_width: default_readable_line_width(),
            update_manifest_url: String::new(),
            last_vault_path: String::new(),
            allowed_origins: Vec::new(),
            projects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultProject {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransferResult {
    pub copied: Vec<String>,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSession {
    pub mode: String,
    pub account_id: Option<String>,
    pub vault_id: Option<String>,
    pub access_token: Option<String>,
    pub remote_status: String,
}

impl AccountSession {
    pub fn local() -> Self {
        Self {
            mode: "local".into(),
            account_id: None,
            vault_id: None,
            access_token: None,
            remote_status: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub line: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteView {
    pub path: String,
    pub body: String,
    pub title: String,
    pub property_title: String,
    pub property_tags: Vec<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub headings: Vec<Heading>,
    pub word_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub title: String,
    pub line: u32,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlink {
    pub source_path: String,
    pub source_title: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub tag: String,
    pub count: u32,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source_path: String,
    pub target_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub status: String,
    pub message: String,
    pub version: Option<String>,
    pub notes: Option<String>,
}
