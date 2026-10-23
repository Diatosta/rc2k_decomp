//! `install::read_sku` (0x411696): reads the edition file `files\d\gfx\sku`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the key `sku`. Its value may start with the marks `oem:` and
//! `dvd:`, in any order and number, each setting a switch to 1 (`dword_5190F8`,
//! `dword_5190FC`); then a number (`dword_5190F4`), after the next `:` a second
//! (`byte_519101`) and after the next `.` within 254 characters a third (`byte_519100`).
//! What is not there keeps its value. One caller, `sub_411239`.
//! Self-test: a redirected copy that reads a file of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::bridge::Raw;
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;

// ---- game data ----

pub const PATH_RVA: usize = rva(0x519102); // "files\d\gfx\sku"
const NUMBER_RVA: usize = rva(0x5190F4); // dword_5190F4
const OEM_RVA: usize = rva(0x5190F8); // dword_5190F8
const DVD_RVA: usize = rva(0x5190FC); // dword_5190FC
const THIRD_RVA: usize = rva(0x519100); // byte_519100
const SECOND_RVA: usize = rva(0x519101); // byte_519101

// ---- Rust version ----

pub unsafe fn read_sku(path: *const u8) -> Result<(), BadInput> {
    unsafe {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        // Converted: `rc2k_core::install::config::read_sku` (the key as its constant).
        let Some(sku) = config::read_sku(&Raw::new(text), &case_table())? else {
            return Ok(());
        };
        if sku.oem {
            game_write(OEM_RVA, 1u32);
        }
        if sku.dvd {
            game_write(DVD_RVA, 1u32);
        }
        game_write(NUMBER_RVA, sku.number);
        if let Some(second) = sku.second {
            game_write(SECOND_RVA, second);
        }
        if let Some(third) = sku.third {
            game_write(THIRD_RVA, third);
        }
        Ok(())
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_sku(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
