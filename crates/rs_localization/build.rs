use std::path::Path;

fn main() {
    let locales_dir = Path::new("../../Resource/locales");
    if let Ok(entries) = std::fs::read_dir(locales_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yml") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
