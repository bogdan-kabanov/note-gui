export type theme_mode = "system" | "light" | "dark";

export type app_settings = {
  theme_mode: theme_mode;
  accent: string;
  readable_line_width: boolean;
  update_manifest_url: string;
  last_vault_path: string;
  allowed_origins: string[];
  projects: vault_project[];
};

export type vault_project = {
  name: string;
  path: string;
};

export type transfer_result = {
  copied: string[];
  skipped: string[];
};

export type account_session = {
  mode: string;
  account_id: string | null;
  vault_id: string | null;
  access_token: string | null;
  remote_status: string;
};

export type tree_node = {
  name: string;
  path: string;
  is_dir: boolean;
  children: tree_node[];
};

export type heading = {
  level: number;
  text: string;
  line: number;
};

export type note_view = {
  path: string;
  body: string;
  title: string;
  property_title: string;
  property_tags: string[];
  tags: string[];
  aliases: string[];
  headings: heading[];
  word_count: number;
};

export type search_hit = {
  path: string;
  title: string;
  line: number;
  snippet: string;
};

export type backlink = {
  source_path: string;
  source_title: string;
  alias: string | null;
};

export type tag_info = {
  tag: string;
  count: number;
  paths: string[];
};

export type note_summary = {
  path: string;
  title: string;
};

export type graph_data = {
  nodes: { path: string; title: string }[];
  edges: { source_path: string; target_path: string }[];
};

export type update_check_result = {
  status: string;
  message: string;
  version: string | null;
  notes: string | null;
};

export type name_modal_state = {
  title: string;
  label: string;
  initial: string;
  kind: "note" | "folder" | "rename" | "vault" | "project";
  parent_path: string;
  target_path: string;
  vault_parent: string;
};

export type left_panel = "files" | "search" | "bookmarks";
export type right_tab = "outline" | "backlinks" | "tags";
export type view_mode = "source" | "preview";
export type focused_pane = "main" | "split";
export type center_view = "editor" | "graph";
