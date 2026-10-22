//! `settings::read_settings` (0x4114A6): reads `var\ral.cfg` into the game's settings.
//!
//! In:  nothing
//! Out: every register preserved
//! `read_text` on `var\ral.cfg`; if it reads, `sub_4114FF` (0x4114FF, one caller, here as
//! `apply_settings`): for each of the 77 entries of the table at 0x518C09 (a key, then the
//! address of its variable), if `find_key` finds the key, the variable becomes
//! `parse_number` of its value. A key that is not in the file leaves its variable alone;
//! a key without a number makes it 0. Two keys share a variable twice (`noaudio` and
//! `nosound`, `allsecretcars` and `secretcar`): the later one in the table wins.
//! Callers: `start` and `sub_411239`.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::bridge::Raw;
use crate::errors::BadInput;
use crate::framework::{game_addr, rva};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;

// ---- game data ----

const PATH_RVA: usize = rva(0x5295CC); // "var\ral.cfg"
/// aNotaskswitch: the settings, each a key and the address of its dword; a 0 ends them.
const TABLE_RVA: usize = rva(0x518C09);
const TABLE_END_RVA: usize = rva(0x5190F1);

// ---- Rust version ----

/// `sub_4114FF`: the settings a text names. Converted: `rc2k_core::files::settings::table` and
/// `apply`; here the variables (the table's dwords are their addresses) are stored.
unsafe fn settings() -> Result<Vec<(Vec<u8>, u32)>, BadInput> {
    let table = unsafe { Raw::new(game_addr(TABLE_RVA) as *const u8) };
    Ok(rc2k_core::files::settings::table(
        &table,
        TABLE_END_RVA - TABLE_RVA,
    )?)
}

pub unsafe fn apply_settings(text: *const u8) -> Result<(), BadInput> {
    if text.is_null() {
        return Err(BadInput("no settings text or no key"));
    }
    unsafe {
        let table = settings()?;
        let keys: Vec<&[u8]> = table.iter().map(|(key, _)| key.as_slice()).collect();
        // Found first, stored after: nothing is written if a key is refused.
        let found = rc2k_core::files::settings::apply(&Raw::new(text), &keys, &case_table())?;
        for (n, number) in found {
            (table[n].1 as *mut u32).write_unaligned(number);
        }
        Ok(())
    }
}

pub unsafe fn read_settings() -> Result<(), BadInput> {
    unsafe {
        match read_text(game_addr(PATH_RVA) as *const u8)? {
            Some((text, _)) => apply_settings(text),
            None => Ok(()),
        }
    }
}
