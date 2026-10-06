use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::index::NoteIndex;
use crate::models::{
    Backlink, GraphData, NoteSummary, NoteView, SearchHit, TagInfo, TransferResult, TreeNode,
};
use crate::parse::{apply_properties, parse_note};
use crate::store::local_fs::{
    join_relative, parent_relative, relative_path, resolve_inside, sanitize_name, LocalFsStore,
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

    pub fn import_paths(&mut self, sources: &[String], parent: &str) -> Result<TransferResult, String> {
        if sources.is_empty() {
            return Err("Не выбраны файлы для импорта".into());
        }
        let parent_dir = resolve_inside(self.store.root(), parent)?;
        if !parent_dir.is_dir() {
            return Err("Папка назначения не найдена".into());
        }
        let vault_root = canonicalize_dir(self.store.root())?;
        let mut result = TransferResult::default();
        for source in sources {
            let source_path = PathBuf::from(source);
            if !source_path.exists() {
                result.skipped.push(format!("{source}: не найден"));
                continue;
            }
            let source_canon = canonicalize_dir(&source_path)?;
            if source_path.is_dir() {
                if source_canon == vault_root {
                    result.skipped.push(format!("{source}: это текущий проект"));
                    continue;
                }
                let parent_canon = canonicalize_dir(&parent_dir)?;
                if parent_canon.starts_with(&source_canon) {
                    result
                        .skipped
                        .push(format!("{source}: папка назначения внутри импортируемой папки"));
                    continue;
                }
                let Some(folder_name) = source_path.file_name().map(|value| value.to_string_lossy().to_string()) else {
                    result.skipped.push(format!("{source}: нет имени папки"));
                    continue;
                };
                if folder_name.starts_with('.') || folder_name == ".note-gui" {
                    result.skipped.push(format!("{folder_name}: служебная папка"));
                    continue;
                }
                let destination = unique_path(&parent_dir, &folder_name);
                copy_markdown_tree(&source_path, &destination, &mut result)?;
            } else if is_markdown_name(&source_path) {
                let file_name = source_path
                    .file_name()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| "заметка.md".into());
                let destination = unique_path(&parent_dir, &file_name);
                fs::copy(&source_path, &destination).map_err(|err| err.to_string())?;
                result
                    .copied
                    .push(relative_path(self.store.root(), &destination)?);
            } else {
                result.skipped.push(format!("{source}: нужен файл .md"));
            }
        }
        self.sync_index()?;
        Ok(result)
    }

    pub fn export_paths(&self, relative_paths: &[String], destination: &str) -> Result<TransferResult, String> {
        let destination_dir = PathBuf::from(destination);
        if destination_dir.exists() && !destination_dir.is_dir() {
            return Err("Для экспорта нужна папка".into());
        }
        fs::create_dir_all(&destination_dir).map_err(|err| err.to_string())?;
        let vault_root = canonicalize_dir(self.store.root())?;
        let destination_canon = canonicalize_dir(&destination_dir)?;
        if destination_canon.starts_with(&vault_root) {
            return Err("Нельзя экспортировать внутрь текущего проекта".into());
        }
        let mut result = TransferResult::default();
        if relative_paths.is_empty() {
            copy_markdown_tree(self.store.root(), &destination_dir, &mut result)?;
            if result.copied.is_empty() {
                result.skipped.clear();
                result.skipped.push("В проекте нет markdown-файлов".into());
            }
            return Ok(result);
        }
        for relative in relative_paths {
            let source = resolve_inside(self.store.root(), relative)?;
            if !source.exists() {
                result.skipped.push(format!("{relative}: не найден"));
                continue;
            }
            if source.is_dir() {
                let folder_name = source
                    .file_name()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| "папка".into());
                let target = unique_path(&destination_dir, &folder_name);
                copy_markdown_tree(&source, &target, &mut result)?;
            } else if is_markdown_name(&source) {
                let file_name = source
                    .file_name()
                    .map(|value| value.to_string_lossy().to_string())
                    .unwrap_or_else(|| "заметка.md".into());
                let target = unique_path(&destination_dir, &file_name);
                fs::copy(&source, &target).map_err(|err| err.to_string())?;
                result.copied.push(target.to_string_lossy().to_string());
            } else {
                result.skipped.push(format!("{relative}: нужен файл .md"));
            }
        }
        Ok(result)
    }

    pub fn export_note_file(&mut self, path: &str, destination: &str) -> Result<(), String> {
        let body = self.store.read_note(path)?;
        let destination_path = PathBuf::from(destination);
        if destination.trim().is_empty() {
            return Err("Путь экспорта не задан".into());
        }
        if let Some(parent) = destination_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|err| err.to_string())?;
            }
        }
        let vault_root = canonicalize_dir(self.store.root())?;
        if let Some(parent) = destination_path.parent() {
            if let Ok(parent_canon) = parent.canonicalize() {
                if parent_canon.starts_with(&vault_root) {
                    let file_name = destination_path
                        .file_name()
                        .ok_or("Нет имени файла")?
                        .to_string_lossy()
                        .to_string();
                    let full = parent_canon.join(&file_name);
                    let relative = full
                        .strip_prefix(&vault_root)
                        .map_err(|_| "Путь вне хранилища".to_string())?
                        .to_string_lossy()
                        .replace('\\', "/");
                    if !is_markdown_name(Path::new(&relative)) {
                        return Err("В проект можно записать только markdown".into());
                    }
                    self.write_note(&relative, &body)?;
                    return Ok(());
                }
            }
        }
        fs::write(destination_path, body).map_err(|err| err.to_string())
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

