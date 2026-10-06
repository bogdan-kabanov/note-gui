use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::index::NoteIndex;
use crate::models::{
    Backlink, GraphData, NoteSummary, NoteView, SearchHit, TagInfo, TreeNode,
};
use crate::parse::{apply_properties, parse_note};
use crate::store::local_fs::{
    join_relative, parent_relative, resolve_inside, sanitize_name, LocalFsStore,
};
use crate::store::VaultStore;

#[derive(Debug, Default, Serialize, Deserialize)]
struct VaultSettings {
    #[serde(default)]
    bookmarks: Vec<String>,
}

pub struct Vault {
    store: LocalFsStore,
    index: NoteIndex,
    bookmarks: Vec<String>,
    bookmarks_path: PathBuf,
}

impl Vault {
    pub fn open(root: PathBuf) -> Result<Self, String> {
        if !root.is_dir() {
            return Err("Папка хранилища не найдена".into());
        }
        let meta_dir = root.join(".note-gui");
        fs::create_dir_all(&meta_dir).map_err(|err| err.to_string())?;
        let index = NoteIndex::open(&meta_dir.join("index.sqlite"))?;
        let bookmarks_path = meta_dir.join("settings.json");
        let bookmarks = load_bookmarks(&bookmarks_path);
        let mut vault = Self {
            store: LocalFsStore::new(root),
            index,
            bookmarks,
            bookmarks_path,
        };
        vault.sync_index()?;
        Ok(vault)
    }

    pub fn bookmarks(&self) -> &[String] {
        &self.bookmarks
    }

    pub fn list_tree(&self) -> Result<Vec<TreeNode>, String> {
        self.store.list_tree()
    }

    pub fn read_document(&self, path: &str) -> Result<NoteView, String> {
        let body = self.store.read_note(path)?;
        Ok(view_from(path, &body))
    }

    pub fn write_note(&mut self, path: &str, body: &str) -> Result<NoteView, String> {
        self.store.write_note(path, body)?;
        let full = resolve_inside(self.store.root(), path)?;
        self.index.upsert(path, body, mtime_ms(&full))?;
        Ok(view_from(path, body))
    }

    pub fn create_note(&mut self, parent: &str, name: &str) -> Result<String, String> {
        let file_name = ensure_markdown_name(&sanitize_name(name)?)?;
        let relative = join_relative(parent, &file_name)?;
        let full = resolve_inside(self.store.root(), &relative)?;
        if full.exists() {
            return Err("Заметка уже существует".into());
        }
        let stem = Path::new(&file_name)
            .file_stem()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or(file_name);
        let body = format!("# {stem}\n");
        self.write_note(&relative, &body)?;
        Ok(relative)
    }

    pub fn create_folder(&mut self, parent: &str, name: &str) -> Result<String, String> {
        let safe = sanitize_name(name)?;
        let relative = join_relative(parent, &safe)?;
        self.store.create_folder(&relative)?;
        Ok(relative)
    }

    pub fn rename_path(&mut self, path: &str, new_name: &str) -> Result<String, String> {
        let source = resolve_inside(self.store.root(), path)?;
        if !source.exists() {
            return Err("Не найдено".into());
        }
        let mut safe = sanitize_name(new_name)?;
        if source.is_file() {
            safe = ensure_markdown_name(&safe)?;
        }
        let relative = join_relative(&parent_relative(path), &safe)?;
        if relative == path {
            return Ok(relative);
        }
        self.store.rename_path(path, &relative)?;
        self.rewrite_bookmark_prefix(path, &relative);
        self.save_bookmarks()?;
        self.sync_index()?;
        Ok(relative)
    }

    pub fn delete_path(&mut self, path: &str) -> Result<(), String> {
        self.store.delete_path(path)?;
        let prefix = format!("{path}/");
        self.bookmarks
            .retain(|item| item != path && !item.starts_with(&prefix));
        self.save_bookmarks()?;
        self.sync_index()?;
        Ok(())
    }

