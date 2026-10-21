//! `marks::expire_squares` (0x4C06AD): each live square item (`g_marksSquareItems`, 0x800 of
//! 0x3C bytes, +0 1 live) ends when its cell's byte in `g_marksMaskA` (or, with
//! `g_marksMaskRunCount` above 1, in `g_marksMaskB`) is the low byte of `g_marksShapesA` (the
//! cell from its place +0x20 / +0x24, `fistp`ed through `dword_81471C` / `dword_814720`,
//! >> 3).
//!
//! In:  nothing
//! Out: eax / ebx `g_marksShapesA` / `B`; ecx / edx the last live item's cell parts; ebp
//!      past the items
//!
//! Caller: `marks::squares_step` (0x4C0682). Self-test: the original and the Rust version on
//! the game's items with a few made live, shapes and mask counts by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::expire_squares", rva(0x4C06AD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const ITEMS_RVA: usize = rva(0x89E18A); // g_marksSquareItems
pub const ITEMS: u32 = 0x800;
pub const ITEM_LEN: u32 = 0x3C;
const SHAPES_RVA: [usize; 2] = [rva(0x82FC56), rva(0x82FC7A)]; // g_marksShapesA / B
const PLACE_RVA: [usize; 2] = [rva(0x81471C), rva(0x814720)]; // dword_81471C, dword_814720
const RUNS_RVA: usize = rva(0x833D7E); // g_marksMaskRunCount
const MASKS_RVA: [usize; 2] = [rva(0x833DC2), rva(0x83BDC2)]; // g_marksMaskA / B

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::expiry::{cell, square_ends}`
pub unsafe fn expire_squares(entry: &Regs) -> Regs {
    use rc2k_core::marks::expiry;
    let mut g = *entry;
    unsafe {
        g.eax = game_read(SHAPES_RVA[0]);
        g.ebx = game_read(SHAPES_RVA[1]);
        let al = g.eax as u8;
        let base = game_addr(ITEMS_RVA) as u32;
        for n in 0..ITEMS {
            let e = base + n * ITEM_LEN;
            if (e as *const u32).read_unaligned() != 1 {
                continue;
            }
            let f = |o: u32| f32::from_bits(((e + o) as *const u32).read_unaligned());
            let c = expiry::cell(f(0x20), f(0x24));
            game_write(PLACE_RVA[1], c.z);
            game_write(PLACE_RVA[0], c.x);
            [g.ecx, g.edx] = [c.column, c.at];
            if expiry::square_ends(
                al,
                || game_read(RUNS_RVA),
                |n| game_read(MASKS_RVA[n] + c.at as usize),
            ) {
                (e as *mut u32).write_unaligned(0);
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
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { expire_squares(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The game's items with a few made live (seeded), for this test and `squares_step`'s.
pub unsafe fn test_items(case: u32) {
    unsafe {
        let base = game_addr(ITEMS_RVA) as u32;
        std::ptr::write_bytes(base as *mut u8, 0, (ITEMS * ITEM_LEN) as usize);
        for n in 0..40u32 {
            let e = base + (n * 41 + case) % ITEMS * ITEM_LEN;
            (e as *mut u32).write_unaligned(1);
            ((e + 0x20) as *mut u32).write_unaligned(((n * 53 + case * 7) as f32 * 3.3).to_bits());
            ((e + 0x24) as *mut u32).write_unaligned(((n * 29) as f32 * 7.1).to_bits());
        }
        game_write(
            SHAPES_RVA[0],
            0x0101_0100u32.wrapping_mul(case) | (case * 37) & 0xFF,
        );
        game_write(RUNS_RVA, case % 3);
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([SHAPES_RVA[0], SHAPES_RVA[1], RUNS_RVA].map(at));
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
