//! Seletores do Telegram Web K centralizados. Atualize quando o layout oficial mudar.

/// Marcadores de DOM usados por CSS/JS injetados (Web Telegram K).
pub struct WebKSelectors;

impl WebKSelectors {
    pub const COLUMN_LEFT: &'static str = "#column-left";
    pub const CHAT_LIST_CHAT: &'static str = ".chatlist-chat";
    /// Seletores de linha na lista de chats (Web K). Vários padrões — DOM pode mudar entre builds.
    pub const CHAT_ROW_SELECTORS: &'static str =
        ".chatlist-chat, .chat-list-item, li.chatlist-chat, .chatlist-chat.row-clickable-hover";
    pub const SIDEBAR_FALLBACK: &'static str = ".sidebar";
}

/// Trecho JS injetado após o resto: avisa na consola se o layout parecer incompatível.
pub fn injected_compat_warning_js() -> String {
    format!(
        r#"
            (function() {{
                const markers = ["{}", "{}"];
                const ok = markers.some((s) => document.querySelector(s));
                if (!ok) {{
                    console.warn(
                        "[KoTauri] Compatibilidade: não foi encontrada a coluna lateral esperada. " +
                        "O Telegram Web K pode ter atualizado — temas e outros ajustes injetados podem falhar."
                    );
                }}
            }})();
        "#,
        WebKSelectors::COLUMN_LEFT,
        WebKSelectors::SIDEBAR_FALLBACK
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectors_non_empty() {
        assert!(WebKSelectors::COLUMN_LEFT.starts_with('#'));
        assert!(WebKSelectors::CHAT_LIST_CHAT.starts_with('.'));
        assert!(WebKSelectors::CHAT_ROW_SELECTORS.contains("chatlist-chat"));
    }
}
