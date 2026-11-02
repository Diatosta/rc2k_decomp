//! `install::read_record` (0x4110C9): reads the installation's record `var\ral.zog`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with three keys, each taken if it is there: `cdrom` becomes base
//! directory A (`byte_5189A8`), `installation` becomes `g_mediaMode` (0x5189A4), held to
//! 0 .. 3, and `language` becomes what `%` stands for in a path (`byte_518AA8`). The two
//! texts get a `\` after them unless they are empty or end in `\` or `/`. Without the
//! file nothing changes.
//! Callers: `start`, `sub_412DE0`, `sub_416653`.
//! Refuses a text of 0xFE bytes or more (the original writes past the 0x100 bytes).
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::BadInput;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

// ---- game data ----

// ---- Rust version ----

pub fn read_record(path: crate::pointers::GamePtr) -> Result<(), BadInput> {
    {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        // Converted: `rc2k_core::install::config::read_record` (the keys as its constants).
        let record = config::read_record(&text.bytes("the install record")?, &case_table())?;
        for (place, text) in [
            (crate::files::state::base_directory_place(), record.base),
            (crate::files::state::locale_directory_place(), record.locale),
        ] {
            if let Some(text) = text {
                place.write_bytes(&text, "an installer path")?;
            }
        }
        if let Some(mode) = record.mode {
            crate::files::state::MEDIA_MODE.store(mode, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }
}
