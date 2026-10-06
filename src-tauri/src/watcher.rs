use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

pub fn start_watcher(
    app: AppHandle,
    root: std::path::PathBuf,
    stop: Arc<AtomicBool>,
) -> Result<RecommendedWatcher, String> {
    let (sender, receiver) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |result| {
        let _ = sender.send(result);
    })
    .map_err(|err| err.to_string())?;
    watcher
        .watch(&root, RecursiveMode::Recursive)
        .map_err(|err| err.to_string())?;
    let stop_flag = stop.clone();
    std::thread::spawn(move || {
        let mut dirty = false;
        let mut last_change = Instant::now();
        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
            match receiver.recv_timeout(Duration::from_millis(250)) {
                Ok(Ok(event)) => {
                    if event.kind.is_access() || !event_is_relevant(&event.paths) {
                        continue;
                    }
                    dirty = true;
                    last_change = Instant::now();
                }
                Ok(Err(_)) => {}
                Err(RecvTimeoutError::Timeout) => {
                    if dirty && last_change.elapsed() >= Duration::from_millis(400) {
                        dirty = false;
                        if rescan(&app).is_ok() {
                            let _ = app.emit("vault_changed", ());
                        }
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    Ok(watcher)
}

fn event_is_relevant(paths: &[std::path::PathBuf]) -> bool {
    paths.iter().any(|path| !path_is_internal(path))
}

fn path_is_internal(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(".note-gui")
    })
}

fn rescan(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut inner = state.inner.lock().map_err(|err| err.to_string())?;
    if let Some(runtime) = inner.vault.as_mut() {
        runtime.vault.sync_index()?;
    }
    Ok(())
}
