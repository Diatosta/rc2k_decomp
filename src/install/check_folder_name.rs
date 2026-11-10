//! `install::check_folder_name` (0x413A6C): whether a folder path (from its third byte on,
//! past a drive letter and colon) holds a character that is not allowed in a folder name
//! (`unk_51DE18`: `"`, `+`, `*`, `,`, `;`, `=`, `/`, `:`, `<`, `>`, `?`, `\`, `{`, `}`,
//! `[`, `]`, `|`, 0x7F) or a double backslash.
//!
//! Takes the path. Returns true if a bad character or `\\` is found.
//! Caller `install::install_files`.

/// Whether `path` has a bad character or `\\` (the original's carry).
pub fn check_folder_name(path: impl Into<crate::place::Place>) -> bool {
    // The system's rules (`install::folders`; Windows: `rc2k_core::install::paths::
    // bad_folder_name`, RAL.EXE's `unk_51DE18`).
    use crate::install::folders::{Folders, Native};
    let answer = path
        .into()
        .text("a folder name")
        .and_then(|text| <Native as Folders>::bad_name(&text));
    answer.unwrap_or_else(|e| crate::fatal::raise::end("install::check_folder_name", e))
}
