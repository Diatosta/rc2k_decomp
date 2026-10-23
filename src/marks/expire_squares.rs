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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};

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
