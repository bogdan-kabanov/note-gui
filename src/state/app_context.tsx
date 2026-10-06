import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { AppContext } from "./context";
import * as api from "../services/vault_api";
import type {
  app_settings,
  backlink,
  center_view,
  focused_pane,
  graph_data,
  left_panel,
  name_modal_state,
  note_summary,
  note_view,
  right_tab,
  search_hit,
  tag_info,
  theme_mode,
  transfer_result,
  tree_node,
  update_check_result,
  vault_project,
  view_mode,
} from "../types";

export type app_api = {
  ready: boolean;
  vault_path: string | null;
  tree: tree_node[];
  tabs: string[];
  active_path: string | null;
  split_path: string | null;
  split_open: boolean;
  focused_pane: focused_pane;
  focused_path: string | null;
  main_view_mode: view_mode;
  split_view_mode: view_mode;
  center_view: center_view;
  left_panel: left_panel;
  left_open: boolean;
  right_open: boolean;
  right_tab: right_tab;
  theme_mode: theme_mode;
  resolved_theme: "light" | "dark";
  accent: string;
  readable_line_width: boolean;
  update_manifest_url: string;
  allowed_origins: string[];
  bookmarks: string[];
  summaries: note_summary[];
  bodies: Record<string, string>;
  views: Record<string, note_view>;
  word_count: number;
  command_palette_open: boolean;
  quick_switcher_open: boolean;
  settings_open: boolean;
  name_modal: name_modal_state | null;
  confirm_delete: string | null;
  status_message: string;
  session_mode: string;
  remote_status: string;
  graph: graph_data | null;
  search_query_text: string;
  search_hits: search_hit[];
  tags: tag_info[];
  backlinks: backlink[];
  update_result: update_check_result | null;
  scroll_line: number | null;
  projects: vault_project[];
  projects_open: boolean;
  project_name: string;
  open_projects: () => void;
  close_projects: () => void;
  open_project: (path: string) => Promise<void>;
  close_project: () => Promise<void>;
  forget_project: (path: string) => Promise<void>;
  ask_rename_project: (project: vault_project) => void;
  import_files: () => Promise<void>;
  import_folder: () => Promise<void>;
  export_project: () => Promise<void>;
  export_path: (path: string) => Promise<void>;
  export_current_note: () => Promise<void>;
  open_vault_dialog: () => Promise<void>;
  create_vault_dialog: () => Promise<void>;
  open_note: (path: string) => Promise<void>;
  close_tab: (path: string) => void;
  change_body: (path: string, body: string) => void;
  ask_create_note: () => void;
  ask_create_folder: () => void;
  ask_rename: (path: string) => void;
  submit_name: (name: string) => Promise<void>;
  close_name_modal: () => void;
  ask_delete: (path: string) => void;
  confirm_delete_path: () => Promise<void>;
  close_confirm: () => void;
  toggle_bookmark: (path: string) => Promise<void>;
  open_daily: () => Promise<void>;
  set_left_panel: (panel: left_panel) => void;
  toggle_left: () => void;
  toggle_right: () => void;
  toggle_split: () => void;
  toggle_view_mode: () => void;
  set_pane_mode: (pane: focused_pane, mode: view_mode) => void;
  set_focused_pane: (pane: focused_pane) => void;
  set_center_view: (view: center_view) => void;
  set_right_tab: (tab: right_tab) => void;
  set_theme_mode: (mode: theme_mode) => Promise<void>;
  set_accent: (accent: string) => Promise<void>;
  set_readable_line_width: (enabled: boolean) => Promise<void>;
  set_update_manifest_url: (url: string) => void;
  save_update_url: () => Promise<void>;
  check_updates: () => Promise<void>;
  install_update: () => Promise<void>;
  run_search: (query: string) => Promise<void>;
  open_link: (target_name: string) => Promise<void>;
  save_properties: (path: string, title: string, tags: string[], aliases: string[]) => Promise<void>;
  open_command_palette: () => void;
  close_command_palette: () => void;
  open_quick_switcher: () => void;
  close_quick_switcher: () => void;
  open_settings: () => void;
  close_settings: () => void;
  dismiss_status: () => void;
  scroll_to_line: (line: number) => void;
  select_dir: (path: string) => void;
  selected_dir: string;
};

