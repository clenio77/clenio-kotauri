import { invoke } from "@tauri-apps/api/core";
import SettingsPanel from "./components/SettingsPanel";
import WebShell from "./components/WebShell";
import "./styles/global.css";

function isTauriRuntime() {
  if (typeof window === "undefined") return false;
  return "__TAURI_INTERNALS__" in window;
}

function App() {
  if (!isTauriRuntime()) {
    return <WebShell />;
  }

  const handleSettingsChange = () => {
    console.log("Settings updated");
  };

  const handleClose = async () => {
    try {
      await invoke("hide_settings");
    } catch (e) {
      console.error("hide_settings failed:", e);
    }
  };

  return (
    <div className="app-container">
      <SettingsPanel
        onClose={handleClose}
        onSettingsChange={handleSettingsChange}
      />
    </div>
  );
}

export default App;
