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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::expire_hexagons::{ITEM_LEN, ITEMS, ITEMS_RVA};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::spawn_hexagon", rva(0x4BC22B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(ITEMS_RVA) as u32, (ITEMS * ITEM_LEN) as usize),
        at(CURSOR_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { spawn_hexagon_plain(r.eax, r.ebx, r.ecx) };
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    for case in 0..6u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs {
                eax: 0x123 + case,
                ebx: 0x3FF - case,
                ecx: case % 8,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &saved);
                let base = game_addr(ITEMS_RVA) as u32;
                std::ptr::write_bytes(base as *mut u8, 0xAA, (ITEMS * ITEM_LEN) as usize);
                // Items live but some, the window full in case 3, a wrap in case 4.
                let start = if case == 4 { ITEMS - 3 } else { case * 100 };
                for n in 0..ITEMS {
                    let free = case != 3 && n == (start + case * 3 + 2) % ITEMS;
                    ((base + n * ITEM_LEN) as *mut u32).write_unaligned(u32::from(!free));
                }
                game_write(CURSOR_RVA, start * ITEM_LEN);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
