//! The HUD's disc of 16 triangles: the ring's 16 entries from the sine and cosine tables at k *
//! 0x100, and of the texture ring at k * 0x100 less `disc_turn` times `disc_turn_a` times
//! `disc_turn_b` (rounded to a whole number, & 0xFFF); then each triangle (the centre, entries k +
//! 1, k + 2 wrapping past 0x10) from `hud::disc_vertex` and drawn by `hud::add_triangle`. The
//! ring's angles and the triangles' entries go through `rc2k_core::hud::disc`; the texture's turn
//! stays the x87's (its rounding follows the control word), worked out once.

use crate::errors::Refused;
use crate::hud::disc_vertex::disc_vertex;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::math::x87;
use crate::math::x87_float::{ld, to_int};
use crate::race::owner_frame::Frame;
use rc2k_core::hud::disc::SEGMENTS;

/// -a x b x c as a whole number, rounded as the x87 rounds.
fn negated_times_times_to_int(a: f32, b: f32, c: f32) -> u32 {
    to_int((-ld(a) * ld(b)) * ld(c)) as u32
}

/// The disc's texture turn the next draw uses, as `draw_disc` works it out (4096 to a turn).
pub fn texture_turn(hud: &Hud) -> u32 {
    negated_times_times_to_int(
        hud.block.disc_turn.get(),
        hud.block.disc_turn_a.get(),
        hud.block.disc_turn_b.get(),
    )
}

pub fn draw_disc(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    {
        use rc2k_core::hud::disc::{disc_triangle, ring_angles};
        // The texture's turn follows the x87's control word: the x87's.
        let t = negated_times_times_to_int(
            hud.block.disc_turn.get(),
            hud.block.disc_turn_a.get(),
            hud.block.disc_turn_b.get(),
        );
        let table = |cell: &crate::memory::session::PlainFloat| x87::loaded(cell);
        for (k, [a, b]) in ring_angles(t).into_iter().enumerate() {
            let n = k + 1;
            hud.block.ring_sines[n].set(table(crate::math::engine_tables::sine(
                pen.engine_tables,
                a as usize,
            )));
            hud.block.ring_cosines[n].set(table(crate::math::engine_tables::cosine(
                pen.engine_tables,
                a as usize,
            )));
            hud.block.texture_ring_sines[n].set(table(crate::math::engine_tables::sine(
                pen.engine_tables,
                b as usize,
            )));
            hud.block.texture_ring_cosines[n].set(table(crate::math::engine_tables::cosine(
                pen.engine_tables,
                b as usize,
            )));
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

/// `negated_times_times_to_int` over the test's operand buffers (`[v + 4 k]`, the result at
/// `[o]`), for the comparison with the recorded x87 run.
#[cfg(test)]
fn x87_1(v: impl crate::math::x87_mem::Base, o: impl crate::math::x87_mem::Base) {
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

    // The recorded run, one function per run.

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
