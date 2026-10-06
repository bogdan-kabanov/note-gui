import { useEffect, useState } from "react";
import { app_version } from "../app_version";
import { use_app } from "../state/use_app";
import type { note_summary } from "../types";

export function Overlays() {
  const app = use_app();
  return (
    <>
      {app.command_palette_open ? <CommandPalette /> : null}
      {app.quick_switcher_open ? <QuickSwitcher /> : null}
      {app.settings_open ? <SettingsModal /> : null}
      {app.projects_open ? <ProjectsModal /> : null}
      {app.name_modal ? <NameModal /> : null}
      {app.confirm_delete ? (
        <div className="overlay">
          <div className="modal">
            <h2>Удалить?</h2>
            <p>{app.confirm_delete}</p>
            <div className="modal-actions">
              <button type="button" className="danger-button" onClick={() => void app.confirm_delete_path()}>
                Удалить
              </button>
              <button type="button" className="text-button" onClick={app.close_confirm}>
                Отмена
              </button>
            </div>
          </div>
        </div>
      ) : null}
    </>
  );
}

function CommandPalette() {
  const app = use_app();
  const [query, set_query] = useState("");
  const commands = [
    { id: "switcher", title: "Быстрый переход", run: app.open_quick_switcher },
    { id: "note", title: "Новая заметка", run: app.ask_create_note },
    { id: "folder", title: "Новая папка", run: app.ask_create_folder },
    { id: "daily", title: "Ежедневная заметка", run: () => void app.open_daily() },
    { id: "preview", title: "Редактор / чтение", run: app.toggle_view_mode },
    { id: "split", title: "Разделить редактор", run: app.toggle_split },
    { id: "graph", title: "Граф связей", run: () => app.set_center_view("graph") },
    { id: "search", title: "Поиск", run: () => app.set_left_panel("search") },
    { id: "bookmarks", title: "Закладки", run: () => app.set_left_panel("bookmarks") },
    { id: "files", title: "Файлы", run: () => app.set_left_panel("files") },
    { id: "projects", title: "Проекты", run: app.open_projects },
    { id: "import_files", title: "Импорт файлов", run: () => void app.import_files() },
    { id: "import_folder", title: "Импорт папки", run: () => void app.import_folder() },
    { id: "export_project", title: "Экспорт проекта", run: () => void app.export_project() },
    { id: "export_note", title: "Экспорт текущей заметки", run: () => void app.export_current_note() },
    { id: "editor", title: "Вернуться к редактору", run: () => app.set_center_view("editor") },
    { id: "left", title: "Показать или скрыть левую панель", run: app.toggle_left },
    { id: "right", title: "Показать или скрыть правую панель", run: app.toggle_right },
    { id: "light", title: "Светлая тема", run: () => void app.set_theme_mode("light") },
    { id: "dark", title: "Тёмная тема", run: () => void app.set_theme_mode("dark") },
    { id: "system", title: "Тема как в системе", run: () => void app.set_theme_mode("system") },
    { id: "settings", title: "Настройки", run: app.open_settings },
    { id: "updates", title: "Проверить обновления", run: () => void app.check_updates() },
    {
      id: "close",
      title: "Закрыть вкладку",
      run: () => {
        if (app.focused_path) {
          app.close_tab(app.focused_path);
        }
      },
    },
  ].filter((command) => command.title.toLowerCase().includes(query.trim().toLowerCase()));
  return (
    <div className="overlay" onMouseDown={app.close_command_palette}>
      <div className="palette" onMouseDown={(event) => event.stopPropagation()}>
        <input autoFocus placeholder="Команда" value={query} onChange={(event) => set_query(event.target.value)} />
        <div className="command-list">
          {commands.map((command) => (
            <button
              key={command.id}
              type="button"
              className="command-row"
              onClick={() => {
                command.run();
                app.close_command_palette();
              }}
            >
              {command.title}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function QuickSwitcher() {
  const app = use_app();
  const [query, set_query] = useState("");
  const [active_index, set_active_index] = useState(0);
  const items = app.summaries.filter((item) => {
    const needle = query.trim().toLowerCase();
    return item.title.toLowerCase().includes(needle) || item.path.toLowerCase().includes(needle);
  });
  useEffect(() => {
    set_active_index(0);
  }, [query]);
  function open_item(item: note_summary) {
    void app.open_note(item.path);
    app.close_quick_switcher();
  }
  return (
    <div className="overlay" onMouseDown={app.close_quick_switcher}>
      <div className="palette" onMouseDown={(event) => event.stopPropagation()}>
        <input
          autoFocus
          placeholder="Перейти к заметке"
          value={query}
          onChange={(event) => set_query(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "ArrowDown") {
              event.preventDefault();
              set_active_index((index) => Math.min(index + 1, Math.max(items.length - 1, 0)));
            } else if (event.key === "ArrowUp") {
              event.preventDefault();
              set_active_index((index) => Math.max(index - 1, 0));
            } else if (event.key === "Enter" && items[active_index]) {
              open_item(items[active_index]);
            }
          }}
        />
        <div className="switcher-list">
          {items.map((item, index) => (
            <button
              key={item.path}
              type="button"
              className={index === active_index ? "command-row active" : "command-row"}
              onClick={() => open_item(item)}
            >
              <span className="result-title">{item.title}</span>
              <span className="meta">{item.path}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function ProjectsModal() {
  const app = use_app();
  return (
    <div className="overlay" onMouseDown={app.close_projects}>
      <div className="modal wide" onMouseDown={(event) => event.stopPropagation()}>
        <h2>Проекты</h2>
        <p>Каждый проект — отдельная папка с заметками. Файлы на диске не удаляются, когда проект убирают из списка.</p>
        <div className="project-list">
          {app.projects.length === 0 ? <p className="empty-copy">Пока нет сохранённых проектов</p> : null}
          {app.projects.map((project) => {
            const current =
              app.vault_path !== null &&
              project.path.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase() ===
                app.vault_path.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
            return (
              <div key={project.path} className="project-line">
                <button
                  type="button"
                  className={current ? "project-row active" : "project-row"}
                  onClick={() => void app.open_project(project.path)}
                >
                  <span className="result-title">{project.name}</span>
                  <span className="meta">{project.path}</span>
                </button>
                <button type="button" className="text-button" onClick={() => app.ask_rename_project(project)}>
                  Имя
                </button>
                <button type="button" className="text-button" onClick={() => void app.forget_project(project.path)}>
                  Убрать
                </button>
              </div>
            );
          })}
        </div>
        <div className="modal-actions">
          <button type="button" className="primary-button" onClick={() => void app.open_vault_dialog()}>
            Открыть папку
          </button>
          <button type="button" className="text-button" onClick={() => void app.create_vault_dialog()}>
            Создать
          </button>
          {app.vault_path ? (
            <button type="button" className="text-button" onClick={() => void app.close_project()}>
              Закрыть текущий
            </button>
          ) : null}
          <button type="button" className="text-button" onClick={app.close_projects}>
            Закрыть
          </button>
        </div>
      </div>
    </div>
  );
}

function SettingsModal() {
  const app = use_app();
  return (
    <div className="overlay" onMouseDown={app.close_settings}>
      <div className="modal" onMouseDown={(event) => event.stopPropagation()}>
        <h2>Настройки</h2>
        <label className="field">
          <span>Тема</span>
          <select
            value={app.theme_mode}
            onChange={(event) => void app.set_theme_mode(event.target.value as "system" | "light" | "dark")}
          >
            <option value="system">Как в системе</option>
            <option value="light">Светлая</option>
            <option value="dark">Тёмная</option>
          </select>
        </label>
        <label className="field">
          <span>Акцент</span>
          <input type="color" value={app.accent} onChange={(event) => void app.set_accent(event.target.value)} />
        </label>
        <label className="field">
          <span>Читаемая ширина строки</span>
          <input
            type="checkbox"
            checked={app.readable_line_width}
            onChange={(event) => void app.set_readable_line_width(event.target.checked)}
          />
        </label>
        <label className="field">
          <span>Адрес latest.json</span>
          <input
            value={app.update_manifest_url}
            placeholder="https://gitlab.com/group/note-gui/-/releases/permalink/latest/downloads/latest.json"
            onChange={(event) => app.set_update_manifest_url(event.target.value)}
            onBlur={() => void app.save_update_url()}
          />
        </label>
        <p className="session-line">Режим: локальный ({app.session_mode})</p>
        <p className="session-line">{app.remote_status}</p>
        <p className="session-line">Версия {app_version}</p>
        {app.update_result ? <p className="session-line">{app.update_result.message}</p> : null}
        <div className="modal-actions">
          <button type="button" className="text-button" onClick={() => void app.check_updates()}>
            Проверить обновления
          </button>
          {app.update_result?.status === "available" ? (
            <button type="button" className="primary-button" onClick={() => void app.install_update()}>
              Установить
            </button>
          ) : null}
          <button type="button" className="text-button" onClick={app.close_settings}>
            Закрыть
          </button>
        </div>
      </div>
    </div>
  );
}

function NameModal() {
  const app = use_app();
  const modal = app.name_modal;
  const [name, set_name] = useState(modal?.initial ?? "");
  useEffect(() => {
    set_name(modal?.initial ?? "");
  }, [modal]);
  if (!modal) {
    return null;
  }
  return (
    <div className="overlay" onMouseDown={app.close_name_modal}>
      <form
        className="modal"
        onMouseDown={(event) => event.stopPropagation()}
        onSubmit={(event) => {
          event.preventDefault();
          void app.submit_name(name);
        }}
      >
        <h2>{modal.title}</h2>
        <label className="field">
          <span>{modal.label}</span>
          <input autoFocus value={name} onChange={(event) => set_name(event.target.value)} />
        </label>
        <div className="modal-actions">
          <button type="submit" className="primary-button">
            {modal.kind === "rename" || modal.kind === "project" ? "Сохранить" : "Создать"}
          </button>
          <button type="button" className="text-button" onClick={app.close_name_modal}>
            Отмена
          </button>
        </div>
      </form>
    </div>
  );
}
