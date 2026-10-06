use std::fs;
use std::path::{Path, PathBuf};

use crate::models::TreeNode;
use crate::store::VaultStore;

pub struct LocalFsStore {
    root: PathBuf,
}

impl LocalFsStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl VaultStore for LocalFsStore {
    fn root(&self) -> &Path {
        &self.root
    }

    fn list_tree(&self) -> Result<Vec<TreeNode>, String> {
        read_dir_nodes(&self.root, &self.root)
    }

    fn read_note(&self, path: &str) -> Result<String, String> {
        let full = resolve_inside(&self.root, path)?;
        fs::read_to_string(&full).or_else(|_| {
            let bytes = fs::read(&full).map_err(|err| err.to_string())?;
            Ok(String::from_utf8_lossy(&bytes).into_owned())
        })
    }

    fn write_note(&self, path: &str, body: &str) -> Result<(), String> {
        let full = resolve_inside(&self.root, path)?;
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        fs::write(full, body).map_err(|err| err.to_string())
    }

    fn rename_path(&self, from: &str, to: &str) -> Result<(), String> {
        let source = resolve_inside(&self.root, from)?;
        let target = resolve_inside(&self.root, to)?;
        if !source.exists() {
            return Err("Файл не найден".into());
        }
        if target.exists() {
            return Err("Имя уже занято".into());
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        fs::rename(source, target).map_err(|err| err.to_string())
    }

    fn delete_path(&self, path: &str) -> Result<(), String> {
        let full = resolve_inside(&self.root, path)?;
        if !full.exists() {
            return Err("Файл не найден".into());
        }
        if full.is_dir() {
            fs::remove_dir_all(full).map_err(|err| err.to_string())
        } else {
            fs::remove_file(full).map_err(|err| err.to_string())
        }
    }

    fn create_folder(&self, path: &str) -> Result<(), String> {
        let full = resolve_inside(&self.root, path)?;
        if full.exists() {
            return Err("Папка уже существует".into());
        }
        fs::create_dir_all(full).map_err(|err| err.to_string())
    }
}

pub fn resolve_inside(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative = relative.replace('\\', "/");
    if relative.starts_with('/') || relative.contains('\0') {
        return Err("Недопустимый путь".into());
    }
    let mut full = root.to_path_buf();
    if relative.is_empty() {
        return Ok(full);
    }
    for part in relative.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            return Err("Путь выходит за пределы хранилища".into());
        }
        if part == ".note-gui" {
            return Err("Служебная папка недоступна".into());
        }
        full.push(part);
    }
    Ok(full)
}

pub fn join_relative(parent: &str, name: &str) -> Result<String, String> {
    let parent = parent.trim_matches('/');
    if parent.is_empty() {
        Ok(name.to_string())
    } else {
        Ok(format!("{parent}/{name}"))
    }
}

pub fn parent_relative(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((parent, _)) => parent.to_string(),
        None => String::new(),
    }
}

pub fn sanitize_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." {
        return Err("Имя не задано".into());
    }
    if name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("Имя содержит недопустимые символы".into());
    }
    Ok(name.to_string())
}

pub fn relative_path(root: &Path, full: &Path) -> Result<String, String> {
    let relative = full
        .strip_prefix(root)
        .map_err(|_| "Путь вне хранилища".to_string())?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn read_dir_nodes(root: &Path, dir: &Path) -> Result<Vec<TreeNode>, String> {
    let mut nodes = Vec::new();
    let entries = fs::read_dir(dir).map_err(|err| err.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let file_type = entry.file_type().map_err(|err| err.to_string())?;
        let path = relative_path(root, &entry.path())?;
        if file_type.is_dir() {
            let children = read_dir_nodes(root, &entry.path())?;
            nodes.push(TreeNode {
                name,
                path,
                is_dir: true,
                children,
            });
        } else if name.to_lowercase().ends_with(".md") {
            nodes.push(TreeNode {
                name,
                path,
                is_dir: false,
                children: Vec::new(),
            });
        }
    }
    nodes.sort_by(|left, right| {
        right
            .is_dir
            .cmp(&left.is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(nodes)
}
