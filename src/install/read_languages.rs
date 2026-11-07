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
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::BadInput;
use crate::install::choose_language::{LANGUAGE_SLOT, LANGUAGE_SLOTS, choose_language};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

// ---- game data ----

// ---- Rust version ----

pub fn read_languages(
    g: &'static crate::install::owner::Install,
    path: &[u8],
) -> Result<(), BadInput> {
    {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        // Converted: `rc2k_core::install::config` (`find_languages`, its keys as constants;
        // `keep_languages`). Found first, stored after (`sub_4139AB`).
        let found = config::find_languages(&text.bytes("the languages' text")?, &case_table())?;
        let all = &g.names.languages;
        let mut slots = all.read_at(0, LANGUAGE_SLOT * LANGUAGE_SLOTS, "the languages' slots")?;
        for (slot, value) in found {
            let at = slot * LANGUAGE_SLOT;
            slots[at..at + value.len()].copy_from_slice(&value);
            slots[at + value.len()] = 0;
        }
        all.write_at(0, &slots, "the languages' slots")?;
        let kept = config::keep_languages(&mut slots);
        g.names.language_count.set(kept);
        choose_language(g, 1)
    }
}
