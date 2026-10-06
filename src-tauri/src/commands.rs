use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, State};
use tauri_plugin_updater::UpdaterExt;

use crate::models::{
    AccountSession, AppSettings, Backlink, GraphData, NoteSummary, NoteView, SearchHit, TagInfo,
    TransferResult, TreeNode, UpdateCheckResult, VaultProject,
};
use crate::parse::parse_note;
use crate::session::current_session;
use crate::settings;
use crate::state::{AppState, VaultRuntime};
use crate::store::local_fs::sanitize_name;
use crate::vault::Vault;
use crate::watcher::start_watcher;

pub struct PendingUpdate {
    pub inner: Mutex<Option<tauri_plugin_updater::Update>>,
}

fn lock_state(state: &AppState) -> Result<std::sync::MutexGuard<'_, crate::state::Inner>, String> {
    state.inner.lock().map_err(|err| err.to_string())
}

fn open_root(app: AppHandle, state: &AppState, root: PathBuf) -> Result<String, String> {
    let vault = Vault::open(root.clone())?;
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let watcher = start_watcher(app, root.clone(), stop.clone()).ok();
    let mut inner = lock_state(state)?;
    if let Some(previous) = inner.vault.take() {
        previous.stop.store(true, Ordering::Relaxed);
    }
    let path = settings::plain_path(&root);
    inner.settings.last_vault_path = path.clone();
    settings::remember_project(&mut inner.settings, &path);
    settings::save(&inner.settings_path, &inner.settings)?;
    inner.vault = Some(VaultRuntime {
        vault,
        watcher,
        stop,
    });
    Ok(path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_get(state: State<AppState>) -> Result<AppSettings, String> {
    let inner = lock_state(&state)?;
    Ok(inner.settings.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub fn settings_set(state: State<AppState>, settings_value: AppSettings) -> Result<AppSettings, String> {
    if !matches!(
        settings_value.theme_mode.as_str(),
        "system" | "light" | "dark"
    ) {
        return Err("Неизвестный режим темы".into());
    }
    if !is_accent(&settings_value.accent) {
        return Err("Цвет акцента должен быть в формате #rrggbb".into());
    }
    let mut inner = lock_state(&state)?;
    inner.settings = settings_value;
    settings::save(&inner.settings_path, &inner.settings)?;
    Ok(inner.settings.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub fn session_get() -> AccountSession {
    current_session()
}

#[tauri::command(rename_all = "snake_case")]
pub fn vault_open(app: AppHandle, state: State<AppState>, path: String) -> Result<String, String> {
    let root = PathBuf::from(&path);
    if !root.is_dir() {
        return Err("Папка не найдена".into());
    }
    open_root(app, &state, root)
}

#[tauri::command(rename_all = "snake_case")]
pub fn vault_create(
    app: AppHandle,
    state: State<AppState>,
    parent: String,
    name: String,
) -> Result<String, String> {
    let safe = sanitize_name(&name)?;
    let root = PathBuf::from(parent).join(safe);
    if root.exists() {
        return Err("Папка уже существует".into());
    }
    std::fs::create_dir_all(&root).map_err(|err| err.to_string())?;
    open_root(app, &state, root)
}

#[tauri::command(rename_all = "snake_case")]
pub fn vault_close(state: State<AppState>) -> Result<(), String> {
    let mut inner = lock_state(&state)?;
    if let Some(previous) = inner.vault.take() {
        previous.stop.store(true, Ordering::Relaxed);
    }
    inner.settings.last_vault_path.clear();
    settings::save(&inner.settings_path, &inner.settings)?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn project_rename(state: State<AppState>, path: String, name: String) -> Result<Vec<VaultProject>, String> {
    let mut inner = lock_state(&state)?;
    settings::rename_project(&mut inner.settings, &path, &name)?;
    settings::save(&inner.settings_path, &inner.settings)?;
    Ok(inner.settings.projects.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub fn project_forget(state: State<AppState>, path: String) -> Result<Vec<VaultProject>, String> {
    let mut inner = lock_state(&state)?;
    let current = if inner.vault.is_some() {
        inner.settings.last_vault_path.clone()
    } else {
        String::new()
    };
    if settings::same_location(&current, &path) {
        if let Some(previous) = inner.vault.take() {
            previous.stop.store(true, Ordering::Relaxed);
        }
        inner.settings.last_vault_path.clear();
    }
    settings::forget_project(&mut inner.settings, &path);
    settings::save(&inner.settings_path, &inner.settings)?;
    Ok(inner.settings.projects.clone())
}

#[tauri::command(rename_all = "snake_case")]
pub fn files_import(
    state: State<AppState>,
    source_paths: Vec<String>,
    parent: String,
) -> Result<TransferResult, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.import_paths(&source_paths, &parent)
}

#[tauri::command(rename_all = "snake_case")]
pub fn files_export(
    state: State<AppState>,
    relative_paths: Vec<String>,
    destination: String,
) -> Result<TransferResult, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    runtime.vault.export_paths(&relative_paths, &destination)
}

#[tauri::command(rename_all = "snake_case")]
pub fn note_export(state: State<AppState>, path: String, destination: String) -> Result<(), String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.export_note_file(&path, &destination)
}

#[tauri::command(rename_all = "snake_case")]
pub fn tree_list(state: State<AppState>) -> Result<Vec<TreeNode>, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    runtime.vault.list_tree()
}

#[tauri::command(rename_all = "snake_case")]
pub fn note_read(state: State<AppState>, path: String) -> Result<NoteView, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    runtime.vault.read_document(&path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn note_write(state: State<AppState>, path: String, body: String) -> Result<NoteView, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.write_note(&path, &body)
}

#[tauri::command(rename_all = "snake_case")]
pub fn note_create(state: State<AppState>, parent: String, name: String) -> Result<String, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.create_note(&parent, &name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn folder_create(state: State<AppState>, parent: String, name: String) -> Result<String, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.create_folder(&parent, &name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn path_rename(state: State<AppState>, path: String, name: String) -> Result<String, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.rename_path(&path, &name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn path_delete(state: State<AppState>, path: String) -> Result<(), String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.delete_path(&path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn note_set_properties(
    state: State<AppState>,
    path: String,
    title: String,
    tags: Vec<String>,
    aliases: Vec<String>,
) -> Result<NoteView, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.set_properties(&path, &title, &tags, &aliases)
}

#[tauri::command(rename_all = "snake_case")]
pub fn search_query(state: State<AppState>, query: String) -> Result<Vec<SearchHit>, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    runtime.vault.search(&query)
}

#[tauri::command(rename_all = "snake_case")]
pub fn note_summaries(state: State<AppState>) -> Result<Vec<NoteSummary>, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    runtime.vault.summaries()
}

#[tauri::command(rename_all = "snake_case")]
pub fn backlinks_for(state: State<AppState>, path: String) -> Result<Vec<Backlink>, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    runtime.vault.backlinks(&path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn tags_list(state: State<AppState>) -> Result<Vec<TagInfo>, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    runtime.vault.tags()
}

#[tauri::command(rename_all = "snake_case")]
pub fn graph_data(state: State<AppState>) -> Result<GraphData, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    runtime.vault.graph()
}

#[tauri::command(rename_all = "snake_case")]
pub fn resolve_link(state: State<AppState>, target_name: String) -> Result<String, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.open_link(&target_name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn daily_note_open(state: State<AppState>) -> Result<String, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.open_daily()
}

#[tauri::command(rename_all = "snake_case")]
pub fn bookmarks_list(state: State<AppState>) -> Result<Vec<String>, String> {
    let inner = lock_state(&state)?;
    let runtime = inner.vault.as_ref().ok_or("Хранилище не открыто")?;
    Ok(runtime.vault.bookmarks().to_vec())
}

#[tauri::command(rename_all = "snake_case")]
pub fn bookmarks_toggle(state: State<AppState>, path: String) -> Result<Vec<String>, String> {
    let mut inner = lock_state(&state)?;
    let runtime = inner.vault.as_mut().ok_or("Хранилище не открыто")?;
    runtime.vault.toggle_bookmark(&path)
}

#[tauri::command(rename_all = "snake_case")]
pub fn parse_note_body(path: String, body: String) -> NoteView {
    let parsed = parse_note(&path, &body);
    NoteView {
        path,
        body,
        title: parsed.title,
        property_title: parsed.property_title,
        property_tags: parsed.property_tags,
        tags: parsed.tags,
        aliases: parsed.aliases,
        headings: parsed.headings,
        word_count: parsed.word_count,
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_check(
    app: AppHandle,
    state: State<'_, AppState>,
    pending: State<'_, PendingUpdate>,
) -> Result<UpdateCheckResult, String> {
    let url = {
        let inner = lock_state(&state)?;
        inner.settings.update_manifest_url.trim().to_string()
    };
    if url.is_empty() {
        pending
            .inner
            .lock()
            .map_err(|err| err.to_string())?
            .take();
        return Ok(UpdateCheckResult {
            status: "not_configured".into(),
            message: "Источник обновлений не задан. Укажите адрес latest.json из GitLab Releases."
                .into(),
            version: None,
            notes: None,
        });
    }
    let parsed_url = url::Url::parse(&url).map_err(|err| format!("Некорректный адрес обновлений: {err}"))?;
    let updater = app
        .updater_builder()
        .endpoints(vec![parsed_url])
        .map_err(|err| err.to_string())?
        .build()
        .map_err(|err| err.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let notes = update.body.clone();
            *pending.inner.lock().map_err(|err| err.to_string())? = Some(update);
            Ok(UpdateCheckResult {
                status: "available".into(),
                message: format!("Доступна версия {version}"),
                version: Some(version),
                notes,
            })
        }
        Ok(None) => {
            pending
                .inner
                .lock()
                .map_err(|err| err.to_string())?
                .take();
            Ok(UpdateCheckResult {
                status: "up_to_date".into(),
                message: "Установлена последняя версия".into(),
                version: None,
                notes: None,
            })
        }
        Err(error) => {
            pending
                .inner
                .lock()
                .map_err(|err| err.to_string())?
                .take();
            Ok(UpdateCheckResult {
                status: "error".into(),
                message: error.to_string(),
                version: None,
                notes: None,
            })
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_install(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<(), String> {
    let update = pending
        .inner
        .lock()
        .map_err(|err| err.to_string())?
        .take()
        .ok_or("Нет загруженного обновления. Сначала проверьте обновления.")?;
    update
        .download_and_install(|_chunk, _total| {}, || {})
        .await
        .map_err(|err| err.to_string())?;
    app.request_restart();
    Ok(())
}

fn is_accent(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7
        && bytes[0] == b'#'
        && bytes[1..].iter().all(|byte| byte.is_ascii_hexdigit())
}
