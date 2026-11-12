//! `hud::draw_disc` (0x4B6C4E): the HUD's disc of 16 triangles: the ring's 16 entries (1 .. 0x10 of
//! `HudBlock.ring_sines[0]` / `HudBlock.ring_cosines[0]`) from `g_sines` / `g_cosines` at k * 0x100, and of the texture ring
//! (`HudBlock.texture_ring_sines[0]` / `HudBlock.texture_ring_cosines[0]`) at k * 0x100 less `HudBlock.disc_turn` times `HudBlock.disc_turn_a` times
//! `HudBlock.disc_turn_b` (`fistp`, & 0xFFF); then each triangle (the centre, entry k + 1, entry k + 2
//! wrapping past 0x10) from `hud::disc_vertex` and drawn by `hud::add_triangle`.
//!
//! Callers: 0x4B6B0A, 0x4B6B5B, 0x4B6BAC, 0x4B6BFD.
//!
//! the ring's angles and the triangles' entries through
//! `rc2k_core::hud::disc` (whole numbers; the sines are copied); the texture's turn stays
//! the x87's (its rounding follows the control word), worked out once (the original works
//! the same value out sixteen times into `HudBlock.fistp_scratch`).

use crate::errors::Refused;
use crate::hud::disc_vertex::disc_vertex;
use crate::hud::owner::{Hud, Pen};
use crate::math::x87;
use crate::math::x87_float::{ld, to_int};
use crate::race::owner_frame::Frame;
use rc2k_core::hud::disc::SEGMENTS;

/// `fld a; fchs; fmul b; fmul c; fistp`.
fn negated_times_times_to_int(a: f32, b: f32, c: f32) -> u32 {
    to_int((-ld(a) * ld(b)) * ld(c)) as u32
}

// Converted: `rc2k_core::hud::disc::{ring_angles, disc_triangle}`
pub fn draw_disc(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    {
        use rc2k_core::hud::disc::{disc_triangle, ring_angles};
        // The texture's turn follows the x87's control word: the x87's.
        let t = negated_times_times_to_int(
            hud.block.disc_turn.get(),
            hud.block.disc_turn_a.get(),
            hud.block.disc_turn_b.get(),
        );
        let table = |cell: &crate::session::Float| x87::loaded(cell);
        for (k, [a, b]) in ring_angles(t).into_iter().enumerate() {
            let n = k + 1;
            hud.block.ring_sines[n].set(table(crate::scene::engine_tables_store::sine(
                pen.engine_tables,
                a as usize,
            )));
            hud.block.ring_cosines[n].set(table(crate::scene::engine_tables_store::cosine(
                pen.engine_tables,
                a as usize,
            )));
            hud.block.texture_ring_sines[n].set(table(crate::scene::engine_tables_store::sine(
                pen.engine_tables,
                b as usize,
            )));
            hud.block.texture_ring_cosines[n].set(table(
                crate::scene::engine_tables_store::cosine(pen.engine_tables, b as usize),
            ));
        }
        hud.block.fistp_scratch.set(t);
        for k in 0..SEGMENTS {
            for (n, e) in disc_triangle(k).into_iter().enumerate() {
                disc_vertex(hud, e, &pen.vertices.vertices[n])?;
            }
            crate::hud::add_triangle::add_triangle(frame, hud, pen)?;
        }
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// `negated_times_times_to_int` over the test's operand buffers (`[v + 4 k]`, the result
/// at `[o]`), for the comparison with the original's x87 run.
#[cfg(test)]
fn x87_1(v: impl crate::scene::x87_mem::Base, o: impl crate::scene::x87_mem::Base) {
    o.set_dword(
        0,
        negated_times_times_to_int(
            f32::from_bits(v.dword(0)),
            f32::from_bits(v.dword(4)),
            f32::from_bits(v.dword(8)),
        ),
    );
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Block, In, Operand, Out, sparse};

    // The original's instructions, one function per run.

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
            plain: |a| x87_1(a[0].cells(), a[1].cells()),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("hud.draw_disc", &blocks(), 1200, 100);
    }
}
