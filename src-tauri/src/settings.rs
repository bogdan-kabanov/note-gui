use std::fs;
use std::path::Path;

use crate::models::{AppSettings, VaultProject};

pub fn plain_path(path: &Path) -> String {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let text = canonical.to_string_lossy().to_string();
    text.trim_start_matches(r"\\?\").to_string()
}

pub fn same_location(left: &str, right: &str) -> bool {
    let left_path = Path::new(left);
    let right_path = Path::new(right);
    match (left_path.canonicalize(), right_path.canonicalize()) {
        (Ok(left_canon), Ok(right_canon)) => left_canon == right_canon,
        _ => normalize_text(left) == normalize_text(right),
    }
}

fn normalize_text(path: &str) -> String {
    path.replace('/', "\\").trim_end_matches('\\').to_ascii_lowercase()
}

pub fn remember_project(settings: &mut AppSettings, path: &str) {
    let stored = plain_path(Path::new(path));
    let folder_name = Path::new(&stored)
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| stored.clone());
    if let Some(index) = settings
        .projects
        .iter()
        .position(|item| same_location(&item.path, &stored))
    {
        let mut item = settings.projects.remove(index);
        item.path = stored;
        if item.name.trim().is_empty() {
            item.name = folder_name;
        }
        settings.projects.insert(0, item);
        return;
    }
    settings.projects.insert(
        0,
        VaultProject {
            name: folder_name,
            path: stored,
        },
    );
}

pub fn rename_project(settings: &mut AppSettings, path: &str, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Имя проекта не задано".into());
    }
    let project = settings
        .projects
        .iter_mut()
        .find(|item| same_location(&item.path, path))
        .ok_or("Проект не найден в списке")?;
    project.name = name.to_string();
    Ok(())
}

pub fn forget_project(settings: &mut AppSettings, path: &str) -> bool {
    let before = settings.projects.len();
    settings.projects.retain(|item| !same_location(&item.path, path));
    before != settings.projects.len()
}

pub fn load(path: &Path) -> AppSettings {
    let Ok(text) = fs::read_to_string(path) else {
        return AppSettings::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

pub fn save(path: &Path, settings: &AppSettings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|err| err.to_string())?;
    fs::write(path, text).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{forget_project, remember_project, rename_project};
    use crate::models::AppSettings;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn remembers_renames_and_forgets_project() {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let root = std::env::temp_dir().join(format!("note-gui-project-{millis}"));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.to_string_lossy().to_string();
        let mut settings = AppSettings::default();
        remember_project(&mut settings, &path);
        remember_project(&mut settings, &path);
        assert_eq!(settings.projects.len(), 1);
        rename_project(&mut settings, &path, "Журнал").unwrap();
        assert_eq!(settings.projects[0].name, "Журнал");
        assert!(forget_project(&mut settings, &path));
        assert!(settings.projects.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn old_settings_without_projects_still_load() {
        let json = r##"{"theme_mode":"dark","accent":"#112233","readable_line_width":false,"update_manifest_url":"","last_vault_path":"C:/vault","allowed_origins":[]}"##;
        let settings: AppSettings = serde_json::from_str(json).unwrap();
        assert!(settings.projects.is_empty());
        assert_eq!(settings.theme_mode, "dark");
        assert_eq!(settings.last_vault_path, "C:/vault");
    }
}
