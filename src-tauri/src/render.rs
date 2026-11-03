use crate::models::{ArticlePackage, ClipboardDocument, ClipboardImage, ValidationReport};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("文章包存在阻断复制的风险，请先按自检建议修复")]
    UnsafePackage,
    #[error("无法读取图片 {0}")]
    ReadImage(String),
}

pub fn build_clipboard_document(
    package: &ArticlePackage,
    report: &ValidationReport,
) -> Result<ClipboardDocument, RenderError> {
    if !report.can_copy {
        return Err(RenderError::UnsafePackage);
    }

    let script = Regex::new(r"(?is)<script[^>]*>.*?</script>").expect("script expression is valid");
    let source = Regex::new(r#"(?i)(<img[^>]+\bsrc\s*=\s*[\"'])([^\"']+)([\"'][^>]*>)"#)
        .expect("image expression is valid");
    let tag = Regex::new(r"(?is)<[^>]+>").expect("tag expression is valid");
    let without_scripts = script.replace_all(&package.article_html, "");
    let mut images = Vec::new();
    let safe_html = source.replace_all(&without_scripts, |captures: &regex::Captures| {
        let path = &captures[2];
        if path.starts_with("data:") {
            return captures[0].to_owned();
        }
        let bytes = fs::read(package.root.join(path)).unwrap_or_default();
        let data_uri = format!("data:{};base64,{}", mime_for(path), STANDARD.encode(bytes));
        images.push(ClipboardImage { source_path: path.to_owned(), data_uri: data_uri.clone() });
        format!("{}{}{}", &captures[1], data_uri, &captures[3])
    });
    let fixed_components = report
        .issues
        .iter()
        .filter(|issue| matches!(issue.kind, crate::models::RiskKind::UnsupportedMarkup))
        .map(|issue| issue.path.clone())
        .collect();

    Ok(ClipboardDocument {
        safe_html: safe_html.into_owned(),
        plain_text: tag.replace_all(&without_scripts, " ").split_whitespace().collect::<Vec<_>>().join("\n"),
        images,
        editable_blocks: report.editable_count,
        fixed_components,
    })
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or_default().to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "jpeg" | "jpg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}
