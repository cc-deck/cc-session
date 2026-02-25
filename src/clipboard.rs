// Cross-platform clipboard with fallback

use arboard::Clipboard;

/// Copy the given text to the system clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("failed to open clipboard: {e}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("failed to set clipboard text: {e}"))
}

/// Check whether clipboard access is available on this system.
#[allow(dead_code)]
pub fn clipboard_available() -> bool {
    Clipboard::new().is_ok()
}
