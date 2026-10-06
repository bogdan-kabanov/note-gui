import { AppShell } from "./components/AppShell";
import { AppProvider } from "./state/app_context";
import "./styles/theme.css";
import "./styles/app.css";

export default function App() {
  return (
    <AppProvider>
      <AppShell />
    </AppProvider>
  );
}
