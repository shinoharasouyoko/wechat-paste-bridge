use crate::models::{ArticlePackage, ValidationIssue, ValidationReport};
use regex::Regex;
use std::fs;

const MAX_GIF_BYTES: u64 = 5 * 1024 * 1024;

pub use crate::models::RiskKind;

pub fn validate_package(package: &ArticlePackage) -> ValidationReport {
    let mut issues = Vec::new();
    let image_sources = Regex::new(r#"(?i)<img[^>]+\bsrc\s*=\s*[\"']([^\"']+)[\"']"#)
        .expect("image source expression is valid");

    for capture in image_sources.captures_iter(&package.article_html) {
        let source = capture[1].trim();
        if source.starts_with("http://") || source.starts_with("https://") {
            issues.push(issue(
                source,
                RiskKind::RemoteImage,
                "发现外部网络图片，复制时不会下载该图片。",
                "将图片下载到文章包 assets 文件夹，并把 src 改为相对路径。",
            ));
            continue;
        }

        if source.starts_with('/') || source.contains("..") {
            issues.push(issue(
                source,
                RiskKind::MissingLocalImage,
                "图片路径必须位于文章包目录内。",
                "将图片放入 assets 文件夹，并使用 assets/文件名 的相对路径。",
            ));
            continue;
        }

        let image_path = package.root.join(source);
        if !image_path.is_file() {
            issues.push(issue(
                source,
                RiskKind::MissingLocalImage,
                "本地图片文件不存在。",
                "补齐该文件，或修改 HTML 中的 src 路径。",
            ));
            continue;
        }

        if source.to_ascii_lowercase().ends_with(".gif") {
            let size = fs::metadata(&image_path).map(|metadata| metadata.len()).unwrap_or(0);
            if size > MAX_GIF_BYTES {
                issues.push(issue(
                    source,
                    RiskKind::OversizedGif,
                    "GIF 超过 5 MB，公众号粘贴失败风险较高。",
                    "压缩 GIF 到 5 MB 以下，或导出为静态 PNG。",
                ));
            }
        }
    }

    let unsupported = ["<script", "position:absolute", "@font-face", "filter:"];
    let mut fixed_component_count = 0;
    for marker in unsupported {
        if package.article_html.to_ascii_lowercase().contains(marker) {
            fixed_component_count += 1;
            issues.push(issue(
                marker,
                RiskKind::UnsupportedMarkup,
                "此组件包含公众号编辑器不稳定的网页样式。",
                "将该组件输出为 PNG 或 GIF 后重新引用。",
            ));
        }
    }

    let editable_count = Regex::new(r"(?i)<(p|h[1-6]|li|blockquote|td|th)[ >]")
        .expect("editable tag expression is valid")
        .find_iter(&package.article_html)
        .count();
    let can_copy = issues.iter().all(|entry| entry.kind == RiskKind::UnsupportedMarkup);

    ValidationReport { issues, editable_count, fixed_component_count, can_copy }
}

fn issue(path: &str, kind: RiskKind, message: &str, repair: &str) -> ValidationIssue {
    ValidationIssue {
        path: path.to_owned(),
        kind,
        message: message.to_owned(),
        repair: repair.to_owned(),
    }
}