fn canonicalize_dir(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize().map_err(|err| err.to_string())
}

fn is_markdown_name(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let plain = dir.join(name);
    if !plain.exists() {
        return plain;
    }
    let file_path = Path::new(name);
    let stem = file_path
        .file_stem()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());
    let extension = file_path
        .extension()
        .map(|value| format!(".{}", value.to_string_lossy()))
        .unwrap_or_default();
    for index in 2..10_000 {
        let candidate = dir.join(format!("{stem} {index}{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    plain
}

fn place_file(dest_dir: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative_path = PathBuf::from(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    let parent = relative_path.parent().unwrap_or(Path::new(""));
    let name = relative_path
        .file_name()
        .ok_or("Нет имени файла")?
        .to_string_lossy()
        .to_string();
    let folder = dest_dir.join(parent);
    fs::create_dir_all(&folder).map_err(|err| err.to_string())?;
    Ok(unique_path(&folder, &name))
}

fn copy_markdown_tree(source: &Path, dest: &Path, result: &mut TransferResult) -> Result<(), String> {
    let mut files = Vec::new();
    collect_markdown(source, source, &mut files)?;
    if files.is_empty() {
        let label = source
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| source.display().to_string());
        result.skipped.push(format!("{label}: нет markdown-файлов"));
        return Ok(());
    }
    fs::create_dir_all(dest).map_err(|err| err.to_string())?;
    for (relative, full) in files {
        let target = place_file(dest, &relative)?;
        fs::copy(&full, &target).map_err(|err| err.to_string())?;
        result.copied.push(target.to_string_lossy().to_string());
    }
    Ok(())
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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        static TEMP_SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = TEMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!(
            "note-gui-{millis}-{}-{seq}",
            std::process::id()
        ));
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

    #[test]
    fn imports_exports_without_overwriting() {
        let root = temp_root();
        let outside = temp_root();
        let mut vault = Vault::open(root.clone()).unwrap();
        fs::write(outside.join("Импорт.md"), "# Импорт\n").unwrap();
        fs::write(outside.join("заметка.txt"), "не markdown").unwrap();
        let nested = outside.join("папка");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("Вложенная.md"), "# Вложенная\n").unwrap();

        let file = outside.join("Импорт.md").to_string_lossy().to_string();
        let imported = vault.import_paths(&[file.clone()], "").unwrap();
        assert_eq!(imported.copied.len(), 1);
        assert!(root.join("Импорт.md").is_file());
        let again = vault.import_paths(&[file], "").unwrap();
        assert_eq!(again.copied.len(), 1);
        assert!(root.join("Импорт 2.md").is_file());

        let folder = nested.to_string_lossy().to_string();
        let from_folder = vault.import_paths(&[folder], "").unwrap();
        assert_eq!(from_folder.copied.len(), 1);
        assert!(root.join("папка").join("Вложенная.md").is_file());

        let text = outside.join("заметка.txt").to_string_lossy().to_string();
        let skipped = vault.import_paths(&[text], "").unwrap();
        assert!(skipped.copied.is_empty());
        assert!(!skipped.skipped.is_empty());

        let destination = temp_root();
        let exported = vault
            .export_paths(&[], &destination.to_string_lossy())
            .unwrap();
        assert!(exported.copied.len() >= 3);
        assert!(destination.join("Импорт.md").is_file());
        assert!(destination.join("папка").join("Вложенная.md").is_file());
        assert!(vault.export_paths(&[], &root.to_string_lossy()).is_err());

        let copy_path = destination.join("копия.md");
        vault
            .export_note_file("Импорт.md", &copy_path.to_string_lossy())
            .unwrap();
        assert_eq!(fs::read_to_string(copy_path).unwrap(), "# Импорт\n");
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
        let _ = fs::remove_dir_all(destination);
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
