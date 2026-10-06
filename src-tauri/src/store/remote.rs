use std::path::{Path, PathBuf};

use crate::models::TreeNode;
use crate::store::VaultStore;

/// Будущий удалённый VaultStore для кооперативного аккаунта.
/// HTTP-клиента здесь нет: методы не возвращают выдуманные заметки.
/// Когда появится API, ответы должны разрешать только origin из
/// `AppSettings.allowed_origins` (заголовок Access-Control-Allow-Origin),
/// без `*` для авторизованных хранилищ. Локальный IPC Tauri CORS не использует.
pub struct RemoteStore {
    root_label: PathBuf,
}

impl RemoteStore {
    pub fn new() -> Self {
        Self {
            root_label: PathBuf::from("remote"),
        }
    }
}

impl Default for RemoteStore {
    fn default() -> Self {
        Self::new()
    }
}

fn unavailable<T>() -> Result<T, String> {
    Err("Удалённое хранилище ещё не подключено".into())
}

impl VaultStore for RemoteStore {
    fn root(&self) -> &Path {
        &self.root_label
    }

    fn list_tree(&self) -> Result<Vec<TreeNode>, String> {
        unavailable()
    }

    fn read_note(&self, _path: &str) -> Result<String, String> {
        unavailable()
    }

    fn write_note(&self, _path: &str, _body: &str) -> Result<(), String> {
        unavailable()
    }

    fn rename_path(&self, _from: &str, _to: &str) -> Result<(), String> {
        unavailable()
    }

    fn delete_path(&self, _path: &str) -> Result<(), String> {
        unavailable()
    }

    fn create_folder(&self, _path: &str) -> Result<(), String> {
        unavailable()
    }
}

#[cfg(test)]
mod tests {
    use super::RemoteStore;
    use crate::store::VaultStore;

    #[test]
    fn remote_store_does_not_invent_notes() {
        let store = RemoteStore::new();
        let error = store.list_tree().unwrap_err();
        assert!(error.contains("не подключено"));
        assert!(store.read_note("заметка.md").is_err());
    }
}
