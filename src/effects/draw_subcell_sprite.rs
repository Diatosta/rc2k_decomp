//! `effects::draw_subcell_sprite` (0x4CBF47): a ring entry's square sprite from a subcell: the
//! place (st(0) x, st(1) y, st(2) z) turned by the view matrix into the draw work's place (the
//! depth also whole in the ring's `whole`); unless the depth is at or below the nearest depth
//! (a signed compare of the bits): the four vertices' specular `scene::fog_at` of the depth,
//! colour the entry's +0x90 | 0xFF000000, texture places from the subcell's entry in the
//! subcell places, depth times the depth scale; half the side (the size's bits as a float,
//! times the view scale / depth); the corners about the projected place; `effects::clip_quad`,
//! and unless all is outside `scene::draw_fan` and the polygon's vertex count 4.
//!
//! Callers: the ring's kinds 0x10, 0x11 (`effects::run_ring`) and the effects' lines.
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the game's variables. Refuses,
//! before anything, a subcell past the table's 0x410, a null entry, and the fog's and the
//! clip's refusals.

use crate::effects::cell_x87::CellX87;
use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::ring_store::RingWork;
use crate::errors::{BadInput, Refused};
use crate::scene::fog_at::fog_at;

const SUBCELLS: u32 = 0x410;

/// The place turned by the view.
fn turn(
    view_matrix: &[crate::session::Float; 16],
    ring: &Ring,
    p: &[[u8; 10]; 3],
    effects: &crate::effects::owner::Effects,
) {
    turn_x87(view_matrix, p, &ring.work, &effects.frame.draw_work);
}

/// The square about the projected place.
fn square(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    effects: &crate::effects::owner::Effects,
) {
    square_x87(buffer, &effects.frame.draw_work);
}

/// `subcell` is the subcell's number, `size` a float's bits, `entry` the ring entry.
#[allow(
    clippy::too_many_arguments,
    reason = "the original's registers, the effects' groups and the owners"
)]
pub fn draw_subcell_sprite(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    subcell: u32,
    size: u32,
    entry: u32,
    place: [[u8; 10]; 3],
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    if subcell >= SUBCELLS {
        return Err(BadInput("a mark subcell past g_marksSubcellPlaces").into());
    }
    if entry == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    fog_at(view.fog, 0)?;
    if effects.setup.settings.is_set.get() == 0 {
        return Err(BadInput("no marks settings").into());
    }
    turn(view.view_matrix, ring, &place, effects);
    if effects.frame.draw_work.turned[2].bits() as i32 <= effects.frame.draw_work.near.bits() as i32
    {
        return Ok(());
    }
    let fogged = fog_at(view.fog, ring.work.whole.get())?;
    let colour = crate::effects::ring_entries::entry(ring, entry)?.car_tag | 0xFF00_0000;
    for n in 0..4 {
        buffer.vertices[n].specular.set(fogged);
        buffer.vertices[n].colour.set(colour);
    }
    effects.frame.draw_work.half[0].set_bits(size);
    let at = subcell << 4;
    let place = |n: usize| masks.surface.word(0x400 + at as usize + 4 * n);
    let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
    for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
        .into_iter()
        .enumerate()
    {
        buffer.vertices[n].u.set_bits(u);
        buffer.vertices[n].v.set_bits(v);
    }
    square(buffer, effects);
    if clip_quad(buffer, effects)? {
        return Ok(());
    }
    draw_fan(buffer, scene);
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set_bits(4u32);
    Ok(())
}

use crate::effects::frame_state::DrawWork;
use crate::effects::plain::st_cell;
use crate::math::x87_float as bh;
use crate::math::x87_float::ld;

/// `turn`'s x87 block as plain Rust on the x87 model (37 instructions), same order and memory places.
fn turn_x87(
    view_matrix: &[crate::session::Float; 16],
    p: &[[u8; 10]; 3],
    w: &RingWork,
    dw: &DrawWork,
) {
    let t0 = bh::from_f80(p[2]);
    let t1 = bh::from_f80(p[1]);
    let t2 = bh::from_f80(p[0]);
    let t3 = view_matrix[0].ld();
    let t4 = t3 * t2;
    let t5 = view_matrix[4].ld();
    let t6 = t5 * t1;
    let t7 = view_matrix[8].ld();
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = t10 + view_matrix[12].ld();
    let t12 = view_matrix[1].ld();
    let t13 = t12 * t2;
    let t14 = view_matrix[5].ld();
    let t15 = t14 * t1;
    let t16 = view_matrix[9].ld();
    let t17 = t16 * t0;
    let t18 = t15 + t17;
    let t19 = t13 + t18;
    let t20 = t19 + view_matrix[13].ld();
    let t21 = view_matrix[2].ld();
    let t22 = t2 * t21;
    let t23 = view_matrix[6].ld();
    let t24 = t1 * t23;
    let t25 = view_matrix[10].ld();
    let t26 = t0 * t25;
    let t27 = t26 + t24;
    let t28 = t22 + t27;
    let t29 = t28 + view_matrix[14].ld();
    dw.turned[0].st(t11);
    dw.turned[1].st(t20);
    w.whole.set_bits(bh::to_int(t29) as u32);
    dw.turned[2].st(t29);
}

