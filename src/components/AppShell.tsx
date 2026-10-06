import { use_app } from "../state/use_app";
import { EditorArea } from "./EditorArea";
import { Overlays } from "./Overlays";
import { Ribbon } from "./Ribbon";
import { RightPanel } from "./RightPanel";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";

export function AppShell() {
  const app = use_app();
  if (!app.ready) {
    return <div className="loading-screen">Загрузка…</div>;
  }
  const class_name = [
    "app-shell",
    app.left_open ? "" : "left-collapsed",
    app.right_open ? "" : "right-collapsed",
  ]
    .filter(Boolean)
    .join(" ");
  return (
    <div className={class_name}>
      <Ribbon />
      <Sidebar />
      <EditorArea />
      <RightPanel />
      <StatusBar />
      <Overlays />
    </div>
  );
}
