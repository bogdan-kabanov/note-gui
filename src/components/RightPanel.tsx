import { useEffect, useState } from "react";
import { use_app } from "../state/use_app";

export function RightPanel() {
  const app = use_app();
  const view = app.focused_path ? app.views[app.focused_path] : undefined;
  const [title, set_title] = useState("");
  const [tags_text, set_tags_text] = useState("");
  const [aliases_text, set_aliases_text] = useState("");

  useEffect(() => {
    set_title(view?.property_title ?? "");
    set_tags_text((view?.property_tags ?? []).join(", "));
    set_aliases_text((view?.aliases ?? []).join(", "));
  }, [view?.path, view?.property_title, view?.property_tags, view?.aliases]);

  if (!app.right_open) {
    return <aside className="right-panel" />;
  }

  function save_if_needed() {
    if (!app.focused_path || !view) {
      return;
    }
    const tags = split_list(tags_text);
    const aliases = split_list(aliases_text);
    const same_title = title === view.property_title;
    const same_tags = tags.join("|") === view.property_tags.join("|");
    const same_aliases = aliases.join("|") === view.aliases.join("|");
    if (same_title && same_tags && same_aliases) {
      return;
    }
    void app.save_properties(app.focused_path, title, tags, aliases);
  }

  return (
    <aside className="right-panel">
      <div className="properties" onBlur={save_if_needed}>
        <label className="field">
          <span>Название</span>
          <input value={title} onChange={(event) => set_title(event.target.value)} />
        </label>
        <label className="field">
          <span>Теги</span>
          <input value={tags_text} onChange={(event) => set_tags_text(event.target.value)} placeholder="через запятую" />
        </label>
        <label className="field">
          <span>Псевдонимы</span>
          <input value={aliases_text} onChange={(event) => set_aliases_text(event.target.value)} placeholder="через запятую" />
        </label>
      </div>
      <div className="right-tabs">
        <button type="button" className={app.right_tab === "outline" ? "active" : ""} onClick={() => app.set_right_tab("outline")}>
          Оглавление
        </button>
        <button type="button" className={app.right_tab === "backlinks" ? "active" : ""} onClick={() => app.set_right_tab("backlinks")}>
          Ссылки
        </button>
        <button type="button" className={app.right_tab === "tags" ? "active" : ""} onClick={() => app.set_right_tab("tags")}>
          Теги
        </button>
      </div>
      <div className="panel-body">
        {app.right_tab === "outline"
          ? (view?.headings ?? []).map((heading) => (
              <button
                key={`${heading.line}:${heading.text}`}
                type="button"
                className="outline-item"
                style={{ paddingLeft: 8 + (heading.level - 1) * 12 }}
                onClick={() => app.scroll_to_line(heading.line)}
              >
                {heading.text}
              </button>
            ))
          : null}
        {app.right_tab === "backlinks"
          ? app.backlinks.map((link) => (
              <button key={link.source_path} type="button" className="result-row" onClick={() => void app.open_note(link.source_path)}>
                <span className="result-title">{link.source_title}</span>
                {link.alias ? <span className="meta">{link.alias}</span> : null}
              </button>
            ))
          : null}
        {app.right_tab === "backlinks" && app.backlinks.length === 0 ? <p className="empty-copy">Обратных ссылок нет</p> : null}
        {app.right_tab === "tags"
          ? app.tags.map((tag) => (
              <div key={tag.tag}>
                <div className="tag-row">
                  <span>#{tag.tag}</span>
                  <span className="tag-count">{tag.count}</span>
                </div>
                {tag.paths.map((path) => (
                  <button key={path} type="button" className="result-row" onClick={() => void app.open_note(path)}>
                    <span className="result-title">{path.replace(/\.md$/i, "")}</span>
                  </button>
                ))}
              </div>
            ))
          : null}
        {app.right_tab === "outline" && (view?.headings.length ?? 0) === 0 ? <p className="empty-copy">Заголовков нет</p> : null}
      </div>
    </aside>
  );
}

function split_list(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}