/// `square`'s x87 block as plain Rust on the x87 model (41 instructions), same order and memory places.
fn square_x87(buffer: &crate::d3d::vertices_store::PolygonVertices, dw: &DrawWork) {
    let t0 = ld(crate::scene::polygon_work::front_end()
        .window
        .view_scale
        .get());
    let t1 = t0 / dw.turned[2].ld();
    let t2 = dw.turned[2].ld();
    let t3 = t2 * dw.z_k.ld();
    st_cell(&buffer.vertices[0].z, t3);
    st_cell(&buffer.vertices[1].z, t3);
    st_cell(&buffer.vertices[2].z, t3);
    st_cell(&buffer.vertices[3].z, t3);
    let t4 = dw.half[0].ld();
    let t5 = t4 * t1;
    dw.half[0].st(t5);
    dw.half[1].st(t5);
    let t6 = t1 * dw.turned[0].ld();
    let t7 = t1 * dw.turned[1].ld();
    let t8 = -t7;
    let t9 = t8 + crate::scene::polygon_work::front_end().window.view_centre[1].ld();
    let t10 = t9 - dw.half[1].ld();
    st_cell(&buffer.vertices[0].y, t10);
    let t11 = t9 - dw.half[1].ld();
    st_cell(&buffer.vertices[1].y, t11);
    let t12 = t9 + dw.half[1].ld();
    st_cell(&buffer.vertices[2].y, t12);
    let t13 = t9 + dw.half[1].ld();
    st_cell(&buffer.vertices[3].y, t13);
    let t14 = t6 + crate::scene::polygon_work::front_end().window.view_centre[0].ld();
    let t15 = t14 - dw.half[0].ld();
    st_cell(&buffer.vertices[0].x, t15);
    let t16 = t14 + dw.half[0].ld();
    st_cell(&buffer.vertices[1].x, t16);
    let t17 = t14 + dw.half[0].ld();
    st_cell(&buffer.vertices[2].x, t17);
    let t18 = t14 - dw.half[0].ld();
    st_cell(&buffer.vertices[3].x, t18);
}

#[cfg(test)]
mod reference_vectors {
    use crate::d3d::vertices_store::VERTICES;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, recorded_writes, with_copy};
    use crate::effects::frame_state::DrawWork;
    use crate::effects::ring_store::RingWork;

    /// The vertex float at byte `off` of the vertices (x, y or z of a vertex).
    fn vertex_cell(off: usize) -> &'static crate::session::Float {
        let v = &VERTICES.vertices[off / 32];
        match off % 32 {
            0x0 => &v.x,
            0x4 => &v.y,
            0x8 => &v.z,
            _ => panic!("not a tested vertex field"),
        }
    }

    /// Runs `f` with the vertex floats at `offsets` holding the test buffer's (at `at`)
    /// bytes; what `f` leaves goes back to the buffer, the vertices as they were.
    fn staged<R>(
        at: crate::effects::asm_test::Operand,
        offsets: &[usize],
        f: impl FnOnce() -> R,
    ) -> R {
        let host = |off: usize| (at + off).host().cast::<u32>();
        let saved: Vec<u32> = offsets.iter().map(|&o| vertex_cell(o).bits()).collect();
        for &o in offsets {
            vertex_cell(o).set_bits(unsafe { host(o).read_unaligned() });
        }
        let out = f();
        for (&o, &old) in offsets.iter().zip(&saved) {
            unsafe { host(o).write_unaligned(vertex_cell(o).bits()) };
            vertex_cell(o).set_bits(old);
        }
        out
    }

    #[test]
    fn turn_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[
                        (0x1EA2CC, F32),
                        (0x1EA2D0, F32),
                        (0x1EA2D4, F32),
                        (0x1EA2DC, F32),
                        (0x1EA2E0, F32),
                        (0x1EA2E4, F32),
                        (0x1EA2EC, F32),
                        (0x1EA2F0, F32),
                        (0x1EA2F4, F32),
                        (0x1EA2FC, F32),
                        (0x1EA300, F32),
                        (0x1EA304, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x186931,
                    hi: 0x1D0826,
                    reads: &[],
                    writes: &[
                        (0x186995, F32),
                        (0x186999, F32),
                        (0x18699D, F32),
                        (0x1D0780, I32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x1E,
                    reads: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_subcell_sprite.turn_x87", &SPEC, |b| {
            with_copy(b[1], RingWork::START, |w| {
                with_copy::<DrawWork, _>(b[1], DrawWork::START, |dw| {
                    let place: [[u8; 10]; 3] = std::array::from_fn(|n| unsafe {
                        (b[2] + 10 * n).host().cast::<[u8; 10]>().read_unaligned()
                    });
                    super::turn_x87(
                        &crate::race::owner::race().frame.matrices.view_matrix,
                        &place,
                        w,
                        dw,
                    );
                });
            });
        });
    }

    #[test]
    fn square_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x6C,
                    reads: &[],
                    writes: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0x20, F32),
                        (0x24, F32),
                        (0x28, F32),
                        (0x40, F32),
                        (0x44, F32),
                        (0x48, F32),
                        (0x60, F32),
                        (0x64, F32),
                        (0x68, F32),
                    ],
                },
                Op::Base {
                    lo: 0x186931,
                    hi: 0x186A31,
                    reads: &[
                        (0x186975, F32),
                        (0x186979, F32),
                        (0x186995, F32),
                        (0x186999, F32),
                        (0x18699D, F32),
                        (0x186A2D, F32),
                    ],
                    writes: &[(0x186975, F32), (0x186979, F32)],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_subcell_sprite.square_x87", &SPEC, |b| {
            with_copy::<DrawWork, _>(b[2], DrawWork::START, |dw| {
                staged(
                    b[1],
                    &[
                        0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                    ],
                    || super::square_x87(&crate::d3d::vertices_store::VERTICES, dw),
                );
            });
        });
    }
}
