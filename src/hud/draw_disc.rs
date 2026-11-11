//! `hud::draw_disc` (0x4B6C4E): the HUD's disc of 16 triangles: the ring's 16 entries (1 .. 0x10 of
//! `flt_75C188` / `flt_75C1CC`) from `g_sines` / `g_cosines` at k * 0x100, and of the texture ring
//! (`flt_75C210` / `flt_75C254`) at k * 0x100 less `dword_75C05C` times `flt_75C028` times
//! `flt_75C024` (`fistp`, & 0xFFF); then each triangle (the centre, entry k + 1, entry k + 2
//! wrapping past 0x10) from `hud::disc_vertex` and drawn by `hud::add_triangle`.
//!
//! Callers: 0x4B6B0A, 0x4B6B5B, 0x4B6BAC, 0x4B6BFD.
//!
//! the ring's angles and the triangles' entries through
//! `rc2k_core::hud::disc` (whole numbers; the sines are copied); the texture's turn stays
//! the x87's (its rounding follows the control word), worked out once (the original works
//! the same value out sixteen times into `dword_75C060`).

use crate::errors::Refused;
use crate::hud::disc_vertex::disc_vertex;
use crate::hud::owner::Hud;
use crate::math::x87;
use crate::math::x87_float::{ld, to_int};
use crate::race::owner_frame::Frame;
use rc2k_core::hud::disc::SEGMENTS;

/// `fld a; fchs; fmul b; fmul c; fistp`.
fn negated_times_times_to_int(a: u32, b: u32, c: u32) -> u32 {
    to_int((-ld(a) * ld(b)) * ld(c)) as u32
}

// Converted: `rc2k_core::hud::disc::{ring_angles, disc_triangle}`
pub fn draw_disc(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    {
        use rc2k_core::hud::disc::{disc_triangle, ring_angles};
        // The texture's turn follows the x87's control word: the x87's.
        let t = negated_times_times_to_int(
            hud.block.disc_turn.bits(),
            hud.block.disc_turn_a.bits(),
            hud.block.disc_turn_b.bits(),
        );
        let table = |cell: &crate::session::Float| x87::loaded(cell);
        for (k, [a, b]) in ring_angles(t).into_iter().enumerate() {
            let n = k + 1;
            hud.block.ring_sines[n]
                .set_bits(table(crate::scene::engine_tables_store::sine(a as usize)).to_bits());
            hud.block.ring_cosines[n]
                .set_bits(table(crate::scene::engine_tables_store::cosine(a as usize)).to_bits());
            hud.block.texture_ring_sines[n]
                .set_bits(table(crate::scene::engine_tables_store::sine(b as usize)).to_bits());
            hud.block.texture_ring_cosines[n]
                .set_bits(table(crate::scene::engine_tables_store::cosine(b as usize)).to_bits());
        }
        hud.block.fistp_scratch.set(t);
        for k in 0..SEGMENTS {
            for (n, e) in disc_triangle(k).into_iter().enumerate() {
                disc_vertex(hud, e, &crate::d3d::vertices_store::VERTICES.vertices[n])?;
            }
            crate::hud::add_triangle::add_triangle(frame, hud)?;
        }
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// `negated_times_times_to_int` over the test's operand buffers (`[v + 4 k]`, the result
/// at `[o]`), for the comparison with the original's x87 run.
#[cfg(all(test, target_arch = "x86"))]
fn x87_1(v: usize, o: usize) {
    use crate::scene::x87_mem::Base;
    o.set_dword(
        0,
        negated_times_times_to_int(v.dword(0), v.dword(4), v.dword(8)),
    );
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
