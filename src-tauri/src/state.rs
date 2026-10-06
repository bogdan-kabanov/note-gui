use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use notify::RecommendedWatcher;

use crate::models::AppSettings;
use crate::vault::Vault;
use std::path::PathBuf;

pub struct VaultRuntime {
    pub vault: Vault,
    // Наблюдатель хранится здесь: если его сбросить, поток слежения за файлами остановится.
    #[allow(dead_code)]
    pub watcher: Option<RecommendedWatcher>,
    pub stop: Arc<AtomicBool>,
}

pub struct Inner {
    pub settings: AppSettings,
    pub settings_path: PathBuf,
    pub vault: Option<VaultRuntime>,
}

pub struct AppState {
    pub inner: Mutex<Inner>,
}

impl AppState {
    pub fn new(settings: AppSettings, settings_path: PathBuf) -> Self {
        Self {
            inner: Mutex::new(Inner {
                settings,
                settings_path,
                vault: None,
            }),
        }
    }
}
