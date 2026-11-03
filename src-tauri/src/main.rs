#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use wechat_paste_bridge::package::load_package;
use wechat_paste_bridge::validate::validate_package;
use wechat_paste_bridge::render::build_clipboard_document;
use wechat_paste_bridge::clipboard::write_clipboard;
use wechat_paste_bridge::models::{ClipboardDocument, ImportResult};
use std::fs;

#[tauri::command]
fn import_article_package(path: String) -> Result<ImportResult, String> {
    let package = load_package(std::path::Path::new(&path)).map_err(|error| error.to_string())?;
    let report = validate_package(&package);
    let document = build_clipboard_document(&package, &report).ok();
    let load_profile = |relative: &Option<String>| -> Option<String> {
        let html = relative.as_ref().and_then(|path| fs::read_to_string(package.root.join(path)).ok())?;
        let mut profile = package.clone();
        profile.article_html = html;
        build_clipboard_document(&profile, &validate_package(&profile)).ok().map(|item| item.safe_html)
    };
    Ok(ImportResult {
        title: package.manifest.title.clone(),
        can_copy: report.can_copy,
        issues: report.issues.iter().map(|issue| format!("{}：{}", issue.path, issue.repair)).collect(),
        safe_html: document.as_ref().map(|item| item.safe_html.clone()),
        plain_text: document.map(|item| item.plain_text),
        wechat_html: load_profile(&package.manifest.wechat_html),
        xiumi_html: load_profile(&package.manifest.xiumi_html),
    })
}

#[tauri::command]
fn copy_article(document: ClipboardDocument) -> Result<wechat_paste_bridge::clipboard::CopyResult, String> {
    write_clipboard(&document).map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![import_article_package, copy_article])
        .run(tauri::generate_context!())
        .expect("failed to launch WeChat Paste Bridge");
}