function error_text(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Неизвестная ошибка";
}

function parent_of(path: string): string {
  const index = path.lastIndexOf("/");
  return index === -1 ? "" : path.slice(0, index);
}

function file_title(path: string): string {
  const name = path.split("/").pop()?.split("\\").pop() ?? path;
  return name.replace(/\.md$/i, "");
}

function same_path(left: string, right: string): boolean {
  const normalize = (value: string) => value.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
  return normalize(left) === normalize(right);
}

function selected_paths(value: unknown): string[] {
  if (typeof value === "string" && value.length > 0) {
    return [value];
  }
  if (Array.isArray(value)) {
    return value.filter((item): item is string => typeof item === "string" && item.length > 0);
  }
  return [];
}

function transfer_message(action: string, result: transfer_result): string {
  if (result.copied.length === 0) {
    return result.skipped[0] ?? `${action}: нечего переносить`;
  }
  if (result.skipped.length === 0) {
    return `${action}: ${result.copied.length}`;
  }
  return `${action}: ${result.copied.length}, пропущено ${result.skipped.length}`;
}

function folder_name(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

const default_settings: app_settings = {
  theme_mode: "system",
  accent: "#7c6aef",
  readable_line_width: true,
  update_manifest_url: "",
  last_vault_path: "",
  allowed_origins: [],
  projects: [],
};

export function AppProvider({ children }: { children: ReactNode }) {
  const [ready, set_ready] = useState(false);
  const [vault_path, set_vault_path] = useState<string | null>(null);
  const [tree, set_tree] = useState<tree_node[]>([]);
  const [tabs, set_tabs] = useState<string[]>([]);
  const [active_path, set_active_path] = useState<string | null>(null);
  const [split_path, set_split_path] = useState<string | null>(null);
  const [split_open, set_split_open] = useState(false);
  const [focused_pane, set_focused_pane] = useState<focused_pane>("main");
  const [main_view_mode, set_main_view_mode] = useState<view_mode>("source");
  const [split_view_mode, set_split_view_mode] = useState<view_mode>("source");
  const [center_view, set_center_view] = useState<center_view>("editor");
  const [left_panel, set_left_panel] = useState<left_panel>("files");
  const [left_open, set_left_open] = useState(true);
  const [right_open, set_right_open] = useState(true);
  const [right_tab, set_right_tab] = useState<right_tab>("outline");
  const [theme_mode, set_theme_mode_state] = useState<theme_mode>("system");
  const [resolved_theme, set_resolved_theme] = useState<"light" | "dark">("light");
  const [accent, set_accent_state] = useState("#7c6aef");
  const [readable_line_width, set_readable_state] = useState(true);
  const [update_manifest_url, set_update_manifest_url] = useState("");
  const [allowed_origins, set_allowed_origins] = useState<string[]>([]);
  const [bookmarks, set_bookmarks] = useState<string[]>([]);
  const [summaries, set_summaries] = useState<note_summary[]>([]);
  const [bodies, set_bodies] = useState<Record<string, string>>({});
  const [views, set_views] = useState<Record<string, note_view>>({});
  const [command_palette_open, set_command_palette_open] = useState(false);
  const [quick_switcher_open, set_quick_switcher_open] = useState(false);
  const [settings_open, set_settings_open] = useState(false);
  const [name_modal, set_name_modal] = useState<name_modal_state | null>(null);
  const [confirm_delete, set_confirm_delete] = useState<string | null>(null);
  const [status_message, set_status_message] = useState("");
  const [session_mode, set_session_mode] = useState("local");
  const [remote_status, set_remote_status] = useState("Удалённое хранилище ещё не подключено");
  const [graph, set_graph] = useState<graph_data | null>(null);
  const [search_query_text, set_search_query_text] = useState("");
  const [search_hits, set_search_hits] = useState<search_hit[]>([]);
  const [tags, set_tags] = useState<tag_info[]>([]);
  const [backlinks, set_backlinks] = useState<backlink[]>([]);
  const [update_result, set_update_result] = useState<update_check_result | null>(null);
  const [scroll_line, set_scroll_line] = useState<number | null>(null);
  const [selected_dir, set_selected_dir] = useState("");
  const [projects, set_projects] = useState<vault_project[]>([]);
  const [projects_open, set_projects_open] = useState(false);
  const actions_ref = useRef({
    ask_create_note: () => {},
    toggle_view_mode: () => {},
  });
  const save_timers = useRef<Record<string, number>>({});
  const parse_timers = useRef<Record<string, number>>({});
  const saving_paths = useRef<Set<string>>(new Set());
  const settings_ref = useRef<app_settings>(default_settings);
  const vault_path_ref = useRef<string | null>(null);
  const active_path_ref = useRef<string | null>(null);
  const split_path_ref = useRef<string | null>(null);
  const center_view_ref = useRef<center_view>("editor");

  const focused_path = focused_pane === "split" ? split_path : active_path;
  const word_count = focused_path ? (views[focused_path]?.word_count ?? 0) : 0;
  const project_name = vault_path
    ? (projects.find((item) => same_path(item.path, vault_path))?.name ?? folder_name(vault_path))
    : "";

  useEffect(() => {
    vault_path_ref.current = vault_path;
    active_path_ref.current = active_path;
    split_path_ref.current = split_path;
    center_view_ref.current = center_view;
  }, [vault_path, active_path, split_path, center_view]);

  async function persist_settings(next: app_settings) {
    settings_ref.current = next;
    const saved = await api.settings_set(next);
    settings_ref.current = saved;
  }

  async function refresh_indexes() {
    const [next_tree, next_bookmarks, next_tags, next_summaries] = await Promise.all([
      api.tree_list(),
      api.bookmarks_list(),
      api.tags_list(),
      api.note_summaries(),
    ]);
    set_tree(next_tree);
    set_bookmarks(next_bookmarks);
    set_tags(next_tags);
    set_summaries(next_summaries);
  }

  async function load_backlinks(path: string | null) {
    if (!path) {
      set_backlinks([]);
      return;
    }
    set_backlinks(await api.backlinks_for(path));
  }

  async function open_note(path: string) {
    const note = await api.note_read(path);
    set_bodies((current) => ({ ...current, [path]: note.body }));
    set_views((current) => ({ ...current, [path]: note }));
    set_tabs((current) => (current.includes(path) ? current : [...current, path]));
    if (focused_pane === "split" && split_open) {
      set_split_path(path);
    } else {
      set_active_path(path);
    }
    set_center_view("editor");
    set_selected_dir(parent_of(path));
    await load_backlinks(path);
  }

  async function open_link(target_name: string) {
    try {
      const path = await api.resolve_link(target_name);
      await refresh_indexes();
      await open_note(path);
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function reload_open_notes() {
    const paths = [active_path_ref.current, split_path_ref.current].filter(
      (path): path is string => Boolean(path),
    );
    for (const path of paths) {
      if (saving_paths.current.has(path)) {
        continue;
      }
      try {
        const note = await api.note_read(path);
        set_bodies((current) => ({ ...current, [path]: note.body }));
        set_views((current) => ({ ...current, [path]: note }));
      } catch {
        set_tabs((current) => current.filter((item) => item !== path));
        if (active_path_ref.current === path) {
          set_active_path(null);
        }
        if (split_path_ref.current === path) {
          set_split_path(null);
        }
      }
    }
  }

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply_theme = () => {
      const next = theme_mode === "system" ? (media.matches ? "dark" : "light") : theme_mode;
      set_resolved_theme(next);
      document.documentElement.dataset.theme = next;
      document.documentElement.style.setProperty("--accent", accent);
    };
    apply_theme();
    media.addEventListener("change", apply_theme);
    return () => media.removeEventListener("change", apply_theme);
  }, [theme_mode, accent]);

  useEffect(() => {
    let unlisten = () => {};
    const on_change = () => {
      if (!vault_path_ref.current) {
        return;
      }
      void refresh_indexes()
        .then(() => reload_open_notes())
        .then(() => load_backlinks(active_path_ref.current))
        .then(async () => {
          if (center_view_ref.current === "graph") {
            set_graph(await api.graph_data());
          }
        })
        .catch((error: unknown) => set_status_message(error_text(error)));
    };
    if ("__TAURI_INTERNALS__" in window) {
      void listen("vault_changed", on_change)
        .then((stop) => {
          unlisten = stop;
        })
        .catch((error: unknown) => set_status_message(error_text(error)));
    }
    return () => unlisten();
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const settings = await api.settings_get();
        if (cancelled) {
          return;
        }
        settings_ref.current = settings;
        set_theme_mode_state(settings.theme_mode);
        set_accent_state(settings.accent);
        set_readable_state(settings.readable_line_width);
        set_update_manifest_url(settings.update_manifest_url);
        set_allowed_origins(settings.allowed_origins);
        set_projects(settings.projects);
        const session = await api.session_get();
        if (!cancelled) {
          set_session_mode(session.mode);
          set_remote_status(session.remote_status);
        }
        if (settings.last_vault_path) {
          try {
            const opened = await api.vault_open(settings.last_vault_path);
            if (cancelled) {
              return;
            }
            set_vault_path(opened);
            await sync_projects();
            await refresh_indexes();
          } catch (error) {
            if (!cancelled) {
              set_status_message(error_text(error));
            }
          }
        }
        const update = await api.update_check();
        if (!cancelled) {
          set_update_result(update);
          if (update.status === "available") {
            set_status_message(update.message);
          }
        }
      } catch (error) {
        if (!cancelled) {
          set_status_message(error_text(error));
        }
      } finally {
        if (!cancelled) {
          set_ready(true);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    function on_key_down(event: KeyboardEvent) {
      const key = event.key.toLowerCase();
      const mod = event.ctrlKey || event.metaKey;
      if (key === "escape") {
        set_command_palette_open(false);
        set_quick_switcher_open(false);
        set_settings_open(false);
        set_name_modal(null);
        set_confirm_delete(null);
        set_projects_open(false);
        return;
      }
      if (!mod) {
        return;
      }
      if (key === "p") {
        event.preventDefault();
        set_quick_switcher_open(false);
        set_command_palette_open((open) => !open);
      } else if (key === "o") {
        event.preventDefault();
        set_command_palette_open(false);
        set_quick_switcher_open((open) => !open);
      } else       if (key === "n" && vault_path_ref.current) {
        event.preventDefault();
        actions_ref.current.ask_create_note();
      } else if (key === "e") {
        event.preventDefault();
        actions_ref.current.toggle_view_mode();
      } else if (key === "\\") {
        event.preventDefault();
        set_left_open((open) => !open);
      } else if (key === "f" && event.shiftKey) {
        event.preventDefault();
        set_left_open(true);
        set_left_panel("search");
      }
    }
    window.addEventListener("keydown", on_key_down);
    return () => window.removeEventListener("keydown", on_key_down);
  }, []);

  function current_settings(): app_settings {
    return {
      ...settings_ref.current,
      theme_mode,
      accent,
      readable_line_width,
      update_manifest_url,
      allowed_origins,
      last_vault_path: vault_path ?? settings_ref.current.last_vault_path,
    };
  }

  async function sync_projects() {
    const settings = await api.settings_get();
    settings_ref.current = settings;
    set_projects(settings.projects);
  }

  function reset_editor() {
    set_tabs([]);
    set_active_path(null);
    set_split_path(null);
    set_bodies({});
    set_views({});
    set_selected_dir("");
    set_search_hits([]);
    set_search_query_text("");
    set_graph(null);
    set_backlinks([]);
    set_center_view("editor");
  }

  async function finish_open(opened: string) {
    set_vault_path(opened);
    reset_editor();
    await sync_projects();
    await refresh_indexes();
    set_projects_open(false);
    set_status_message("");
  }

  async function open_project(path: string) {
    try {
      const opened = await api.vault_open(path);
      await finish_open(opened);
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function close_project() {
    try {
      await api.vault_close();
      set_vault_path(null);
      reset_editor();
      await sync_projects();
      set_status_message("");
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function forget_project(path: string) {
    try {
      const next = await api.project_forget(path);
      settings_ref.current = { ...settings_ref.current, projects: next };
      set_projects(next);
      if (vault_path && same_path(vault_path, path)) {
        set_vault_path(null);
        reset_editor();
      }
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  function ask_rename_project(project: vault_project) {
    set_name_modal({
      title: "Название проекта",
      label: "Имя в списке",
      initial: project.name,
      kind: "project",
      parent_path: "",
      target_path: project.path,
      vault_parent: "",
    });
  }

  async function import_files() {
    if (!vault_path) {
      set_status_message("Сначала откройте проект");
      return;
    }
    const selected = await open({
      title: "Импорт заметок",
      multiple: true,
      directory: false,
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    const paths = selected_paths(selected);
    if (paths.length === 0) {
      return;
    }
    try {
      const result = await api.files_import(paths, selected_dir);
      await refresh_indexes();
      set_status_message(transfer_message("Импортировано", result));
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function import_folder() {
    if (!vault_path) {
      set_status_message("Сначала откройте проект");
      return;
    }
    const selected = await open({
      title: "Импорт папки с заметками",
      directory: true,
      multiple: false,
    });
    if (typeof selected !== "string") {
      return;
    }
    try {
      const result = await api.files_import([selected], selected_dir);
      await refresh_indexes();
      set_status_message(transfer_message("Импортировано", result));
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function export_project() {
    if (!vault_path) {
      set_status_message("Сначала откройте проект");
      return;
    }
    const selected = await open({
      title: "Куда экспортировать проект",
      directory: true,
      multiple: false,
    });
    if (typeof selected !== "string") {
      return;
    }
    try {
      const result = await api.files_export([], selected);
      set_status_message(transfer_message("Экспортировано", result));
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function export_path(path: string) {
    if (!vault_path) {
      set_status_message("Сначала откройте проект");
      return;
    }
    const selected = await open({
      title: "Куда экспортировать",
      directory: true,
      multiple: false,
    });
    if (typeof selected !== "string") {
      return;
    }
    try {
      const result = await api.files_export([path], selected);
      set_status_message(transfer_message("Экспортировано", result));
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function export_current_note() {
    const path = focused_path;
    if (!path) {
      set_status_message("Сначала откройте заметку");
      return;
    }
    const body = bodies[path];
    if (body !== undefined) {
      await persist_body(path, body);
    }
    const destination = await save({
      title: "Экспорт заметки",
      defaultPath: `${file_title(path)}.md`,
      filters: [{ name: "Markdown", extensions: ["md"] }],
    });
    if (typeof destination !== "string" || destination.length === 0) {
      return;
    }
    try {
      await api.note_export(path, destination);
      set_status_message("Заметка экспортирована");
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function open_vault_dialog() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Открыть хранилище",
    });
    if (typeof selected !== "string") {
      return;
    }
    try {
      const opened = await api.vault_open(selected);
      await finish_open(opened);
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function create_vault_dialog() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Где создать хранилище",
    });
    if (typeof selected !== "string") {
      return;
    }
    set_name_modal({
      title: "Новое хранилище",
      label: "Название папки",
      initial: "Заметки",
      kind: "vault",
      parent_path: "",
      target_path: "",
      vault_parent: selected,
    });
  }

  function ask_create_note() {
    set_name_modal({
      title: "Новая заметка",
      label: "Имя",
      initial: "",
      kind: "note",
      parent_path: selected_dir || parent_of(focused_path ?? ""),
      target_path: "",
      vault_parent: "",
    });
  }

  function ask_create_folder() {
    set_name_modal({
      title: "Новая папка",
      label: "Имя",
      initial: "",
      kind: "folder",
      parent_path: selected_dir || parent_of(focused_path ?? ""),
      target_path: "",
      vault_parent: "",
    });
  }

  function ask_rename(path: string) {
    set_name_modal({
      title: "Переименовать",
      label: "Новое имя",
      initial: file_title(path),
      kind: "rename",
      parent_path: parent_of(path),
      target_path: path,
      vault_parent: "",
    });
  }

  async function submit_name(name: string) {
    if (!name_modal) {
      return;
    }
    try {
      if (name_modal.kind === "vault") {
        const opened = await api.vault_create(name_modal.vault_parent, name);
        await finish_open(opened);
      } else if (name_modal.kind === "project") {
        const next = await api.project_rename(name_modal.target_path, name);
        settings_ref.current = { ...settings_ref.current, projects: next };
        set_projects(next);
      } else if (name_modal.kind === "note") {
        const path = await api.note_create(name_modal.parent_path, name);
        await refresh_indexes();
        await open_note(path);
      } else if (name_modal.kind === "folder") {
        const path = await api.folder_create(name_modal.parent_path, name);
        set_selected_dir(path);
        await refresh_indexes();
      } else {
        const renamed = await api.path_rename(name_modal.target_path, name);
        set_tabs((current) =>
          current.map((item) => (item === name_modal.target_path ? renamed : item)),
        );
        if (active_path === name_modal.target_path) {
          set_active_path(renamed);
        }
        if (split_path === name_modal.target_path) {
          set_split_path(renamed);
        }
        set_bodies((current) => {
          const next = { ...current };
          if (name_modal.target_path in next) {
            next[renamed] = next[name_modal.target_path];
            delete next[name_modal.target_path];
          }
          return next;
        });
        await refresh_indexes();
        await open_note(renamed);
      }
      set_name_modal(null);
      set_status_message("");
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  function close_tab(path: string) {
    set_tabs((current) => {
      const next = current.filter((item) => item !== path);
      if (active_path === path) {
        set_active_path(next[next.length - 1] ?? null);
      }
      return next;
    });
    if (split_path === path) {
      set_split_path(null);
    }
  }

  function change_body(path: string, body: string) {
    set_bodies((current) => ({ ...current, [path]: body }));
    window.clearTimeout(save_timers.current[path]);
    save_timers.current[path] = window.setTimeout(() => {
      void persist_body(path, body);
    }, 450);
    window.clearTimeout(parse_timers.current[path]);
    parse_timers.current[path] = window.setTimeout(() => {
      void api
        .parse_note_body(path, body)
        .then((view) => set_views((current) => ({ ...current, [path]: view })))
        .catch(() => undefined);
    }, 180);
  }

  async function persist_body(path: string, body: string) {
    saving_paths.current.add(path);
    try {
      const view = await api.note_write(path, body);
      set_views((current) => ({ ...current, [path]: view }));
      await refresh_indexes();
      if (active_path_ref.current === path) {
        await load_backlinks(path);
      }
    } catch (error) {
      set_status_message(error_text(error));
    } finally {
      window.setTimeout(() => saving_paths.current.delete(path), 800);
    }
  }

  async function confirm_delete_path() {
    if (!confirm_delete) {
      return;
    }
    try {
      await api.path_delete(confirm_delete);
      close_tab(confirm_delete);
      await refresh_indexes();
      set_confirm_delete(null);
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function toggle_bookmark(path: string) {
    try {
      set_bookmarks(await api.bookmarks_toggle(path));
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function open_daily() {
    try {
      const path = await api.daily_note_open();
      await refresh_indexes();
      await open_note(path);
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  function toggle_view_mode() {
    if (focused_pane === "split") {
      set_split_view_mode((mode) => (mode === "source" ? "preview" : "source"));
    } else {
      set_main_view_mode((mode) => (mode === "source" ? "preview" : "source"));
    }
  }

  actions_ref.current.ask_create_note = ask_create_note;
  actions_ref.current.toggle_view_mode = toggle_view_mode;

  async function set_theme_mode(mode: theme_mode) {
    set_theme_mode_state(mode);
    try {
      await persist_settings({ ...current_settings(), theme_mode: mode });
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function set_accent(next_accent: string) {
    set_accent_state(next_accent);
    try {
      await persist_settings({ ...current_settings(), accent: next_accent });
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function set_readable_line_width(enabled: boolean) {
    set_readable_state(enabled);
    try {
      await persist_settings({ ...current_settings(), readable_line_width: enabled });
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function save_update_url() {
    try {
      await persist_settings(current_settings());
      set_status_message("Адрес обновлений сохранён");
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function check_updates() {
    try {
      await persist_settings(current_settings());
      const result = await api.update_check();
      set_update_result(result);
      set_status_message(result.message);
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function install_update() {
    try {
      set_status_message("Установка обновления…");
      await api.update_install();
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function run_search(query: string) {
    set_search_query_text(query);
    if (!vault_path || query.trim() === "") {
      set_search_hits([]);
      return;
    }
    try {
      set_search_hits(await api.search_query(query));
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function save_properties(path: string, title: string, tags_value: string[], aliases: string[]) {
    try {
      const view = await api.note_set_properties(path, title, tags_value, aliases);
      set_bodies((current) => ({ ...current, [path]: view.body }));
      set_views((current) => ({ ...current, [path]: view }));
      await refresh_indexes();
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  async function show_graph() {
    set_center_view("graph");
    set_left_panel("files");
    if (!vault_path) {
      return;
    }
    try {
      set_graph(await api.graph_data());
    } catch (error) {
      set_status_message(error_text(error));
    }
  }

  const api_value = useMemo<app_api>(
    () => ({
      ready,
      vault_path,
      tree,
      tabs,
      active_path,
      split_path,
      split_open,
      focused_pane,
      focused_path,
      main_view_mode,
      split_view_mode,
      center_view,
      left_panel,
      left_open,
      right_open,
      right_tab,
      theme_mode,
      resolved_theme,
      accent,
      readable_line_width,
      update_manifest_url,
      allowed_origins,
      bookmarks,
      summaries,
      bodies,
      views,
      word_count,
      command_palette_open,
      quick_switcher_open,
      settings_open,
      name_modal,
      confirm_delete,
      status_message,
      session_mode,
      remote_status,
      graph,
      search_query_text,
      search_hits,
      tags,
      backlinks,
      update_result,
      scroll_line,
      projects,
      projects_open,
      project_name,
      open_projects: () => set_projects_open(true),
      close_projects: () => set_projects_open(false),
      open_project,
      close_project,
      forget_project,
      ask_rename_project,
      import_files,
      import_folder,
      export_project,
      export_path,
      export_current_note,
      open_vault_dialog,
      create_vault_dialog,
      open_note,
      close_tab,
      change_body,
      ask_create_note,
      ask_create_folder,
      ask_rename,
      submit_name,
      close_name_modal: () => set_name_modal(null),
      ask_delete: set_confirm_delete,
      confirm_delete_path,
      close_confirm: () => set_confirm_delete(null),
      toggle_bookmark,
      open_daily,
      set_left_panel: (panel) => {
        set_left_open(true);
        set_left_panel(panel);
      },
      toggle_left: () => set_left_open((open) => !open),
      toggle_right: () => set_right_open((open) => !open),
      toggle_split: () => {
        set_split_open((open) => {
          const next = !open;
          if (next && !split_path && active_path) {
            set_split_path(active_path);
          }
          return next;
        });
      },
      toggle_view_mode,
      set_pane_mode: (pane, mode) => {
        if (pane === "split") {
          set_split_view_mode(mode);
        } else {
          set_main_view_mode(mode);
        }
        set_focused_pane(pane);
      },
      set_focused_pane,
      set_center_view: (view) => {
        if (view === "graph") {
          void show_graph();
        } else {
          set_center_view(view);
        }
      },
      set_right_tab,
      set_theme_mode,
      set_accent,
      set_readable_line_width,
      set_update_manifest_url,
      save_update_url,
      check_updates,
      install_update,
      run_search,
      open_link,
      save_properties,
      open_command_palette: () => set_command_palette_open(true),
      close_command_palette: () => set_command_palette_open(false),
      open_quick_switcher: () => set_quick_switcher_open(true),
      close_quick_switcher: () => set_quick_switcher_open(false),
      open_settings: () => set_settings_open(true),
      close_settings: () => set_settings_open(false),
      dismiss_status: () => set_status_message(""),
      scroll_to_line: (line) => {
        set_scroll_line(line);
        if (focused_pane === "split") {
          set_split_view_mode("source");
        } else {
          set_main_view_mode("source");
        }
      },
      select_dir: set_selected_dir,
      selected_dir,
    }),
    [
      ready,
      vault_path,
      tree,
      tabs,
      active_path,
      split_path,
      split_open,
      focused_pane,
      focused_path,
      main_view_mode,
      split_view_mode,
      center_view,
      left_panel,
      left_open,
      right_open,
      right_tab,
      theme_mode,
      resolved_theme,
      accent,
      readable_line_width,
      update_manifest_url,
      allowed_origins,
      bookmarks,
      summaries,
      bodies,
      views,
      word_count,
      command_palette_open,
      quick_switcher_open,
      settings_open,
      name_modal,
      confirm_delete,
      status_message,
      session_mode,
      remote_status,
      graph,
      search_query_text,
      search_hits,
      tags,
      backlinks,
      update_result,
      scroll_line,
      selected_dir,
      projects,
      projects_open,
      project_name,
    ],
  );

  return <AppContext.Provider value={api_value}>{children}</AppContext.Provider>;
}
