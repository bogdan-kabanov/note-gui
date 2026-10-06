import { app_version } from "../app_version";
import { use_app } from "../state/use_app";

export function StatusBar() {
  const app = use_app();
  const theme_label = app.resolved_theme === "dark" ? "Тёмная" : "Светлая";
  return (
    <footer className="status-bar">
      <span>{app.vault_path ?? "Хранилище не открыто"}</span>
      <span>{app.status_message}</span>
      <span>{app.focused_path ? `${app.word_count} сл.` : ""}</span>
      <span className="spacer" />
      <span>{theme_label}</span>
      <span>{app_version}</span>
    </footer>
  );
}