    pub fn set_properties(
        &mut self,
        path: &str,
        title: &str,
        tags: &[String],
        aliases: &[String],
    ) -> Result<NoteView, String> {
        let body = self.store.read_note(path)?;
        let next = apply_properties(&body, title, tags, aliases);
        self.write_note(path, &next)
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchHit>, String> {
        self.index.search(query)
    }

    pub fn summaries(&self) -> Result<Vec<NoteSummary>, String> {
        self.index.summaries()
    }

    pub fn backlinks(&self, path: &str) -> Result<Vec<Backlink>, String> {
        self.index.backlinks(path)
    }

    pub fn tags(&self) -> Result<Vec<TagInfo>, String> {
        self.index.tags()
    }

    pub fn graph(&self) -> Result<GraphData, String> {
        self.index.graph()
    }

    pub fn resolve_link(&self, target_name: &str) -> Result<Option<String>, String> {
        self.index.resolve(target_name)
    }

    pub fn open_link(&mut self, target_name: &str) -> Result<String, String> {
        match self.resolve_link(target_name)? {
            Some(path) => Ok(path),
            None => {
                let relative = link_to_relative(target_name)?;
                let full = resolve_inside(self.store.root(), &relative)?;
                if !full.exists() {
                    let stem = Path::new(&relative)
                        .file_stem()
                        .map(|value| value.to_string_lossy().to_string())
                        .unwrap_or_else(|| target_name.to_string());
                    let body = format!("# {stem}\n");
                    self.write_note(&relative, &body)?;
                }
                Ok(relative)
            }
        }
    }

    pub fn open_daily(&mut self) -> Result<String, String> {
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let relative = format!("daily/{date}.md");
        let full = resolve_inside(self.store.root(), &relative)?;
        if !full.exists() {
            let body = format!("# {date}\n");
            self.write_note(&relative, &body)?;
        }
        Ok(relative)
    }

    pub fn toggle_bookmark(&mut self, path: &str) -> Result<Vec<String>, String> {
        if let Some(index) = self.bookmarks.iter().position(|item| item == path) {
            self.bookmarks.remove(index);
        } else {
            self.bookmarks.push(path.to_string());
        }
        self.save_bookmarks()?;
        Ok(self.bookmarks.clone())
    }

    pub fn sync_index(&mut self) -> Result<(), String> {
        let mut files = Vec::new();
        collect_markdown(self.store.root(), self.store.root(), &mut files)?;
        let mut seen = std::collections::HashSet::new();
        for (relative, full) in files {
            let body = fs::read_to_string(&full).unwrap_or_else(|_| {
                fs::read(&full)
                    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                    .unwrap_or_default()
            });
            self.index.upsert(&relative, &body, mtime_ms(&full))?;
            seen.insert(relative);
        }
        self.index.retain_paths(&seen)?;
        Ok(())
    }

    fn rewrite_bookmark_prefix(&mut self, from: &str, to: &str) {
        let nested = format!("{from}/");
        for bookmark in &mut self.bookmarks {
            if bookmark == from {
                *bookmark = to.to_string();
            } else if let Some(rest) = bookmark.strip_prefix(&nested) {
                *bookmark = format!("{to}/{rest}");
            }
        }
    }

    fn save_bookmarks(&self) -> Result<(), String> {
        let settings = VaultSettings {
            bookmarks: self.bookmarks.clone(),
        };
        let text = serde_json::to_string_pretty(&settings).map_err(|err| err.to_string())?;
        fs::write(&self.bookmarks_path, text).map_err(|err| err.to_string())
    }
}

fn view_from(path: &str, body: &str) -> NoteView {
    let parsed = parse_note(path, body);
    NoteView {
        path: path.to_string(),
        body: body.to_string(),
        title: parsed.title,
        property_title: parsed.property_title,
        property_tags: parsed.property_tags,
        tags: parsed.tags,
        aliases: parsed.aliases,
        headings: parsed.headings,
        word_count: parsed.word_count,
    }
}

fn load_bookmarks(path: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str::<VaultSettings>(&text)
        .map(|settings| settings.bookmarks)
        .unwrap_or_default()
}

fn collect_markdown(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|err| err.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(root, &path, out)?;
        } else if name.to_lowercase().ends_with(".md") {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| "Путь вне хранилища".to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, path));
        }
    }
    Ok(())
}

fn mtime_ms(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_millis() as i64)
                .unwrap_or(0)
        })
}

fn ensure_markdown_name(name: &str) -> Result<String, String> {
    if name.to_lowercase().ends_with(".md") {
        Ok(name.to_string())
    } else {
        Ok(format!("{name}.md"))
    }
}

#[cfg(test)]
mod tests {
    use super::Vault;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("note-gui-{millis}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn creates_links_search_graph_and_deletes() {
        let root = temp_root();
        let mut vault = Vault::open(root.clone()).unwrap();
        let alpha = vault.create_note("", "Альфа").unwrap();
        let beta = vault.create_note("журнал", "Бета").unwrap();
        vault
            .write_note(
                &alpha,
                "---\ntitle: Альфа\naliases:\n  - Первая\n---\n# Альфа\n\nСвязь [[Бета]] и #проект\n",
            )
            .unwrap();
        let hits = vault.search("проект").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, alpha);
        let backlinks = vault.backlinks(&beta).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_path, alpha);
        let resolved = vault.resolve_link("Первая").unwrap();
        assert_eq!(resolved.as_deref(), Some(alpha.as_str()));
        let graph = vault.graph().unwrap();
        assert!(graph.edges.iter().any(|edge| {
            edge.source_path == alpha && edge.target_path == beta
        }));
        let bookmarks = vault.toggle_bookmark(&alpha).unwrap();
        assert_eq!(bookmarks, vec![alpha.clone()]);
        let renamed = vault.rename_path(&beta, "Гамма").unwrap();
        assert!(renamed.ends_with("Гамма.md"));
        let updated = vault
            .set_properties(&alpha, "Новое", &["тег".into()], &[])
            .unwrap();
        assert_eq!(updated.property_title, "Новое");
        assert!(updated.tags.iter().any(|tag| tag == "тег"));
        assert!(vault.read_document("../secret.md").is_err());
        vault.delete_path(&alpha).unwrap();
        let tree = vault.list_tree().unwrap();
        assert!(tree.iter().all(|node| node.name != "Альфа.md"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn daily_note_uses_real_file() {
        let root = temp_root();
        let mut vault = Vault::open(root.clone()).unwrap();
        let path = vault.open_daily().unwrap();
        let document = vault.read_document(&path).unwrap();
        assert!(document.body.starts_with("# "));
        let again = vault.open_daily().unwrap();
        assert_eq!(path, again);
        let _ = fs::remove_dir_all(root);
    }
}

fn link_to_relative(target_name: &str) -> Result<String, String> {
    let target = target_name.trim().replace('\\', "/");
    if target.is_empty() || target.contains("..") || target.starts_with('/') {
        return Err("Нельзя создать заметку по этой ссылке".into());
    }
    let mut parts = Vec::new();
    for part in target.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        parts.push(sanitize_name(part)?);
    }
    if parts.is_empty() {
        return Err("Нельзя создать заметку по этой ссылке".into());
    }
    let mut relative = parts.join("/");
    if !relative.to_lowercase().ends_with(".md") {
        relative.push_str(".md");
    }
    Ok(relative)
}
