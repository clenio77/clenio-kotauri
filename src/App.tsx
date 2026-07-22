import SettingsPanel from "./components/SettingsPanel";
import "./styles/global.css";

const TELEGRAM_WEB_K_URL = "https://web.telegram.org/k/";

function isTauriRuntime() {
  if (typeof window === "undefined") return false;
  return "__TAURI_INTERNALS__" in window;
}

function App() {
  if (!isTauriRuntime()) {
    return (
      <main className="web-shell" data-testid="web-shell">
        <h1>KoTauri Web</h1>
        <p className="web-shell-description">
          Open Telegram Web K and install this app from your browser menu for a
          full-screen experience on phone.
        </p>
        <a
          className="web-shell-cta"
          href={TELEGRAM_WEB_K_URL}
          data-testid="open-telegram-web-k"
        >
          Open Telegram Web K
        </a>
        <p className="web-shell-install-tip">
          Install tip: in Chrome/Edge use "Add to Home screen" or "Install app"
          from the browser menu.
        </p>
      </main>
    );
  }

  const handleSettingsChange = () => {
    // In Tauri, the main window will refresh its injections 
    // when it gains focus, so we don't need explicit logic here.
    console.log("Settings updated");
  };

  return (
    <div className="app-container">
      <SettingsPanel
        onClose={() => {
          // The window close button handles this
        }}
        onSettingsChange={handleSettingsChange}
      />
    </div>
  );
}

export default App;
