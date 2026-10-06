import { use_app } from "../state/use_app";
import { IconBookmark, IconDaily, IconFiles, IconGraph, IconProjects, IconSearch, IconSettings } from "./Icons";

export function Ribbon() {
  const app = use_app();
  return (
    <nav className="ribbon">
      <button
        type="button"
        className={app.left_open && app.left_panel === "files" ? "ribbon-button active" : "ribbon-button"}
        title="Файлы"
        onClick={() => app.set_left_panel("files")}
      >
        <IconFiles />
      </button>
      <button
        type="button"
        className={app.left_open && app.left_panel === "search" ? "ribbon-button active" : "ribbon-button"}
        title="Поиск"
        onClick={() => app.set_left_panel("search")}
      >
        <IconSearch />
      </button>
      <button
        type="button"
        className={app.left_open && app.left_panel === "bookmarks" ? "ribbon-button active" : "ribbon-button"}
        title="Закладки"
        onClick={() => app.set_left_panel("bookmarks")}
      >
        <IconBookmark />
      </button>
      <button
        type="button"
        className={app.center_view === "graph" ? "ribbon-button active" : "ribbon-button"}
        title="Граф"
        onClick={() => app.set_center_view("graph")}
      >
        <IconGraph />
      </button>
      <button type="button" className="ribbon-button" title="Ежедневная заметка" onClick={() => void app.open_daily()}>
        <IconDaily />
      </button>
      <div className="ribbon-spacer" />
      <button
        type="button"
        className={app.projects_open ? "ribbon-button active" : "ribbon-button"}
        title="Проекты"
        onClick={app.open_projects}
      >
        <IconProjects />
      </button>
      <button type="button" className="ribbon-button" title="Настройки" onClick={app.open_settings}>
        <IconSettings />
      </button>
    </nav>
  );
}
