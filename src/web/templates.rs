//! Template utilities
//!
//! Helper functions for template rendering.

use std::path::PathBuf;

/// Load template from file or use inline fallback
pub fn load_template(name: &str) -> Option<String> {
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates");

    let template_path = template_dir.join(name);

    if template_path.exists() {
        std::fs::read_to_string(&template_path).ok()
    } else {
        None
    }
}
