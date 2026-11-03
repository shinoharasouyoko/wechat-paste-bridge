use crate::models::{ArticlePackage, Manifest, SourceKind};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("文章包路径不存在或不是文件夹: {0}")]
    InvalidInput(PathBuf),
    #[error("文章包中缺少 manifest.json: {0}")]
    MissingManifest(PathBuf),
    #[error("manifest.json 格式无效: {0}")]
    InvalidManifest(#[from] serde_json::Error),
    #[error("文章包中的路径越出了根目录: {0}")]
    RootEscape(String),
    #[error("无法读取文章正文: {0}")]
    ReadArticle(#[source] std::io::Error),
    #[error("封面文件不存在: {0}")]
    MissingCover(PathBuf),
}

pub fn load_package(input: &Path) -> Result<ArticlePackage, PackageError> {
    if !input.is_dir() {
        return Err(PackageError::InvalidInput(input.to_path_buf()));
    }

    let root = fs::canonicalize(input).map_err(|_| PackageError::InvalidInput(input.to_path_buf()))?;
    let manifest_path = [root.join("draft.json"), root.join("manifest.json")]
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| PackageError::MissingManifest(root.join("manifest.json")))?;
    let manifest_contents = fs::read_to_string(&manifest_path).map_err(PackageError::ReadArticle)?;
    let manifest: Manifest = serde_json::from_str(manifest_contents.trim_start_matches('\u{feff}'))?;

    let article_path = resolve_in_root(&root, &manifest.article_html)?;
    let cover_path = resolve_in_root(&root, &manifest.cover)?;
    if !cover_path.is_file() {
        return Err(PackageError::MissingCover(cover_path));
    }
    let article_html = fs::read_to_string(article_path)
        .map_err(PackageError::ReadArticle)?
        .trim_end()
        .to_owned();

    Ok(ArticlePackage {
        root,
        manifest,
        article_html,
        cover_path,
        source_kind: SourceKind::Directory,
    })
}

fn resolve_in_root(root: &Path, relative: &str) -> Result<PathBuf, PackageError> {
    let candidate = root.join(relative);
    let canonical = fs::canonicalize(&candidate)
        .map_err(|_| PackageError::RootEscape(relative.to_owned()))?;
    if canonical.starts_with(root) {
        Ok(canonical)
    } else {
        Err(PackageError::RootEscape(relative.to_owned()))
    }
}
