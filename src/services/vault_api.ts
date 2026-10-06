import { invoke } from "@tauri-apps/api/core";
import type {
  account_session,
  app_settings,
  backlink,
  graph_data,
  note_summary,
  note_view,
  search_hit,
  tag_info,
  transfer_result,
  tree_node,
  update_check_result,
  vault_project,
} from "../types";

export function settings_get(): Promise<app_settings> {
  return invoke("settings_get");
}

export function settings_set(settings_value: app_settings): Promise<app_settings> {
  return invoke("settings_set", { settings_value: settings_value });
}

export function session_get(): Promise<account_session> {
  return invoke("session_get");
}

export function vault_open(path: string): Promise<string> {
  return invoke("vault_open", { path });
}

export function vault_create(parent: string, name: string): Promise<string> {
  return invoke("vault_create", { parent, name });
}

export function vault_close(): Promise<void> {
  return invoke("vault_close");
}

export function project_rename(path: string, name: string): Promise<vault_project[]> {
  return invoke("project_rename", { path, name });
}

export function project_forget(path: string): Promise<vault_project[]> {
  return invoke("project_forget", { path });
}

export function files_import(source_paths: string[], parent: string): Promise<transfer_result> {
  return invoke("files_import", { source_paths, parent });
}

export function files_export(relative_paths: string[], destination: string): Promise<transfer_result> {
  return invoke("files_export", { relative_paths, destination });
}

export function note_export(path: string, destination: string): Promise<void> {
  return invoke("note_export", { path, destination });
}

export function tree_list(): Promise<tree_node[]> {
  return invoke("tree_list");
}

export function note_read(path: string): Promise<note_view> {
  return invoke("note_read", { path });
}

export function note_write(path: string, body: string): Promise<note_view> {
  return invoke("note_write", { path, body });
}

export function note_create(parent: string, name: string): Promise<string> {
  return invoke("note_create", { parent, name });
}

export function folder_create(parent: string, name: string): Promise<string> {
  return invoke("folder_create", { parent, name });
}

export function path_rename(path: string, name: string): Promise<string> {
  return invoke("path_rename", { path, name });
}

export function path_delete(path: string): Promise<void> {
  return invoke("path_delete", { path });
}

export function note_set_properties(
  path: string,
  title: string,
  tags: string[],
  aliases: string[],
): Promise<note_view> {
  return invoke("note_set_properties", { path, title, tags, aliases });
}

export function search_query(query: string): Promise<search_hit[]> {
  return invoke("search_query", { query });
}

export function note_summaries(): Promise<note_summary[]> {
  return invoke("note_summaries");
}

export function backlinks_for(path: string): Promise<backlink[]> {
  return invoke("backlinks_for", { path });
}

export function tags_list(): Promise<tag_info[]> {
  return invoke("tags_list");
}

export function graph_data(): Promise<graph_data> {
  return invoke("graph_data");
}

export function resolve_link(target_name: string): Promise<string> {
  return invoke("resolve_link", { target_name: target_name });
}

export function daily_note_open(): Promise<string> {
  return invoke("daily_note_open");
}

export function bookmarks_list(): Promise<string[]> {
  return invoke("bookmarks_list");
}

export function bookmarks_toggle(path: string): Promise<string[]> {
  return invoke("bookmarks_toggle", { path });
}

export function parse_note_body(path: string, body: string): Promise<note_view> {
  return invoke("parse_note_body", { path, body });
}

export function update_check(): Promise<update_check_result> {
  return invoke("update_check");
}

export function update_install(): Promise<void> {
  return invoke("update_install");
}
