//! The file search's first and next steps as the installer calls them (the screenshots' numbering
//! is `gfx::write_screen_pcx`'s).

use crate::files::state::set_last_file_path;

/// `files::find_next_wanted`: the next name not of the skipped attributes; false when
/// there is none.
pub fn find_next_wanted(skipped: u32) -> bool {
    crate::files::search::next(skipped)
}

/// `files::find_first`: the first name matching `pattern` (without its NUL; not
/// resolved; it becomes `last_file_path`) not of the skipped attributes; false when none.
pub fn find_first(pattern: &[u8], skipped: u32) -> bool {
    set_last_file_path(Some(pattern));
    crate::files::search::first(pattern, skipped)
}
