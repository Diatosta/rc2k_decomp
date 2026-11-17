//! Folders of the game's paths that live elsewhere than the path says: a bundle on macOS keeps
//! the game's read-only `files` tree inside itself (`Contents/Resources/files`) while the
//! current folder, where `var` and `savedata` are written, is the user's Application Support
//! folder. A rule swaps the start of a path (as `files::host_path` reads it, `/` separated) for
//! another folder, whatever the case of the names it starts with.

// Only the macOS bundle sets a rule; the other systems build it for its tests.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::sync::Mutex;

/// (the start to match, with its `/` at the end, lower case; what replaces it, with its `/`).
static RULES: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

fn rules() -> std::sync::MutexGuard<'static, Vec<(String, String)>> {
    RULES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Paths under the folder `from` are read under the folder `to`.
pub fn add(from: &str, to: &str) {
    let slash = |t: &str| format!("{}/", t.trim_end_matches('/'));
    rules().push((slash(from).to_lowercase(), slash(to)));
}

/// Whether a rule is set.
pub fn active() -> bool {
    !rules().is_empty()
}

/// `text` with the first rule that matches its start applied.
pub fn apply(text: String) -> String {
    let rules = rules();
    if rules.is_empty() {
        return text;
    }
    let lower = text.to_lowercase();
    for (from, to) in rules.iter() {
        // The folder itself (no `/` after it) as well as what is in it.
        let whole = from.trim_end_matches('/');
        if lower.starts_with(from.as_str()) && text.is_char_boundary(from.len()) {
            return format!("{to}{}", &text[from.len()..]);
        }
        if lower == whole {
            return to.trim_end_matches('/').to_string();
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rule_swaps_the_start_of_a_path() {
        add(
            "/support/Rally/files",
            "/Apps/Rally.app/Contents/Resources/files",
        );
        let at = |t: &str| apply(t.to_string());
        assert_eq!(
            at("/Support/rally/FILES/d/gfx/jade.bun"),
            "/Apps/Rally.app/Contents/Resources/files/d/gfx/jade.bun"
        );
        assert_eq!(
            at("/support/Rally/files"),
            "/Apps/Rally.app/Contents/Resources/files"
        );
        assert_eq!(at("/support/Rally/var/x"), "/support/Rally/var/x");
        assert_eq!(at("/support/Rally/filesx/y"), "/support/Rally/filesx/y");
        RULES.lock().unwrap().clear();
    }
}
