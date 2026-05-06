import SettingsPanel from "./components/SettingsPanel";
import "./styles/global.css";

function App() {
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
