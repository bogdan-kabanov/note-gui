import { useState } from "react";
import { use_app } from "../state/use_app";
import type { tree_node } from "../types";

export function FileTree() {
  const app = use_app();
  const [expanded, set_expanded] = useState<Record<string, boolean>>({});
  const [menu, set_menu] = useState<{ x: number; y: number; node: tree_node } | null>(null);

  function toggle_dir(path: string) {
    set_expanded((current) => ({ ...current, [path]: current[path] === false }));
    app.select_dir(path);
  }

  function render_nodes(nodes: tree_node[]) {
    return nodes.map((node) => {
      const is_open = expanded[node.path] !== false;
      const active = node.is_dir ? app.selected_dir === node.path : app.focused_path === node.path;
      return (
        <div key={node.path}>
          <div
            className={active ? "tree-row active" : "tree-row"}
            onContextMenu={(event) => {
              event.preventDefault();
              set_menu({ x: event.clientX, y: event.clientY, node });
            }}
          >
            <button
              type="button"
              className="tree-row"
              onClick={() => {
                if (node.is_dir) {
                  toggle_dir(node.path);
                } else {
                  void app.open_note(node.path);
                }
              }}
            >
              <span>{node.is_dir ? (is_open ? "▾" : "▸") : "•"}</span>
              <span className="tree-name">{node.name.replace(/\.md$/i, "")}</span>
            </button>
          </div>
          {node.is_dir && is_open ? <div className="tree-children">{render_nodes(node.children)}</div> : null}
        </div>
      );
    });
  }

  return (
    <div className="panel-body" onClick={() => set_menu(null)}>
      {app.tree.length === 0 ? <p className="empty-copy">В хранилище пока нет заметок</p> : render_nodes(app.tree)}
      {menu ? (
        <div className="context-menu" style={{ left: menu.x, top: menu.y }}>
          <button
            type="button"
            className="menu-item"
            onClick={() => {
              app.ask_rename(menu.node.path);
              set_menu(null);
            }}
          >
            Переименовать
          </button>
          <button
            type="button"
            className="menu-item"
            onClick={() => {
              void app.export_path(menu.node.path);
              set_menu(null);
            }}
          >
            Экспортировать
          </button>
          <button
            type="button"
            className="menu-item"
            onClick={() => {
              app.ask_delete(menu.node.path);
              set_menu(null);
            }}
          >
            Удалить
          </button>
        </div>
      ) : null}
    </div>
  );
}
