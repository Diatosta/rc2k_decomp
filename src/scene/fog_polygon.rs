//! `scene::fog_polygon` (0x44E286): each of the polygon's points gets its fog by its
//! depth.
//!
//! In:  nothing
//! Out: eax = the last byte stored, ecx = the number of vertices, edx = the last
//!      point's record, esi = the last entry; everything else preserved
//!
//! For each entry: the depth of its point's record (+0x18) as a whole number (`fistp`,
//! kept in `dword_62400C`), under 0 taken as 0, over 0x2000 is the place in the table
//! `byte_63661C` (`sub_44E260`; its length `dword_63761C`, a place past it is the last
//! one); the entry's byte +0x20 is that table's byte inverted. With `dword_6365C8` at 4
//! the byte is the low byte of `dword_636610` inverted instead.
//! One caller, the six-byte `sub_44E25A`, which six functions call.
//! Refuses a polygon of no vertices (the original goes through 2^32), an entry without
//! its record and a table's length above 0x1000. A length of 0, as the menus have it,
//! makes every place the byte before the table (the original's `length - 1`): kept.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::{dword, listed};

// ---- game data ----

const TABLE_RVA: usize = rva(0x63661C); // byte_63661C
const TABLE_LEN: u32 = 0x1000;
const LENGTH_RVA: usize = rva(0x63761C); // dword_63761C
const KIND_RVA: usize = rva(0x6365C8); // dword_6365C8
const FIXED_RVA: usize = rva(0x636610); // dword_636610

// ---- Rust version ----

pub unsafe fn fog_polygon() -> Result<(), Refused> {
    unsafe {
        let entries = listed()?;
        if entries.is_empty() {
            return Err(BadInput("a polygon of no vertices").into());
        }
        if entries.iter().any(|&entry| dword(entry + 4) == 0) {
            return Err(BadInput("an entry without its record").into());
        }
        let length: u32 = game_read(LENGTH_RVA);
        if length > TABLE_LEN {
            return Err(BadInput("a fog table longer than its room").into());
        }
        let fixed = (game_read::<u32>(KIND_RVA) == 4).then(|| game_read::<u32>(FIXED_RVA));
        for &entry in &entries {
            let record = dword(entry + 4);
            let depth = x87::to_int(dword(record as usize + 0x18));
            game_write(CHECKED_RVA, depth);
            // A place past the length is the last one; with no length (the menus)
            // that is the byte before the table, as in the original.
            let place = depth.max(0) as u32 >> 13;
            let place = if (place.wrapping_sub(length) as i32) < 0 {
                place
            } else {
                length.wrapping_sub(1)
            };
            let fog =
                ((game_addr(TABLE_RVA) as u32).wrapping_add(place) as *const u8).read() as u32;
            ((entry + 0x20) as *mut u8).write((fixed.unwrap_or(fog) ^ 0xFF) as u8);
        }
        Ok(())
    }
}
