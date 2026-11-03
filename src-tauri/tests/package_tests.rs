use std::fs;
use wechat_paste_bridge::package::{load_package, PackageError};
use wechat_paste_bridge::validate::{validate_package, RiskKind};
use wechat_paste_bridge::render::build_clipboard_document;
use wechat_paste_bridge::models::ImportResult;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures").join(name)
}

#[test]
fn loads_a_valid_package_without_modifying_source() {
    let html_path = fixture("valid-package").join("article.html");
    let before = fs::metadata(&html_path).unwrap().modified().unwrap();

    let package = load_package(&fixture("valid-package")).unwrap();

    assert_eq!(package.manifest.title, "测试文章");
    assert_eq!(package.article_html, "<p>正文</p>");
    assert_eq!(fs::metadata(html_path).unwrap().modified().unwrap(), before);
}

#[test]
fn loads_existing_codex_draft_package_format() {
    let package = load_package(&fixture("legacy-draft-package")).unwrap();

    assert_eq!(package.manifest.title, "既有文章包");
    assert_eq!(package.article_html, "<p>既有正文</p>");
}

#[test]
fn prefers_a_valid_draft_json_when_an_unrelated_manifest_is_invalid() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    fs::write(root.join("manifest.json"), "not json").unwrap();
    fs::write(root.join("draft.json"), r#"{"title":"草稿优先","content_html":"article.html","cover":"cover.jpg"}"#).unwrap();
    fs::write(root.join("article.html"), "<p>正文</p>").unwrap();
    fs::write(root.join("cover.jpg"), "cover").unwrap();

    let package = load_package(root).unwrap();

    assert_eq!(package.manifest.title, "草稿优先");
}

#[test]
fn rejects_article_paths_outside_the_package_root() {
    let error = load_package(&fixture("invalid-package")).unwrap_err();

    assert!(matches!(error, PackageError::RootEscape(_)));
}

#[test]
fn reports_the_exact_missing_local_image() {
    let mut package = load_package(&fixture("valid-package")).unwrap();
    package.article_html = r#"<p>正文</p><img src="assets/missing.jpg">"#.to_owned();

    let report = validate_package(&package);

    assert!(report.issues.iter().any(|issue| {
        issue.kind == RiskKind::MissingLocalImage && issue.path == "assets/missing.jpg"
    }));
    assert!(!report.can_copy);
}

#[test]
fn reports_remote_images_without_fetching_them() {
    let mut package = load_package(&fixture("valid-package")).unwrap();
    package.article_html = r#"<img src="https://example.invalid/remote.jpg">"#.to_owned();

    let report = validate_package(&package);

    assert!(report.issues.iter().any(|issue| {
        issue.kind == RiskKind::RemoteImage && issue.path == "https://example.invalid/remote.jpg"
    }));
    assert!(!report.can_copy);
}

#[test]
fn blocks_an_oversized_gif() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    fs::write(root.join("article.html"), r#"<img src="assets/large.gif">"#).unwrap();
    fs::write(root.join("cover.jpg"), "cover").unwrap();
    fs::create_dir(root.join("assets")).unwrap();
    fs::write(root.join("assets/large.gif"), vec![0_u8; 5 * 1024 * 1024 + 1]).unwrap();
    fs::write(
        root.join("manifest.json"),
        r#"{"title":"GIF","article_html":"article.html","cover":"cover.jpg"}"#,
    )
    .unwrap();
    let package = load_package(root).unwrap();

    let report = validate_package(&package);

    assert!(report.issues.iter().any(|issue| issue.kind == RiskKind::OversizedGif));
    assert!(!report.can_copy);
}

#[test]
fn removes_scripts_and_marks_complex_sections_as_fixed_components() {
    let mut package = load_package(&fixture("valid-package")).unwrap();
    package.article_html = r#"<p>保留的文字</p><script>alert('x')</script><div style="position:absolute">组件</div>"#.to_owned();
    let report = validate_package(&package);

    let document = build_clipboard_document(&package, &report).unwrap();

    assert!(!document.safe_html.contains("script"));
    assert!(document.fixed_components.iter().any(|component| component.contains("position:absolute")));
    assert!(document.safe_html.contains("保留的文字"));
}

#[test]
fn serializes_import_result_for_the_typescript_contract() {
    let result = ImportResult {
        title: "测试".to_owned(),
        can_copy: true,
        issues: vec![],
        safe_html: Some("<p>正文</p>".to_owned()),
        plain_text: Some("正文".to_owned()),
        wechat_html: None,
        xiumi_html: None,
    };

    let value = serde_json::to_value(result).unwrap();

    assert_eq!(value["canCopy"], true);
    assert_eq!(value["safeHtml"], "<p>正文</p>");
    assert_eq!(value["plainText"], "正文");
}
