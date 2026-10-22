//! `settings::read_driver_names` (0x4162DB): other names for the game's drivers, from
//! `var\name.cfg`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the keys `name000` .. `name179` (the key is made in place: the
//! number goes over the digits of `aName000`), each followed by up to three words as
//! `system::next_argument` cuts them: a name, of which at most 30 characters are kept in
//! the pool at `unk_95EFC8` (one after another, each with a NUL) and pointed at by
//! `dword_95ECF8[n]`; a number under 0x28 for the driver's record at `dword_73B688` and a
//! number under 0x22 for `dword_73B68C` (records of 0x94 bytes). A number that is too
//! large is left out; a missing third word ends the entry. Without the file nothing
//! changes. One caller, `sub_411239`. The owner's install has no such file.
//! Refuses names that together pass the pool's end at `unk_960000` (the original writes
//! on; 180 names of 30 characters would).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_write, rva};
use crate::settings::read_text::read_text;
use crate::strings::format_u32::format_u32;

// ---- game data ----

pub const PATH_RVA: usize = rva(0x529D98); // "var\name.cfg"
const KEY_RVA: usize = rva(0x529DA5); // aName000
const DRIVERS: usize = 180;
const NAMES_RVA: usize = rva(0x95ECF8); // dword_95ECF8: a pointer a driver
const POOL_RVA: usize = rva(0x95EFC8); // unk_95EFC8
/// Up to `unk_960000`: room for about 130 names of the full length, not for 180.
const POOL_ROOM: usize = 0x960000 - 0x95EFC8;
const RECORD_LEN: usize = 0x94;
const FIRST_NUMBER_RVA: usize = rva(0x73B688); // dword_73B688
const SECOND_NUMBER_RVA: usize = rva(0x73B68C); // dword_73B68C

// ---- Rust version ----

pub unsafe fn read_driver_names(path: *const u8) -> Result<(), BadInput> {
    // Converted: `rc2k_core::files::settings::driver_names`; the pool's room checked before
    // anything is written. The key the original makes in place (`aName000`) is left as
    // its last one, "name179".
    unsafe {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        let entries = rc2k_core::files::settings::driver_names(
            &crate::bridge::Raw::new(text),
            &crate::strings::same_text::case_table(),
        )
        .map_err(|e| BadInput(e.0))?;
        let room: usize = entries
            .iter()
            .flatten()
            .filter_map(|e| e.name.as_ref())
            .map(|n| n.len() + 1)
            .sum();
        if room > POOL_ROOM {
            return Err(BadInput("more driver names than their pool holds"));
        }
        let last = format_u32(DRIVERS as u32 - 1, 0x13);
        std::ptr::copy(
            last.as_ptr(),
            (game_addr(KEY_RVA) as *mut u8).add(4),
            last.len(),
        );
        let mut pool = game_addr(POOL_RVA) as *mut u8;
        for (driver, entry) in entries.iter().enumerate() {
            let Some(entry) = entry else {
                continue;
            };
            if let Some(name) = &entry.name {
                game_write(NAMES_RVA + driver * 4, pool as u32);
                std::ptr::copy(name.as_ptr(), pool, name.len());
                pool.add(name.len()).write(0);
                pool = pool.add(name.len() + 1);
            }
            if let Some(n) = entry.first {
                game_write(FIRST_NUMBER_RVA + driver * RECORD_LEN, n);
            }
            if let Some(n) = entry.second {
                game_write(SECOND_NUMBER_RVA + driver * RECORD_LEN, n);
            }
        }
        Ok(())
    }
}
