//! `marks::expire_hexagons` (0x4BCB1D): each live hexagon item (`g_marksHexagonItems`, 0x800
//! of 0x30 bytes, +0 1 live) ends one time in 0x800 (its own random `dword_82E2B6`, times
//! 0xC5 plus 0x5D, bits 16 .. 26 0), or when its cell's byte in `g_marksMaskA` or
//! `g_marksMaskB` (the cell from its place +0x14 / +0x18, `fistp`ed through `dword_81471C` /
//! `dword_814720`, >> 3) is the low byte of one of the last `dword_81622E` steps' values
//! (`dword_844A10`[(`dword_84460C` + k) & 0xFF], the latest first).
//!
//! In:  nothing
//! Out: eax / ebx `g_marksShapesA` / `B`, ecx `dword_844604` going in (eax then as the last test left it), ecx /
//!      edx the last live item's random bits and cell, ebp past the items
//!
//! Caller: `marks::hexagons_step` (0x4BCAF1). Self-test: the original and the Rust version
//! on the game's items with a few made live, places, steps and randoms by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::expire_hexagons", rva(0x4BCB1D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const ITEMS_RVA: usize = rva(0x81623E); // g_marksHexagonItems
pub const ITEMS: u32 = 0x800;
pub const ITEM_LEN: u32 = 0x30;
const SHAPES_RVA: [usize; 2] = [rva(0x82FC56), rva(0x82FC7A)]; // g_marksShapesA / B
const ECX_RVA: usize = rva(0x844604); // dword_844604
const PLACE_RVA: [usize; 2] = [rva(0x81471C), rva(0x814720)]; // dword_81471C, dword_814720
const RANDOM_RVA: usize = rva(0x82E2B6); // dword_82E2B6
pub const STEPS_RVA: usize = rva(0x81622E); // dword_81622E
pub const LATEST_RVA: usize = rva(0x84460C); // dword_84460C
const VALUES_RVA: usize = rva(0x844A10); // dword_844A10: 0x100
const MASKS_RVA: [usize; 2] = [rva(0x833DC2), rva(0x83BDC2)]; // g_marksMaskA / B

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn expire_hexagons(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(SHAPES_RVA[0]);
        g.ebx = game_read(SHAPES_RVA[1]);
        g.ecx = game_read(ECX_RVA);
        let base = game_addr(ITEMS_RVA) as u32;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            g.ebp = e;
            if (e as *const u32).read_unaligned() != 1 {
                continue;
            }
            let z = x87::to_int(((e + 0x18) as *const u32).read_unaligned());
            let x = x87::to_int(((e + 0x14) as *const u32).read_unaligned());
            game_write(PLACE_RVA[1], z);
            game_write(PLACE_RVA[0], x);
            g.edx = ((((z as u32) >> 3) & 0x7F) << 8) + (((x as u32) >> 3) & 0xFF);
            let r = game_read::<u32>(RANDOM_RVA)
                .wrapping_mul(0xC5)
                .wrapping_add(0x5D);
            game_write(RANDOM_RVA, r);
            g.ecx = r.rotate_right(16) & 0x7FF;
            if g.ecx == 0 {
                (e as *mut u32).write_unaligned(0);
            }
            let steps = game_read::<u32>(STEPS_RVA) as i32;
            for k in (0..steps.max(0) as u32).rev() {
                g.eax = game_read::<u32>(LATEST_RVA).wrapping_add(k) & 0xFF;
                g.eax = game_read(VALUES_RVA + 4 * g.eax as usize);
                let al = g.eax as u8;
                let hit = game_read::<u8>(MASKS_RVA[0] + g.edx as usize) == al
                    || game_read::<u8>(MASKS_RVA[1] + g.edx as usize) == al;
                if hit {
                    (e as *mut u32).write_unaligned(0);
                    break;
                }
            }
        }
        g.ebp = base + ITEMS * ITEM_LEN;
    }
    g
}

pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(ITEMS_RVA) as u32, (ITEMS * ITEM_LEN) as usize),
        at(PLACE_RVA[0]),
        at(PLACE_RVA[1]),
        at(RANDOM_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { expire_hexagons(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The game's items with a few made live (seeded), for this test and `hexagons_step`'s.
pub unsafe fn test_items(case: u32) {
    unsafe {
        let base = game_addr(ITEMS_RVA) as u32;
        std::ptr::write_bytes(base as *mut u8, 0, (ITEMS * ITEM_LEN) as usize);
        for n in 0..40u32 {
            let e = base + (n * 37 + case) % ITEMS * ITEM_LEN;
            (e as *mut u32).write_unaligned(1);
            ((e + 0x14) as *mut u32).write_unaligned(((n * 53 + case * 7) as f32 * 3.3).to_bits());
            ((e + 0x18) as *mut u32).write_unaligned(((n * 29) as f32 * 7.1).to_bits());
        }
        game_write(RANDOM_RVA, case.wrapping_mul(0x2545_F491));
        game_write(STEPS_RVA, [0u32, 1, 3, 9][case as usize % 4]);
        game_write(LATEST_RVA, case * 5);
        for n in 0..0x100usize {
            game_write(VALUES_RVA + 4 * n, (n as u32 * 0x0101_0007) ^ case);
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([STEPS_RVA, LATEST_RVA].map(at));
    vars.push((game_addr(VALUES_RVA) as u32, 0x400));
    let saved = capture(&vars);
    for case in 0..6u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                test_items(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
