// KoTauri — Settings Manager
// Manages user preferences stored as JSON

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::web_selectors::{injected_compat_warning_js, WebKSelectors};

#[derive(Debug)]
pub struct AppSettings {
    pub inner: Mutex<SettingsData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingsData {
    // Appearance
    pub custom_font: String,
    pub font_size: u32,
    pub compact_mode: bool,
    pub adaptive_bubbles: bool,
    pub sticker_height: u32,
    pub big_emoji_outline: bool,
    pub theme: String,
    pub sidebar_folders: bool,

    // Chat
    pub show_chat_id: bool,
    pub disable_up_edit: bool,
    pub always_show_scheduled: bool,

    // Forward
    pub forward_without_author: bool,
    pub forward_retain_selection: bool,

    // System
    pub minimize_to_tray: bool,
    pub start_minimized: bool,

    // Custom text replaces
    pub text_replaces: Vec<(String, String)>,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            custom_font: String::new(),
            font_size: 14,
            compact_mode: false,
            adaptive_bubbles: false,
            sticker_height: 170,
            big_emoji_outline: true,
            theme: "default".to_string(),
            sidebar_folders: true,
            show_chat_id: false,
            disable_up_edit: false,
            always_show_scheduled: false,
            forward_without_author: false,
            forward_retain_selection: false,
            minimize_to_tray: true,
            start_minimized: false,
            text_replaces: vec![],
        }
    }
}

