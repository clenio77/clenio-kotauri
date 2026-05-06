use serde_json::Value;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Listener, Manager, Emitter,
};

use crate::settings::AppSettings;

pub mod settings;
pub mod web_selectors;

/// Primeira carga do Web K: várias tentativas espaçadas (SPA substitui o DOM).
const INITIAL_INJECTION_DELAY_MS: u64 = 400;
const INJECTION_INTERVAL_MS: u64 = 750;
const INJECTION_ATTEMPTS: u32 = 22;

fn show_settings_window(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("settings")
        .ok_or_else(|| "janela de configurações não encontrada".to_string())?;
    let settings = app.state::<AppSettings>();
    let data = settings
        .inner
        .lock()
        .map_err(|e| format!("mutex de configurações: {e}"))?;
    let json = serde_json::to_string(&*data).map_err(|e| e.to_string())?;
    window
        .eval(format!("window.initialSettings = {json};"))
        .map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    show_settings_window(&app)
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Result<String, String> {
    let settings = app.state::<AppSettings>();
    let data = settings
        .inner
        .lock()
        .map_err(|e| format!("mutex de configurações: {e}"))?;
    serde_json::to_string(&*data).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_setting(app: tauri::AppHandle, key: String, value: Value) -> Result<(), String> {
    println!("[KoTauri] alteração: key={key}, value={value}");
    let settings = app.state::<AppSettings>();

    let value_str = match value {
        Value::String(s) => s,
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    };

    settings.update(&key, &value_str);
    settings.save();
    inject_to_main(&app);
    Ok(())
}

async fn run_initial_injection_schedule(app: tauri::AppHandle) {
    tokio::time::sleep(Duration::from_millis(INITIAL_INJECTION_DELAY_MS)).await;
    for attempt in 1..=INJECTION_ATTEMPTS {
        inject_to_main(&app);
        if attempt < INJECTION_ATTEMPTS {
            tokio::time::sleep(Duration::from_millis(INJECTION_INTERVAL_MS)).await;
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_settings = AppSettings::load();
    let start_minimized = app_settings
        .inner
        .lock()
        .map(|d| d.start_minimized)
        .unwrap_or(false);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_settings)
        .invoke_handler(tauri::generate_handler![open_settings, get_settings, update_setting])
        .setup(move |app| {
            if let Some(settings_win) = app.get_webview_window("settings") {
                let win_clone = settings_win.clone();
                settings_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            let app_handle_ipc = app.handle().clone();

            let main_window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::External("https://web.telegram.org/k/".parse().unwrap()),
            )
            .title("KoTauri — Telegram")
            .inner_size(1200.0, 800.0)
            .initialization_script(
                "
                window.kotauri = {
                    openSettings: () => {
                        let f = document.createElement('iframe');
                        f.style.display = 'none';
                        f.src = 'https://kotauri.internal/open-settings';
                        document.body.appendChild(f);
                        setTimeout(() => f.remove(), 500);
                    }
                };
            ",
            )
            .on_navigation(move |url| {
                if url.as_str().contains("kotauri.internal/open-settings") {
                    let _ = app_handle_ipc.emit("open-settings", ());
                    return false;
                }
                true
            })
            .build()
            .map_err(|e| format!("falha ao criar janela principal: {e}"))?;

            #[cfg(target_os = "linux")]
            {
                main_window
                    .with_webview(|webview| {
                        use webkit2gtk::{PermissionRequestExt, WebViewExt};
                        webview.inner().connect_permission_request(|_, request| {
                            request.allow();
                            true
                        });
                    })
                    .map_err(|e| format!("webkit permissões: {e}"))?;
            }

            let main_for_close = main_window.clone();
            let app_for_focus = app.handle().clone();
            let app_for_close = app.handle().clone();
            main_window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::Focused(true) => {
                        inject_to_main(&app_for_focus);
                    }
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        let minimize = app_for_close
                            .state::<AppSettings>()
                            .inner
                            .lock()
                            .map(|d| d.minimize_to_tray)
                            .unwrap_or(false);
                        if minimize {
                            api.prevent_close();
                            let _ = main_for_close.hide();
                        }
                    }
                    _ => {}
                }
            });

            if start_minimized {
                let _ = main_window.hide();
            }

            let app_handle_init = app.handle().clone();
            tauri::async_runtime::spawn(run_initial_injection_schedule(app_handle_init));

            let app_handle_for_open = app.handle().clone();
            app.listen("open-settings", move |_| {
                let _ = show_settings_window(&app_handle_for_open);
            });

            let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Mostrar Telegram", true, None::<&str>)?;
            let settings_i =
                MenuItem::with_id(app, "settings_menu", "Configurações", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &settings_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "settings_menu" => {
                            let _ = show_settings_window(app.app_handle());
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running KoTauri");
}

fn inject_to_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        println!("[KoTauri] injetando CSS/JS na janela principal...");
        let settings = app.state::<AppSettings>();
        let css = settings.generate_css();
        let js = settings.generate_js();

        let injection = format!(
            "
            (function() {{
                let style = document.getElementById('kotauri-css');
                if (!style) {{
                    style = document.createElement('style');
                    style.id = 'kotauri-css';
                    document.head.appendChild(style);
                }}
                style.textContent = {:?};

                let script = document.getElementById('kotauri-js');
                if (!script) {{
                    script = document.createElement('script');
                    script.id = 'kotauri-js';
                    script.textContent = {:?};
                    document.head.appendChild(script);
                }}
            }})();
            ",
            css, js
        );

        match window.eval(&injection) {
            Ok(_) => println!("[KoTauri] injeção aplicada."),
            Err(e) => println!("[KoTauri] erro na injeção: {:?}", e),
        }
    } else {
        println!("[KoTauri] janela 'main' não encontrada.");
    }
}
