use std::fs;
use std::path::Path;

#[test]
fn grants_the_main_window_permission_to_open_the_native_folder_dialog() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities/default.json");
    let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    let permissions = value["permissions"].as_array().unwrap();

    assert!(permissions.iter().any(|entry| entry == "dialog:allow-open"));
    assert!(permissions.iter().any(|entry| entry == "opener:default"));
}
