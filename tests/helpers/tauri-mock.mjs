import { DEFAULT_SETTINGS } from "./defaults.mjs";

/**
 * Injeta mock de Tauri 2 antes dos scripts da página.
 * SettingsPanel usa invoke() + window.initialSettings.
 */
export async function installTauriMock(page, settings = DEFAULT_SETTINGS) {
  await page.evaluateOnNewDocument((initial) => {
    const state = structuredClone(initial);
    const updates = [];

    window.__kotauriTest = { settings: state, updates };

    window.__TAURI_INTERNALS__ = {
      invoke(cmd, args = {}) {
        if (cmd === "get_settings") {
          return Promise.resolve(JSON.stringify(state));
        }
        if (cmd === "update_setting") {
          const key = args.key;
          let value = args.value;
          if (value === "true") value = true;
          if (value === "false") value = false;
          state[key] = value;
          updates.push({ key, value });
          window.__kotauriTest.settings = state;
          window.__kotauriTest.updates = updates;
          return Promise.resolve(null);
        }
        if (cmd === "open_settings") {
          return Promise.resolve(null);
        }
        return Promise.reject(new Error(`[kotauri-e2e] comando não mockado: ${cmd}`));
      },
      transformCallback(callback) {
        return callback;
      },
      unregisterCallback() {},
    };

    window.initialSettings = structuredClone(initial);
  }, settings);
}

export async function readMockUpdates(page) {
  return page.evaluate(() => window.__kotauriTest?.updates ?? []);
}

export async function readMockSettings(page) {
  return page.evaluate(() => window.__kotauriTest?.settings ?? null);
}
