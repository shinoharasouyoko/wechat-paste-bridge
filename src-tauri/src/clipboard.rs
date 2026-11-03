use crate::models::ClipboardDocument;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct ClipboardPayload {
    pub html: String,
    pub plain_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CopyResult {
    pub copied_images: usize,
    pub compatibility_status: CompatibilityStatus,
}

#[derive(Debug, Clone, Serialize)]
pub enum CompatibilityStatus {
    Pending,
    Verified,
    Failed,
}

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("复制内容含有本地文件路径，已拒绝写入剪贴板")]
    LocalPath,
    #[error("无法访问 Windows 剪贴板: {0}")]
    System(String),
}

pub fn build_clipboard_payload(document: &ClipboardDocument) -> Result<ClipboardPayload, ClipboardError> {
    if document.safe_html.contains("assets/") || document.safe_html.contains("file://") {
        return Err(ClipboardError::LocalPath);
    }
    Ok(ClipboardPayload { html: document.safe_html.clone(), plain_text: document.plain_text.clone() })
}

pub fn write_clipboard(document: &ClipboardDocument) -> Result<CopyResult, ClipboardError> {
    let payload = build_clipboard_payload(document)?;
    write_windows_html_fragment(&payload)?;
    Ok(CopyResult { copied_images: document.images.len(), compatibility_status: CompatibilityStatus::Verified })
}

fn write_windows_html_fragment(payload: &ClipboardPayload) -> Result<(), ClipboardError> {
    use clipboard_win::{formats, Clipboard, Getter, Setter};

    let _clipboard = Clipboard::new_attempts(10)
        .map_err(|error| ClipboardError::System(error.to_string()))?;

    // CF_UNICODETEXT clears the previous clipboard content. It must be written before
    // the registered HTML Format so both representations are available to the editor.
    formats::Unicode.write_clipboard(&payload.plain_text)
        .map_err(|error| ClipboardError::System(error.to_string()))?;
    let html_format = formats::Html::new()
        .ok_or_else(|| ClipboardError::System("无法注册 Windows HTML Format".to_owned()))?;
    let header_template = "Version:1.0\r\nStartHTML:0000000000\r\nEndHTML:0000000000\r\nStartFragment:0000000000\r\nEndFragment:0000000000\r\n";
    let prefix = "<html><body><!--StartFragment-->";
    let suffix = "<!--EndFragment--></body></html>";
    let start_html = header_template.len();
    let start_fragment = start_html + prefix.len();
    let end_fragment = start_fragment + payload.html.len();
    let end_html = end_fragment + suffix.len();
    let header = format!("Version:1.0\r\nStartHTML:{start_html:010}\r\nEndHTML:{end_html:010}\r\nStartFragment:{start_fragment:010}\r\nEndFragment:{end_fragment:010}\r\n");
    let cf_html = format!("{header}{prefix}{}{suffix}\0", payload.html);
    clipboard_win::raw::set_without_clear(html_format.code(), cf_html.as_bytes())
        .map_err(|error| ClipboardError::System(error.to_string()))?;

    let mut read_back = String::new();
    html_format.read_clipboard(&mut read_back)
        .map_err(|error| ClipboardError::System(format!("富文本格式回读失败: {error}")))?;
    if read_back != payload.html {
        return Err(ClipboardError::System("富文本格式回读内容不完整，已停止复制".to_owned()));
    }
    Ok(())
}
