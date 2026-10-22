//! `install::read_languages` (0x413908): reads the languages file `lang.cfg`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the keys `lang1` .. `lang9` (the table at 0x51C378: key, slot,
//! 0x7F), each value `name:directory`. Every key found has its value copied to its slot
//! of 0x80 bytes at 0x51BEEE (`sub_4139AB`, inlined). Then the slots that start with a
//! character above a space other than `:` and have a `:` are kept, moved together to the
//! front, the `:` made a NUL, and counted in `dword_51BEEA`; then `choose_language` for the
//! first of them. Without the file nothing changes, not even the count.
//! Callers: `start` and `sub_412DE0`.
//! Refuses a value of 0x80 bytes or more (the original writes on into the next slot).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::bridge::Raw;
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::install::choose_language::{
    LANGUAGE_COUNT_RVA, LANGUAGE_SLOT, LANGUAGE_SLOTS, LANGUAGES_RVA, choose_language,
};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

// ---- game data ----

const PATH_RVA: usize = rva(0x51C36E); // "lang.cfg"

// ---- Rust version ----

pub unsafe fn read_languages(path: *const u8) -> Result<(), BadInput> {
    unsafe {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        // Converted: `rc2k_core::install::config` (`find_languages`, its keys as constants;
        // `keep_languages`). Found first, stored after (`sub_4139AB`).
        let found = config::find_languages(&Raw::new(text), &case_table())?;
        let slots = std::slice::from_raw_parts_mut(
            game_addr(LANGUAGES_RVA) as *mut u8,
            LANGUAGE_SLOT * LANGUAGE_SLOTS,
        );
        for (slot, value) in found {
            let at = slot * LANGUAGE_SLOT;
            slots[at..at + value.len()].copy_from_slice(&value);
            slots[at + value.len()] = 0;
        }
        let kept = config::keep_languages(slots);
        game_write(LANGUAGE_COUNT_RVA, kept);
        choose_language(1)
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_languages(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
