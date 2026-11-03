use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub title: String,
    #[serde(alias = "content_html")]
    pub article_html: String,
    pub cover: String,
    #[serde(default, alias = "digest")]
    pub summary: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub wechat_html: Option<String>,
    #[serde(default)]
    pub xiumi_html: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SourceKind {
    Directory,
}

#[derive(Debug, Clone)]
pub struct ArticlePackage {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub article_html: String,
    pub cover_path: PathBuf,
    pub source_kind: SourceKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RiskKind {
    MissingLocalImage,
    RemoteImage,
    OversizedGif,
    UnsupportedMarkup,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationIssue {
    pub path: String,
    pub kind: RiskKind,
    pub message: String,
    pub repair: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub editable_count: usize,
    pub fixed_component_count: usize,
    pub can_copy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardImage {
    pub source_path: String,
    pub data_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardDocument {
    pub safe_html: String,
    pub plain_text: String,
    pub images: Vec<ClipboardImage>,
    pub editable_blocks: usize,
    pub fixed_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub title: String,
    pub can_copy: bool,
    pub issues: Vec<String>,
    pub safe_html: Option<String>,
    pub plain_text: Option<String>,
    pub wechat_html: Option<String>,
    pub xiumi_html: Option<String>,
}
