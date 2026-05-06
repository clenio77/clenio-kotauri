fn test(builder: tauri::WebviewWindowBuilder<tauri::Wry>) {
    builder.on_permission_request(|_, _| {});
}
fn main() {}
