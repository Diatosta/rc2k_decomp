//! `marks::spawn_hexagon` (0x4BC22B): a hexagon item made live: of the 32 items from the
//! cursor `dword_81517E` on (wrapping at 0x800), the first free one (+0 0) gets +0 1, its
//! place (eax, ebx as floats at +0x14, +0x18), its kind ecx (+4) and the rest 0; the cursor
//! the item after it. With none free, the cursor moves on 32.
//!
//! In:  eax x, ebx z, ecx the kind
//! Out: every register kept
//!
//! Caller: `marks::pick_hexagon`. Self-test: the original and the Rust version on the
//! game's items (as `expire_hexagons`'), cursors and fillings by lot, a full window among
//! them; put back.
//!
//! Converted 2026-10-21: the hook's Rust version is `spawn_hexagon_plain`, through
//! `rc2k_core::marks::items::find_free`; compared exactly as before (the earlier version
//! stays for the Rust callers in unconverted code).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::marks::expire_hexagons::{ITEM_LEN, ITEMS, ITEMS_RVA};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

pub const CURSOR_RVA: usize = rva(0x81517E); // dword_81517E: an offset into the items
const WINDOW: u32 = 32;

/// The Rust version (see the header).
pub unsafe fn spawn_hexagon(x: u32, z: u32, kind: u32) {
    unsafe {
        let base = game_addr(ITEMS_RVA) as u32;
        let end = ITEMS * ITEM_LEN;
        let mut at = game_read::<u32>(CURSOR_RVA);
        for _ in 0..WINDOW {
            let e = base.wrapping_add(at);
            if (e as *const u32).read_unaligned() == 0 {
                let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
                put(0, 1);
                put(0x14, x87::int_to_float(x as i32));
                put(0x18, x87::int_to_float(z as i32));
                put(4, kind);
                for o in [0x1C, 0x20, 0x2C, 8, 0xC, 0x10, 0x24, 0x28] {
                    put(o, 0);
                }
                at += ITEM_LEN;
                if at == end {
                    at = 0;
                }
                game_write(CURSOR_RVA, at);
                return;
            }
            at += ITEM_LEN;
            if at == end {
                at = 0;
            }
        }
        game_write(CURSOR_RVA, at);
    }
}

/// The converted version: the hook's (and `pick_hexagon`'s).
// Converted: `rc2k_core::marks::items::find_free`
pub unsafe fn spawn_hexagon_plain(x: u32, z: u32, kind: u32) {
    use rc2k_core::marks::items::{HEXAGON_LEN, find_free};
    unsafe {
        let base = game_addr(ITEMS_RVA) as u32;
        let found = find_free(game_read(CURSOR_RVA), HEXAGON_LEN, |at| {
            (base.wrapping_add(at) as *const u32).read_unaligned() == 0
        });
        if let Some(at) = found.at {
            let e = base.wrapping_add(at);
            let put = |o: u32, v: u32| ((e + o) as *mut u32).write_unaligned(v);
            put(0, 1);
            put(0x14, (x as i32 as f32).to_bits());
            put(0x18, (z as i32 as f32).to_bits());
            put(4, kind);
            for o in [0x1C, 0x20, 0x2C, 8, 0xC, 0x10, 0x24, 0x28] {
                put(o, 0);
            }
        }
        game_write(CURSOR_RVA, found.cursor);
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { spawn_hexagon_plain(r.eax, r.ebx, r.ecx) };
    Ok(left_of(r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
