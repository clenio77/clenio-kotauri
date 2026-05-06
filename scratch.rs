use std::fs;

fn main() {
    let mut css = String::new();
    let data_sidebar_folders = true;
    
    if data_sidebar_folders {
        css.push_str("
            /* 1. Move o conteúdo principal da sidebar para a direita e espreme ele */
            .sidebar-content {
                left: 85px !important;
                width: calc(100% - 85px) !important;
                position: absolute !important;
                overflow: visible !important;
            }
            
            .transition-item {
                overflow: visible !important;
            }
            
            /* 2. Container original das abas: NÃO colocar height 0 senão o React desmonta! Apenas tira do fluxo. */
            ._container_fkln9_1 {
                position: absolute !important;
                left: 0 !important;
                top: 0 !important;
                background: transparent !important;
                overflow: visible !important;
                z-index: 9999 !important;
            }
            
            /* 3. Tira as pastas do fluxo e joga exatamente no espaço de 85px que esvaziamos à esquerda */
            .folders-tabs-scrollable {
                position: absolute !important;
                left: -85px !important;
                top: 0 !important;
                width: 85px !important;
                height: 100vh !important;
                flex-direction: column !important;
                background-color: var(--bg-color, #17212b) !important;
                border-right: 1px solid var(--border-color, rgba(0,0,0,0.1)) !important;
                z-index: 9999 !important;
                overflow-y: auto !important;
                scrollbar-width: none !important;
                border-radius: 0 !important;
                padding-top: 8px !important;
                display: flex !important;
            }
            .folders-tabs-scrollable::-webkit-scrollbar { display: none !important; }
            
            .menu-horizontal-div {
                flex-direction: column !important;
                width: 100% !important;
            }
            
            .menu-horizontal-div-item {
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
            }
        ");
    }
    
    println!("CSS IS: {}", css);
}
