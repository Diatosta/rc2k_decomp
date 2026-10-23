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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};

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
// Converted: `rc2k_core::marks::expiry::{cell, hexagon_draw, hexagon_hit}`
pub unsafe fn expire_hexagons(entry: &Regs) -> Regs {
    use rc2k_core::marks::expiry;
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
            let f = |o: u32| f32::from_bits(((e + o) as *const u32).read_unaligned());
            let c = expiry::cell(f(0x14), f(0x18));
            game_write(PLACE_RVA[1], c.z);
            game_write(PLACE_RVA[0], c.x);
            g.edx = c.at;
            let (r, draw) = expiry::hexagon_draw(game_read(RANDOM_RVA));
            game_write(RANDOM_RVA, r);
            g.ecx = draw;
            if g.ecx == 0 {
                (e as *mut u32).write_unaligned(0);
            }
            let (hit, last) = expiry::hexagon_hit(
                game_read(STEPS_RVA),
                |k| {
                    let at = game_read::<u32>(LATEST_RVA).wrapping_add(k) & 0xFF;
                    game_read(VALUES_RVA + 4 * at as usize)
                },
                |n| game_read(MASKS_RVA[n] + c.at as usize),
            );
            if let Some(v) = last {
                g.eax = v;
            }
            if hit {
                (e as *mut u32).write_unaligned(0);
            }
        }
        g.ebp = base + ITEMS * ITEM_LEN;
    }
    g
}
