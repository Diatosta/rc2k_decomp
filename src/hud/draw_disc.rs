//! `hud::draw_disc` (0x4B6C4E): the HUD's disc of 16 triangles: the ring's 16 entries (1 ..
//! 0x10 of `flt_75C188` / `flt_75C1CC`) from `g_sines` / `g_cosines` at k * 0x100, and of
//! the texture ring (`flt_75C210` / `flt_75C254`) at k * 0x100 less `dword_75C05C` times
//! `flt_75C028` times `flt_75C024` (`fistp`, & 0xFFF); then each triangle (the centre,
//! entry k + 1, entry k + 2 wrapping past 0x10) from `hud::disc_vertex` and drawn by
//! `hud::add_triangle`.
//!
//! In:  nothing
//! Out: eax the last texture angle, ebx 0x10, ecx 0x10, edx 0x10; the rest kept
//!
//! Callers: `sub_4B6B0A`, `sub_4B6B5B`, `sub_4B6BAC`, `sub_4B6BFD`.
//!
//! Converted 2026-10-21: the ring's angles and the triangles' entries through
//! `rc2k_core::hud::disc` (whole numbers; the sines are copied); the texture's turn stays
//! the x87's (its rounding follows the control word), worked out once (the original works
//! the same value out sixteen times into `dword_75C060`).

use crate::errors::Refused;
use crate::hud::disc_vertex::disc_vertex;
use crate::hud::store::HUD;
use crate::math::x87;
use crate::scene::x87_mem::{fistp, fld};
use rc2k_core::hud::disc::SEGMENTS;

/// `fld a; fchs; fmul b; fmul c; fistp`.
fn negated_times_times_to_int(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    x87_1(
        (crate::pointers::game_address(v.as_ptr())) as usize,
        crate::pointers::game_address(&raw mut out) as usize,
    );
    out
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::disc::{ring_angles, disc_triangle}`
pub fn draw_disc() -> Result<(), Refused> {
    {
        use rc2k_core::hud::disc::{disc_triangle, ring_angles};
        // The texture's turn follows the x87's control word: the x87's.
        let t = negated_times_times_to_int(
            HUD.disc_turn.bits(),
            HUD.disc_turn_a.bits(),
            HUD.disc_turn_b.bits(),
        );
        let table = |cell: &crate::session::Float| x87::loaded(cell.bits());
        for (k, [a, b]) in ring_angles(t).into_iter().enumerate() {
            let n = k + 1;
            HUD.ring_sines[n].set_bits(table(crate::scene::engine_tables_store::sine(a as usize)));
            HUD.ring_cosines[n]
                .set_bits(table(crate::scene::engine_tables_store::cosine(a as usize)));
            HUD.texture_ring_sines[n]
                .set_bits(table(crate::scene::engine_tables_store::sine(b as usize)));
            HUD.texture_ring_cosines[n]
                .set_bits(table(crate::scene::engine_tables_store::cosine(b as usize)));
        }
        HUD.scratch.set(t);
        let base = crate::d3d::vertices_store::piece_addr();

        for k in 0..SEGMENTS {
            for (n, e) in disc_triangle(k).into_iter().enumerate() {
                disc_vertex(e, base + 0x20 * n as u32)?;
            }
            crate::hud::add_triangle::add_triangle()?;
        }
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = (-[v + 0] * [v + 4]) * [v + 8]`
fn x87_1(v: usize, o: usize) {
    let t0 = fld(v, 0);
    let t1 = -t0;
    let t2 = fld(v, 4);
    let t3 = t1 * t2;
    let t4 = fld(v, 8);
    let t5 = t3 * t4;
    fistp(o, 0, t5);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(v, o) "fld dword ptr [{v}]", "fchs", "fmul dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]", "fistp dword ptr [{o}]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "draw_disc line 38",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F), (8, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::I)],
                },
            ],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0], a[1]) },
            plain: |a| x87_1(a[0], a[1]),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
