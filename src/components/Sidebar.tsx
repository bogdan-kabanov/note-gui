import { IconPlus } from "./Icons";
import { FileTree } from "./FileTree";
import { use_app } from "../state/use_app";

export function Sidebar() {
  const app = use_app();
  if (!app.left_open) {
    return <aside className="sidebar" />;
  }
  return (
    <aside className="sidebar">
      {app.left_panel === "files" ? (
        <>
          <div className="panel-header">
            <h2>Файлы</h2>
            <button type="button" className="icon-button" title="Новая заметка" onClick={app.ask_create_note}>
              <IconPlus />
            </button>
            <button type="button" className="text-button" onClick={app.ask_create_folder}>
              Папка
            </button>
          </div>
          {app.vault_path ? <FileTree /> : <p className="empty-copy">Хранилище не открыто</p>}
        </>
      ) : null}
      {app.left_panel === "search" ? <SearchPanel /> : null}
      {app.left_panel === "bookmarks" ? <BookmarksPanel /> : null}
    </aside>
  );
}

function SearchPanel() {
  const app = use_app();
  return (
    <>
      <div className="panel-header">
        <h2>Поиск</h2>
      </div>
      <div className="search-wrap">
        <input
          className="search-box"
          value={app.search_query_text}
          placeholder="Найти в заметках"
          onChange={(event) => void app.run_search(event.target.value)}
        />
      </div>
      <div className="panel-body">
        {app.search_hits.map((hit) => (
          <div key={`${hit.path}:${hit.line}`}>
            <button type="button" className="result-row" onClick={() => void app.open_note(hit.path)}>
              <span className="result-title">{hit.title}</span>
              <span className="meta">{hit.path}:{hit.line}</span>
            </button>
            <div className="snippet">{hit.snippet}</div>
          </div>
        ))}
        {app.search_query_text && app.search_hits.length === 0 ? <p className="empty-copy">Ничего не найдено</p> : null}
      </div>
    </>
  );
}

function BookmarksPanel() {
  const app = use_app();
  return (
    <>
      <div className="panel-header">
        <h2>Закладки</h2>
      </div>
      <div className="panel-body">
        {app.bookmarks.length === 0 ? <p className="empty-copy">Закладок пока нет</p> : null}
        {app.bookmarks.map((path) => (
          <button key={path} type="button" className="bookmark-row" onClick={() => void app.open_note(path)}>
            <span className="bookmark-name">{path.replace(/\.md$/i, "")}</span>
          </button>
        ))}
      </div>
    </>
  );
}
