//! Destino nativo de downloads do WebView → pasta Downloads do usuário.

use std::path::{Path, PathBuf};

use tauri::webview::DownloadEvent;
use tauri::Webview;

/// Handler para [`WebviewWindowBuilder::on_download`].
pub fn handle_download_event(event: DownloadEvent<'_>) -> bool {
    match event {
        DownloadEvent::Requested { url, destination } => {
            let dir = download_dir();
            let suggested = suggested_filename(destination, url.as_str());
            let path = unique_path(&dir, &suggested);
            println!("[KoTauri] download solicitado: {url} -> {}", path.display());
            *destination = path;
            true
        }
        DownloadEvent::Finished { url, path, success } => {
            if success {
                println!(
                    "[KoTauri] download concluído: {url} -> {}",
                    path.as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "(sem caminho)".into())
                );
            } else {
                eprintln!("[KoTauri] download falhou: {url} path={path:?}");
            }
            true
        }
        _ => true,
    }
}

fn download_dir() -> PathBuf {
    dirs::download_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .map(|h| h.join("Downloads"))
            .unwrap_or_else(|| PathBuf::from("."))
    })
}

fn suggested_filename(destination: &Path, url: &str) -> String {
    if let Some(name) = destination.file_name() {
        let s = name.to_string_lossy();
        if !s.is_empty() && s != "." && !s.starts_with("data:") {
            return sanitize_filename(&s);
        }
    }
    if let Some(seg) = url
        .split('?')
        .next()
        .unwrap_or(url)
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
    {
        return sanitize_filename(seg);
    }
    "download".to_string()
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_start_matches('.');
    if trimmed.is_empty() {
        "download".to_string()
    } else {
        trimmed.chars().take(180).collect()
    }
}

fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let _ = std::fs::create_dir_all(dir);
    let mut path = dir.join(filename);
    if !path.exists() {
        return path;
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".into());
    let ext = path
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    let mut counter = 1u32;
    loop {
        path = dir.join(format!("{stem} ({counter}){ext}"));
        if !path.exists() {
            return path;
        }
        counter += 1;
        if counter > 10_000 {
            return dir.join(format!("{stem}-{}.{}", counter, ext.trim_start_matches('.')));
        }
    }
}

/// Assinatura adaptada ao callback do Tauri (ignora o webview).
pub fn on_download<R: tauri::Runtime>(_webview: Webview<R>, event: DownloadEvent<'_>) -> bool {
    handle_download_event(event)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_strips_path_chars() {
        assert_eq!(sanitize_filename("../../a:b?.png"), "_.._a_b_.png");
    }

    #[test]
    fn unique_path_when_missing() {
        let dir = std::env::temp_dir().join(format!("kotauri-dl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = unique_path(&dir, "foto.png");
        assert_eq!(path, dir.join("foto.png"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn suggested_from_url_path() {
        assert_eq!(
            suggested_filename(Path::new(""), "https://cdn.example/files/doc.pdf?x=1"),
            "doc.pdf"
        );
    }
}
