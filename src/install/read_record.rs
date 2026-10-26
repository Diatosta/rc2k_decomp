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
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::bridge::Raw;
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::install::{BASE_DIRECTORY_RVA, LOCALE_RVA};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

// ---- game data ----

const PATH_RVA: usize = rva(0x5295C0); // "var\ral.zog"
const MEDIA_MODE_RVA: usize = rva(0x5189A4); // g_mediaMode

// ---- Rust version ----

pub unsafe fn read_record(path: *const u8) -> Result<(), BadInput> {
    unsafe {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        // Converted: `rc2k_core::install::config::read_record` (the keys as its constants).
        let record = config::read_record(&Raw::new(text), &case_table())?;
        for (place, text) in [
            (BASE_DIRECTORY_RVA, record.base),
            (LOCALE_RVA, record.locale),
        ] {
            if let Some(text) = text {
                std::ptr::copy(text.as_ptr(), game_addr(place) as *mut u8, text.len());
            }
        }
        if let Some(mode) = record.mode {
            game_write(MEDIA_MODE_RVA, mode);
        }
        Ok(())
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_record(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
