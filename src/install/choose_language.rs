//! `install::choose_language` (0x41397A): makes one of the languages of `lang.cfg` the
//! language of the paths.
//!
//! In:  eax = the language's number, from 1
//! Out: every register preserved
//! Nothing if the number is above the count of languages (`dword_51BEEA`). A language is a
//! slot of 0x80 bytes at 0x51BEEE holding two texts, each ended by a NUL (`name:directory`
//! in the file); the second becomes what `%` stands for in a path (`byte_518AA8`).
//! Callers: `install::read_languages`, `sub_412F34`, `sub_414199`.
//! Refuses number 0 (the original takes the slot before the first) and a slot whose two
//! texts do not end inside it.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::install::LOCALE_RVA;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::install::config;

// ---- game data ----

pub const LANGUAGE_COUNT_RVA: usize = rva(0x51BEEA); // dword_51BEEA
pub const LANGUAGES_RVA: usize = rva(0x51BEEE); // unk_51BEEE
pub const LANGUAGE_SLOT: usize = config::SLOT;
pub const LANGUAGE_SLOTS: usize = config::SLOTS;

// ---- Rust version ----

pub unsafe fn choose_language(number: u32) -> Result<(), BadInput> {
    unsafe {
        // Converted: `rc2k_core::install::config` (`language_slot`, `language_directory`).
        let Some(slot) = config::language_slot(number, game_read::<u32>(LANGUAGE_COUNT_RVA))?
        else {
            return Ok(());
        };
        let slot = std::slice::from_raw_parts(
            game_addr(LANGUAGES_RVA + slot * LANGUAGE_SLOT) as *const u8,
            LANGUAGE_SLOT,
        );
        let directory = config::language_directory(slot)?;
        let to = game_addr(LOCALE_RVA) as *mut u8;
        std::ptr::copy(directory.as_ptr(), to, directory.len());
        to.add(directory.len()).write(0);
        Ok(())
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { choose_language(r.eax) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
