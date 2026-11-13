//! Whether a folder path (from its third byte on, past a drive letter and colon) holds a
//! character that is not allowed in a folder name (`"`, `+`, `*`, `,`, `;`, `=`, `/`, `:`, `<`,
//! `>`, `?`, `\`, `{`, `}`, `[`, `]`, `|`, 0x7F) or a double backslash. Used by
//! `install::install_files`.

/// Whether `path` has a bad character or `\\`.
pub fn check_folder_name(path: impl Into<crate::place::BytePlace>) -> bool {
    // The system's rules (`install::folders`; Windows: `rc2k_core::install::paths::
    // bad_folder_name`).
    use crate::install::folders::{Folders, Native};
    let answer = path
        .into()
        .text("a folder name")
        .and_then(|text| <Native as Folders>::bad_name(&text));
    answer.unwrap_or_else(|e| crate::fatal::raise::end("install::check_folder_name", e))
}
