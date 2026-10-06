import { use_app } from "../state/use_app";
import { GraphView } from "./GraphView";
import { IconBookmark, IconClose } from "./Icons";
import { MarkdownEditor } from "./MarkdownEditor";
import { MarkdownPreview } from "./MarkdownPreview";

export function EditorArea() {
  const app = use_app();
  if (!app.vault_path) {
    return (
      <section className="workspace">
        <EmptyVault />
      </section>
    );
  }
  if (app.center_view === "graph") {
    return (
      <section className="workspace">
        <GraphView />
      </section>
    );
  }
  return (
    <section className="workspace">
      <div className="tab-bar">
        {app.tabs.map((path) => (
          <div key={path} className={path === app.focused_path ? "tab active" : "tab"}>
            <button type="button" onClick={() => void app.open_note(path)}>
              <span>{path.split("/").pop()?.replace(/\.md$/i, "")}</span>
            </button>
            <button type="button" className="tab-close" onClick={() => app.close_tab(path)} title="Закрыть">
              <IconClose size={14} />
            </button>
          </div>
        ))}
      </div>
      {app.status_message ? (
        <div className="banner">
          <span>{app.status_message}</span>
          <button type="button" className="text-button" onClick={app.dismiss_status}>
            Скрыть
          </button>
        </div>
      ) : null}
      <div className={app.split_open ? "editor-split split" : "editor-split"}>
        <NotePane pane="main" />
        {app.split_open ? <NotePane pane="split" /> : null}
      </div>
    </section>
  );
}

function NotePane({ pane }: { pane: "main" | "split" }) {
  const app = use_app();
  const path = pane === "main" ? app.active_path : app.split_path;
  const mode = pane === "main" ? app.main_view_mode : app.split_view_mode;
  const body = path ? (app.bodies[path] ?? "") : "";
  const bookmarked = path ? app.bookmarks.includes(path) : false;
  return (
    <div
      className={app.focused_pane === pane ? "note-pane focused" : "note-pane"}
      onMouseDown={() => app.set_focused_pane(pane)}
    >
      <div className="editor-toolbar">
        <strong>{path ? path.replace(/\.md$/i, "") : "Нет заметки"}</strong>
        <div className="toolbar-actions">
          <button type="button" className={mode === "source" ? "text-button active" : "text-button"} onClick={() => app.set_pane_mode(pane, "source")}>
            Редактор
          </button>
          <button type="button" className={mode === "preview" ? "text-button active" : "text-button"} onClick={() => app.set_pane_mode(pane, "preview")}>
            Чтение
          </button>
          <button type="button" className="text-button" disabled={!path} onClick={() => void app.export_current_note()}>
            Экспорт
          </button>
          <button
            type="button"
            className={bookmarked ? "icon-button active" : "icon-button"}
            title="Закладка"
            disabled={!path}
            onClick={() => path && void app.toggle_bookmark(path)}
          >
            <IconBookmark size={16} />
          </button>
        </div>
      </div>
      {path ? (
        <div className={app.readable_line_width ? "readable panel-body" : "panel-body"}>
          {mode === "source" ? (
            <MarkdownEditor
              path={path}
              body={body}
              dark={app.resolved_theme === "dark"}
              on_change={app.change_body}
              on_open_link={(target) => void app.open_link(target)}
              scroll_line={app.scroll_line}
            />
          ) : (
            <MarkdownPreview body={body} on_open_link={(target) => void app.open_link(target)} />
          )}
        </div>
      ) : (
        <p className="empty-copy">Откройте заметку или создайте новую</p>
      )}
    </div>
  );
}

function EmptyVault() {
  const app = use_app();
  return (
    <div className="empty-vault">
      <div className={app.projects.length > 0 ? "empty-card wide" : "empty-card"}>
        <h1>Записная книжка</h1>
        <p>Откройте папку с markdown-заметками или создайте новый проект на диске.</p>
        {app.projects.length > 0 ? (
          <div className="project-list">
            {app.projects.map((project) => (
              <button key={project.path} type="button" className="project-row" onClick={() => void app.open_project(project.path)}>
                <span className="result-title">{project.name}</span>
                <span className="meta">{project.path}</span>
              </button>
            ))}
          </div>
        ) : null}
        <div className="modal-actions">
          <button type="button" className="primary-button" onClick={() => void app.open_vault_dialog()}>
            Открыть папку
          </button>
          <button type="button" className="text-button" onClick={() => void app.create_vault_dialog()}>
            Создать проект
          </button>
        </div>
      </div>
    </div>
  );
}
