use wechat_paste_bridge::clipboard::{build_clipboard_payload, write_clipboard, CompatibilityStatus};
use wechat_paste_bridge::models::{ClipboardDocument, ClipboardImage};

#[test]
fn builds_html_and_plain_text_clipboard_payloads_without_local_paths() {
    let document = ClipboardDocument {
        safe_html: r#"<p>正文</p><img src="data:image/png;base64,AA==">"#.to_owned(),
        plain_text: "正文".to_owned(),
        images: vec![ClipboardImage {
            source_path: "assets/photo.png".to_owned(),
            data_uri: "data:image/png;base64,AA==".to_owned(),
        }],
        editable_blocks: 1,
        fixed_components: vec![],
    };

    let payload = build_clipboard_payload(&document).unwrap();

    assert_eq!(payload.plain_text, "正文");
    assert!(payload.html.contains("data:image/png"));
    assert!(!payload.html.contains("assets/photo.png"));
}

#[test]
fn writes_a_verified_html_fragment_to_the_windows_clipboard() {
    let document = ClipboardDocument {
        safe_html: r#"<div style="background:#151816;color:#f4f2eb"><strong>富文本探针</strong></div>"#.to_owned(),
        plain_text: "富文本探针".to_owned(),
        images: vec![],
        editable_blocks: 1,
        fixed_components: vec![],
    };

    let result = write_clipboard(&document).unwrap();
    let mut clipboard = arboard::Clipboard::new().unwrap();
    let html = clipboard.get().html().unwrap();

    assert!(matches!(result.compatibility_status, CompatibilityStatus::Verified));
    assert!(html.contains("background:#151816"));
    assert!(html.contains("富文本探针"));
}

#[test]
fn writes_large_multibyte_html_with_correct_fragment_offsets() {
    let body = format!("<section>{}<img src=\"data:image/jpeg;base64,{}\"></section>", "舞台测试".repeat(100), "A".repeat(3_000_000));
    let document = ClipboardDocument { safe_html: body.clone(), plain_text: "舞台测试".into(), images: vec![], editable_blocks: 1, fixed_components: vec![] };
    write_clipboard(&document).unwrap();
    let mut clipboard = arboard::Clipboard::new().unwrap();
    assert_eq!(clipboard.get().html().unwrap(), body);
}