impl AppSettings {
    fn config_path() -> std::path::PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("kotauri");
        std::fs::create_dir_all(&config_dir).ok();
        config_dir.join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        let data = if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str::<SettingsData>(&content).unwrap_or_default()
        } else {
            let defaults = SettingsData::default();
            // Write defaults on first run
            if let Ok(json) = serde_json::to_string_pretty(&defaults) {
                std::fs::write(&path, json).ok();
            }
            defaults
        };

        AppSettings {
            inner: Mutex::new(data),
        }
    }

    pub fn save(&self) {
        if let Ok(data) = self.inner.lock() {
            if let Ok(json) = serde_json::to_string_pretty(&*data) {
                std::fs::write(Self::config_path(), json).ok();
            }
        }
    }

    pub fn update(&self, key: &str, value: &str) {
        if let Ok(mut data) = self.inner.lock() {
            match key {
                "custom_font" => data.custom_font = value.to_string(),
                "font_size" => data.font_size = value.parse().unwrap_or(14),
                "compact_mode" => data.compact_mode = value == "true",
                "adaptive_bubbles" => data.adaptive_bubbles = value == "true",
                "sticker_height" => data.sticker_height = value.parse().unwrap_or(170),
                "big_emoji_outline" => data.big_emoji_outline = value == "true",
                "theme" => data.theme = value.to_string(),
                "show_chat_id" => data.show_chat_id = value == "true",
                "sidebar_folders" => data.sidebar_folders = value == "true",
                "disable_up_edit" => data.disable_up_edit = value == "true",
                "always_show_scheduled" => data.always_show_scheduled = value == "true",
                "forward_without_author" => data.forward_without_author = value == "true",
                "forward_retain_selection" => data.forward_retain_selection = value == "true",
                "minimize_to_tray" => data.minimize_to_tray = value == "true",
                "start_minimized" => data.start_minimized = value == "true",
                _ => {}
            }
        }
    }

    /// Generates CSS to inject into the Telegram WebView
    pub fn generate_css(&self) -> String {
        let data = self
            .inner
            .lock()
            .expect("AppSettings mutex poisoned");
        let mut css = String::new();

        // Custom font: Rolo compressor universal
        if !data.custom_font.is_empty() {
            css.push_str(&format!(
                r#"
                * {{
                    font-family: '{}', system-ui, -apple-system, sans-serif !important;
                }}
                [class^="icon-"], [class*=" icon-"], i, .icon, .tgico {{
                    font-family: 'tgico', 'icomoon' !important;
                }}
                "#,
                data.custom_font
            ));
        }

        // Font size: MODO DEUS
        if data.font_size != 14 {
            css.push_str(&format!(
                "
                * {{
                    font-size: {}px !important;
                }}
                ",
                data.font_size
            ));
        }

        // Compact mode
        if data.compact_mode {
            css.push_str("
                .chatlist-chat, .chat-list-item {
                    height: 56px !important;
                    min-height: 56px !important;
                }
                .chatlist-chat .user-title, .chatlist-chat .dialog-subtitle {
                    font-size: 0.9em !important;
                }
                /* Reduz o avatar no modo compacto */
                .chatlist-chat .avatar-54, .chatlist-chat .row-media-bigger {
                    width: 42px !important;
                    height: 42px !important;
                    min-width: 42px !important;
                    min-height: 42px !important;
                }
                .bubble, .message {
                    padding: 4px 10px !important;
                    margin-bottom: 2px !important;
                }
            ");
        }

        // Adaptive bubbles
        if data.adaptive_bubbles {
            css.push_str("
                .bubble, .message {
                    max-width: 90% !important;
                    width: fit-content !important;
                }
            ");
        }

        // Sticker height
        if data.sticker_height != 170 {
            css.push_str(&format!("
                .sticker-media, .media-sticker, img.sticker, video.media-video {{
                    max-height: {}px !important;
                    max-width: {}px !important;
                }}
            ", data.sticker_height, data.sticker_height));
        }

        // Themes
        if data.theme == "nord" {
            css.push_str("
                :root {
                    --surface-color: #2E3440 !important;
                    --background-color: #2E3440 !important;
                    --primary-color: #88C0D0 !important;
                    --text-color: #ECEFF4 !important;
                    --secondary-text-color: #D8DEE9 !important;
                    --border-color: #4C566A !important;
                    --chat-background-color: #2E3440 !important;
                    --message-out-background-color: #4C566A !important;
                    --message-in-background-color: #3B4252 !important;
                }
            ");
        } else if data.theme == "catppuccin" {
            css.push_str("
                :root {
                    --surface-color: #1e1e2e !important;
                    --background-color: #1e1e2e !important;
                    --primary-color: #cba6f7 !important;
                    --text-color: #cdd6f4 !important;
                    --secondary-text-color: #a6adc8 !important;
                    --border-color: #313244 !important;
                    --chat-background-color: #1e1e2e !important;
                    --message-out-background-color: #89b4fa !important;
                    --message-in-background-color: #313244 !important;
                }
            ");
        } else if data.theme == "midnight" {
            css.push_str("
                :root {
                    --surface-color: #000000 !important;
                    --background-color: #000000 !important;
                    --primary-color: #3390ec !important;
                    --text-color: #ffffff !important;
                    --secondary-text-color: #aaaaaa !important;
                    --border-color: #222222 !important;
                    --chat-background-color: #000000 !important;
                    --message-out-background-color: #1a4f8b !important;
                    --message-in-background-color: #181818 !important;
                }
            ");
        }

        // Sidebar folders (Kotatogram style)
        if data.sidebar_folders {
            let col = WebKSelectors::COLUMN_LEFT;
            let folders_wrap = WebKSelectors::FOLDERS_CONTAINER_UNSTABLE;
            css.push_str(&format!("
                /* 1. Empurra TODOS os containers principais da coluna esquerda para a direita */
                {col} .sidebar-header,
                {col} .chatlist-parts,
                {col} .connection-status-bottom {{
                    transform: translateX(85px) !important;
                    width: calc(100% - 85px) !important;
                }}
                
                /* 2. O container original das abas não é transformado, então ele fica no espaço vazio de 85px! */
                {col} .transition-item {{
                    overflow: visible !important;
                    z-index: 9999 !important;
                }}
                
                {col} {folders_wrap} {{
                    position: absolute !important;
                    left: 0 !important;
                    top: 0 !important;
                    background: transparent !important;
                    z-index: 9999 !important;
                }}
                
                /* 4. As pastas ancoradas fisicamente na esquerda, agindo como máscara visual opaca */
                .folders-tabs-scrollable {{
                    position: fixed !important;
                    left: 0 !important;
                    top: 0 !important;
                    width: 85px !important;
                    height: 100vh !important;
                    flex-direction: column !important;
                    background: #17212b !important;
                    border-right: 1px solid var(--border-color, rgba(0,0,0,0.2)) !important;
                    z-index: 99999 !important;
                    overflow-y: auto !important;
                    scrollbar-width: none !important;
                    border-radius: 0 !important;
                    padding-top: 10px !important;
                    display: flex !important;
                }}
                .folders-tabs-scrollable::-webkit-scrollbar {{ display: none !important; }}
                
                /* 5. Transforma em coluna nativamente */
                .menu-horizontal-div {{
                    display: flex !important;
                    flex-direction: column !important;
                    width: 100% !important;
                }}
                
                /* 6. Estilo dos ícones Kotatogram */
                .menu-horizontal-div-item {{
                    width: 100% !important;
                    height: auto !important;
                    min-height: 64px !important;
                    padding: 8px 4px !important;
                    justify-content: center !important;
                    align-items: center !important;
                    flex-direction: column !important;
                    font-size: 11px !important;
                    flex-shrink: 0 !important;
                    border-radius: 0 !important;
                    text-align: center !important;
                    white-space: pre-wrap !important;
                }}
            "));
        }

        css
    }

    /// Generates JavaScript to inject into the Telegram WebView
    pub fn generate_js(&self) -> String {
        let show_chat_id = {
            let data = self
                .inner
                .lock()
                .expect("AppSettings mutex poisoned");
            data.show_chat_id
        };

        let mut js = String::new();

        if show_chat_id {
            js.push_str(&format!(
                r##"
            (function() {{
                if (!window._kotauri_chat_id_observer) {{
                    const ROOT_SEL = '{chat_root}';
                    const ROW_SELECTOR_LIST = '{chat_rows}';

                    function normalizePeerId(v) {{
                        if (v === null || v === undefined) return null;
                        const s = String(v).trim();
                        return /^-?\d+$/.test(s) ? s : null;
                    }}

                    function peerIdFromHref(href) {{
                        if (!href) return null;
                        const m = String(href).match(/^#(-?\d+)$/);
                        return m ? m[1] : null;
                    }}

                    function tryReactPeerId(el) {{
                        const keys = Object.keys(el);
                        for (let i = 0; i < keys.length; i++) {{
                            const k = keys[i];
                            if (!k.startsWith('__reactFiber$') && !k.startsWith('__reactInternalInstance$')) continue;
                            let fiber = el[k];
                            let depth = 0;
                            while (fiber && depth++ < 48) {{
                                const props = fiber.memoizedProps || fiber.pendingProps;
                                if (props && typeof props === 'object') {{
                                    let pid = normalizePeerId(props.peerId);
                                    if (pid) return pid;
                                    if (props.dialog) {{
                                        pid = normalizePeerId(props.dialog.peerId);
                                        if (pid) return pid;
                                    }}
                                }}
                                fiber = fiber.return;
                            }}
                        }}
                        return null;
                    }}

                    function extractPeerId(row) {{
                        let id = normalizePeerId(row.getAttribute('data-peer-id'));
                        if (id) return id;
                        id = normalizePeerId(row.getAttribute('data-dialog-id'));
                        if (id) return id;
                        const attrEl = row.querySelector('[data-peer-id], [data-dialog-id]');
                        if (attrEl) {{
                            id = normalizePeerId(attrEl.getAttribute('data-peer-id'))
                                || normalizePeerId(attrEl.getAttribute('data-dialog-id'));
                            if (id) return id;
                        }}
                        const links = row.querySelectorAll('a[href^="#"]');
                        for (let i = 0; i < links.length; i++) {{
                            id = peerIdFromHref(links[i].getAttribute('href'));
                            if (id) return id;
                        }}
                        return tryReactPeerId(row);
                    }}

                    function collectChatRows() {{
                        const root = document.querySelector(ROOT_SEL) || document.body;
                        const rows = new Set();
                        ROW_SELECTOR_LIST.split(',').map(function(s) {{ return s.trim(); }}).forEach(function(sel) {{
                            if (!sel) return;
                            try {{
                                root.querySelectorAll(sel).forEach(function(el) {{ rows.add(el); }});
                            }} catch (e) {{}}
                        }});
                        root.querySelectorAll('a[href^="#"]').forEach(function(a) {{
                            const href = a.getAttribute('href') || '';
                            if (!/^#[-]?\d/.test(href)) return;
                            const row = a.closest('.chatlist-chat')
                                || a.closest('.chat-list-item')
                                || a.closest('li.chatlist-chat')
                                || a.closest(ROOT_SEL + ' li');
                            if (row && root.contains(row)) rows.add(row);
                        }});
                        return rows;
                    }}

                    function ensurePositioned(row) {{
                        const cs = window.getComputedStyle(row);
                        if (cs.position === 'static' || cs.position === '') {{
                            row.style.position = 'relative';
                        }}
                    }}

                    function scanChatIds() {{
                        collectChatRows().forEach(function(chat) {{
                            if (chat.querySelector('.kotauri-chat-id')) return;
                            const id = extractPeerId(chat);
                            if (!id) return;
                            ensurePositioned(chat);
                            const span = document.createElement('span');
                            span.className = 'kotauri-chat-id';
                            span.textContent = id;
                            span.style.cssText = 'position:absolute;right:8px;bottom:2px;font-size:10px;line-height:1.2;color:rgba(255,255,255,0.45);z-index:10;pointer-events:none;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;letter-spacing:0.02em;';
                            chat.appendChild(span);
                        }});
                    }}

                    let scheduled = null;
                    function scheduleScan() {{
                        if (scheduled !== null) return;
                        scheduled = window.requestAnimationFrame(function() {{
                            scheduled = null;
                            scanChatIds();
                        }});
                    }}

                    const obsRoot = document.querySelector(ROOT_SEL) || document.body;
                    window._kotauri_chat_id_observer = new MutationObserver(scheduleScan);
                    window._kotauri_chat_id_observer.observe(obsRoot, {{ childList: true, subtree: true }});
                    scheduleScan();
                    window._kotauri_chat_id_interval = window.setInterval(scheduleScan, 2500);
                }}
            }})();
            "##,
                chat_root = WebKSelectors::COLUMN_LEFT,
                chat_rows = WebKSelectors::CHAT_ROW_SELECTORS,
            ));
        } else {
            js.push_str(r#"
            (function() {
                if (window._kotauri_chat_id_interval) {
                    clearInterval(window._kotauri_chat_id_interval);
                    window._kotauri_chat_id_interval = null;
                }
                if (window._kotauri_chat_id_observer) {
                    window._kotauri_chat_id_observer.disconnect();
                    window._kotauri_chat_id_observer = null;
                }
                document.querySelectorAll('.kotauri-chat-id').forEach(el => el.remove());
            })();
            "#);
        }

        js.push_str(&injected_compat_warning_js());
        js
    }
}

// Implement Serialize for the outer struct (for Tauri state)
impl Serialize for AppSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let data = self
            .inner
            .lock()
            .map_err(|e| serde::ser::Error::custom(format!("settings lock: {e}")))?;
        data.serialize(serializer)
    }
}
