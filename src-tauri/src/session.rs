use crate::models::AccountSession;
use crate::store::remote::RemoteStore;
use crate::store::VaultStore;

/// Локальная сессия. Поля `account_id`, `vault_id` и `access_token` зарезервированы
/// под кооперативный аккаунт и пока не заполняются.
pub fn current_session() -> AccountSession {
    let remote_store = RemoteStore::new();
    let _remote_root = remote_store.root();
    let remote_status = match remote_store.list_tree() {
        Ok(_) => "Удалённое хранилище доступно".to_string(),
        Err(error) => error,
    };
    let mut session = AccountSession::local();
    session.remote_status = remote_status;
    session
}
