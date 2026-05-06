// Preload script para KoTauri
// Usa o sistema de eventos que já possui permissão nativa no core

window.kotauri = {
  openSettings: () => {
    // Usamos o dispatchEvent do próprio navegador como ponte se necessário, 
    // ou o objeto __TAURI__ se ele estiver disponível no contexto de preload
    if (window.__TAURI__ && window.__TAURI__.event) {
        window.__TAURI__.event.emit('open-settings');
    } else {
        console.error('KoTauri: Tauri Event API not found in preload context');
    }
  }
};
