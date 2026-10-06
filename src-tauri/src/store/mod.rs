use std::path::Path;

use crate::models::TreeNode;

pub mod local_fs;
pub mod remote;

pub trait VaultStore {
    fn root(&self) -> &Path;
    fn list_tree(&self) -> Result<Vec<TreeNode>, String>;
    fn read_note(&self, path: &str) -> Result<String, String>;
    fn write_note(&self, path: &str, body: &str) -> Result<(), String>;
    fn rename_path(&self, from: &str, to: &str) -> Result<(), String>;
    fn delete_path(&self, path: &str) -> Result<(), String>;
    fn create_folder(&self, path: &str) -> Result<(), String>;
}
