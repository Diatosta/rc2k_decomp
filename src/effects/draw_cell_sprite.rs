//! `effects::draw_cell_sprite` (0x4CB796): a ring entry's sprite: the place (st(0) x, st(1) y,
//! st(2) z) turned by the view matrix into the draw work's place (the depth also whole in the
//! ring's `whole`); unless the depth is at or below the nearest depth (a signed compare of the
//! bits): the four vertices' specular `scene::fog_at` of the depth, colour (the alpha's top
//! byte | the entry's +0x90), texture places from the cell's entry in the cell places, depth
//! times the depth scale; half the side (the size times the view scale / depth, and that
//! times the cell's corner slope) turned by the sine and cosine tables at the angle; the
//! corners about the projected place; `effects::clip_quad`, and unless all is outside
//! `scene::draw_fan` and the polygon's vertex count 4.
//!
//! Callers: the ring's kinds 1, 5, 6, 0xD, 0x13, 0x15, 0x16 (`effects::run_ring`) and one more caller.
//! The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the game's variables. Refuses,
//! before anything, a cell past 0x40, an angle of 0x1000 or more, a null entry, and the
//! fog's and the clip's refusals.

use crate::effects::cell_x87::CellX87;
use crate::effects::clip_quad::{clip_quad, draw_fan};
use crate::effects::mask_store::Masks;
use crate::effects::ring_store::Ring;
use crate::effects::ring_store::RingWork;
use crate::errors::{BadInput, Refused};
use crate::scene::fog_at::fog_at;

const CELLS: u32 = 0x41;
const ANGLES: u32 = 0x1000;

/// The place turned by the view.
fn turn(
    view_matrix: &[crate::session::Float; 16],
    ring: &Ring,
    p: &[[u8; 10]; 3],
    effects: &crate::effects::owner::Effects,
) {
    turn_x87(view_matrix, p, &ring.work, &effects.frame.draw_work);
}

/// The square, turned, about the projected place. Gives a value the original keeps in a
/// register: the draw work's `DrawWork.sprite_half[1]` cell between its two stores.
fn square(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    k: u32,
    angle: u32,
    effects: &crate::effects::owner::Effects,
) -> u32 {
    square_x87(
        engine_tables,
        buffer,
        k as usize,
        angle as usize,
        &effects.frame.draw_work,
        &effects.blocks.corner_slopes,
    )
}

/// `cell` is the cell's number, `angle` the turn, `alpha` the colour's top byte, `size` the
/// side, `entry` the ring entry.
#[allow(
    clippy::too_many_arguments,
    reason = "the original's five registers and the cell places, and now the two owners"
)]
pub fn draw_cell_sprite(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    masks: &Masks,
    ring: &Ring,
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
    cell: u32,
    angle: u32,
    alpha: u32,
    size: u32,
    entry: u32,
    place: [[u8; 10]; 3],
    view: crate::effects::run_frame::EffectsView<'_>,
) -> Result<(), Refused> {
    if cell >= CELLS {
        return Err(BadInput("a mark cell past g_marksCellPlaces").into());
    }
    if angle >= ANGLES {
        return Err(BadInput("an angle past the sine tables").into());
    }
    if entry == 0 {
        return Err(BadInput("a null ring entry").into());
    }
    // The fog's and the clip's refusals, before anything.
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
    let colour = (alpha & 0xFF00_0000) | crate::effects::ring_entries::entry(ring, entry)?.car_tag;
    for n in 0..4 {
        buffer.vertices[n].specular.set(fogged);
        buffer.vertices[n].colour.set(colour);
    }
    effects.frame.draw_work.sprite_half[0].set_bits(size);
    let at = cell << 4;
    let place = |n: usize| masks.surface.places.get_u32(at as usize + 4 * n);
    let [u0, v0, u1, v1] = [place(0), place(1), place(2), place(3)];
    for (n, (u, v)) in [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
        .into_iter()
        .enumerate()
    {
        buffer.vertices[n].u.set_bits(u);
        buffer.vertices[n].v.set_bits(v);
    }
    square(view.engine_tables, buffer, at, angle, effects);
    if clip_quad(buffer, effects)? {
        return Ok(());
    }
    draw_fan(buffer, scene);
    crate::scene::polygon_work::front_end()
        .polygon_vertex_count
        .set_bits(4u32);
    Ok(())
}

use crate::effects::blocks::CornerSlopes;
use crate::effects::frame_state::DrawWork;
use crate::effects::plain::{ld_cell, st_cell};
use crate::math::x87_float as bh;
use crate::scene::engine_tables_store::{cosine, sine};
use crate::session::PlainFloat;

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

/// The float at byte `k` of the slope records, whatever field of a record it falls on.
fn slope_at(cs: &CornerSlopes, k: usize) -> &PlainFloat {
    let record = &cs.slopes[k / 0x24];
    match (k % 0x24) / 4 {
        0 => &record.slope,
        n => &record.values[n - 1],
    }
}

/// `square`'s x87 block as plain Rust on the x87 model (77 instructions), same order and memory places.
/// `k` is the cell's offset in the cell table (16 bytes a cell), which the original also
/// adds to the corner slopes table (records of 0x24: it reads the float at that byte, whatever
/// field it falls on: section 16), so that read goes through `slope_at`. `angle` is the
/// sine table's cell (the original's `s` is four times it; the cosine is the cell 0x400 on).
fn square_x87(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    k: usize,
    angle: usize,
    dw: &DrawWork,
    cs: &CornerSlopes,
) -> u32 {
    let t0 = crate::math::x87_float::ld(
        crate::scene::polygon_work::front_end()
            .window
            .view_scale
            .get(),
    );
    let t1 = t0 / dw.turned[2].ld();
    let t2 = dw.turned[2].ld();
    let t3 = t2 * dw.z_k.ld();
    st_cell(&buffer.vertices[0].z, t3);
    st_cell(&buffer.vertices[1].z, t3);
    st_cell(&buffer.vertices[2].z, t3);
    st_cell(&buffer.vertices[3].z, t3);
    let t4 = bh::int(dw.sprite_half[0].bits() as i32);
    let t5 = t4 * t1;
    dw.sprite_half[0].st(t5);
    let t6 = t5 * ld_cell(slope_at(cs, k));
    dw.sprite_half[1].st(t6);
    let t7 = dw.sprite_half[0].bits();
    let t8 = t7 ^ 0x80000000;
    dw.sprite_half[3].set_bits(t8);
    let t9 = dw.sprite_half[1].bits();
    dw.sprite_half[2].set_bits(t9);
    let t10 = dw.sprite_half[0].ld();
    let t11 = dw.sprite_half[1].ld();
    let t12 = t11 * ld_cell(cosine(engine_tables, angle));
    let t13 = t10 * ld_cell(sine(engine_tables, angle));
    let t14 = t12 + t13;
    dw.sprite_half[0].st(t14);
    let t15 = -t11;
    let t16 = t15 * ld_cell(sine(engine_tables, angle));
    let t17 = t10 * ld_cell(cosine(engine_tables, angle));
    let t18 = t16 + t17;
    dw.sprite_half[1].st(t18);
    let t19 = dw.sprite_half[2].ld();
    let t20 = dw.sprite_half[3].ld();
    let t21 = t20 * ld_cell(cosine(engine_tables, angle));
    let t22 = t19 * ld_cell(sine(engine_tables, angle));
    let t23 = t21 + t22;
    dw.sprite_half[2].st(t23);
    let t24 = -t20;
    let t25 = t24 * ld_cell(sine(engine_tables, angle));
    let t26 = t19 * ld_cell(cosine(engine_tables, angle));
    let t27 = t25 + t26;
    dw.sprite_half[3].st(t27);
    let t28 = t1 * dw.turned[0].ld();
    let t29 = t1 * dw.turned[1].ld();
    let t30 = -t29;
    let t31 = t30
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[1].get(),
        );
    let t32 = t31 - dw.sprite_half[1].ld();
    st_cell(&buffer.vertices[0].y, t32);
    let t33 = t31 - dw.sprite_half[3].ld();
    st_cell(&buffer.vertices[1].y, t33);
    let t34 = t31 + dw.sprite_half[1].ld();
    st_cell(&buffer.vertices[2].y, t34);
    let t35 = t31 + dw.sprite_half[3].ld();
    st_cell(&buffer.vertices[3].y, t35);
    let t36 = t28
        + crate::math::x87_float::ld(
            crate::scene::polygon_work::front_end().window.view_centre[0].get(),
        );
    let t37 = t36 - dw.sprite_half[0].ld();
    st_cell(&buffer.vertices[0].x, t37);
    let t38 = t36 - dw.sprite_half[2].ld();
    st_cell(&buffer.vertices[1].x, t38);
    let t39 = t36 + dw.sprite_half[0].ld();
    st_cell(&buffer.vertices[2].x, t39);
    let t40 = t36 + dw.sprite_half[2].ld();
    st_cell(&buffer.vertices[3].x, t40);
    t9
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::{Op, Spec, recorded_writes, with_copy};
    use crate::effects::blocks::CornerSlopes;
    use crate::effects::frame_state::DrawWork;
    use crate::effects::ring_store::RingWork;
    use crate::scene::engine_tables_store::{cosine, sine};

    /// The vertex float at byte `off` of the vertices (x, y or z of a vertex).
    fn vertex_cell(off: usize) -> &'static crate::session::Float {
        let v = &crate::d3d::vertices_store::VERTICES.vertices[off / 32];
        match off % 32 {
            0x0 => &v.x,
            0x4 => &v.y,
            0x8 => &v.z,
            _ => panic!("not a tested vertex field"),
        }
    }

    /// Runs `f` with each cell holding the test buffer's dword at its address; what `f`
    /// leaves goes back to the buffer, the cells as they were.
    fn staged<R>(
        cells: &[(
            &dyn crate::session::Cell32,
            crate::effects::asm_test::Operand,
        )],
        f: impl FnOnce() -> R,
    ) -> R {
        let host = |a: crate::effects::asm_test::Operand| a.host().cast::<u32>();
        let saved: Vec<u32> = cells.iter().map(|(c, _)| c.bits()).collect();
        for &(c, a) in cells {
            c.set_bits(unsafe { host(a).read_unaligned() });
        }
        let out = f();
        for (&(c, a), &old) in cells.iter().zip(&saved) {
            unsafe { host(a).write_unaligned(c.bits()) };
            c.set_bits(old);
        }
        out
    }

    /// The three ten-byte values at `at` of the test buffer.
    fn values(at: crate::effects::asm_test::Operand) -> [[u8; 10]; 3] {
        std::array::from_fn(|n| std::array::from_fn(|i| unsafe { (at + 10 * n + i).host().read() }))
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
        recorded_writes("effects.draw_cell_sprite.turn_x87", &SPEC, |b| {
            with_copy(b[1], RingWork::START, |w| {
                with_copy::<DrawWork, _>(b[1], DrawWork::START, |dw| {
                    super::turn_x87(
                        &crate::race::owner::race().frame.matrices.view_matrix,
                        &values(b[2]),
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
                    hi: 0xC8544,
                    reads: &[
                        (0xC3544, F32),
                        (0xC3548, F32),
                        (0xC35A8, F32),
                        (0xC4544, F32),
                        (0xC4548, F32),
                        (0xC45A8, F32),
                        (0xC7540, F32),
                        (0xC8540, F32),
                    ],
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
                    hi: 0x1BDBF4,
                    reads: &[
                        (0x18697D, F32),
                        (0x18697D, I32),
                        (0x18697D, U32),
                        (0x186981, F32),
                        (0x186981, U32),
                        (0x186985, F32),
                        (0x186989, F32),
                        (0x186995, F32),
                        (0x186999, F32),
                        (0x18699D, F32),
                        (0x186A2D, F32),
                        (0x1BB7F4, F32),
                        (0x1BB804, F32),
                        (0x1BB824, F32),
                        (0x1BBBE4, F32),
                        (0x1BBBF4, F32),
                    ],
                    writes: &[
                        (0x18697D, F32),
                        (0x186981, F32),
                        (0x186985, F32),
                        (0x186985, U32),
                        (0x186989, F32),
                        (0x186989, U32),
                    ],
                },
                Op::Index(&[0, 16, 48, 1008, 1024]),
                Op::Index(&[0, 4, 100, 16380]),
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_cell_sprite.square_x87", &SPEC, |b| {
            with_copy::<DrawWork, _>(b[2], DrawWork::START, |dw| {
                with_copy::<CornerSlopes, _>(b[2], CornerSlopes::START, |cs| {
                    let vertices: Vec<(
                        &dyn crate::session::Cell32,
                        crate::effects::asm_test::Operand,
                    )> = [
                        0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                    ]
                    .into_iter()
                    .map(|o| {
                        (
                            vertex_cell(o) as &dyn crate::session::Cell32,
                            b[1].wrapping_add(o),
                        )
                    })
                    .collect();
                    let tables = b[1].wrapping_add(0xC3544);
                    let mut cells = vertices;
                    cells.push((
                        sine(
                            &crate::race::owner::race().shared.buffers.engine_tables,
                            b[4].number() / 4,
                        ),
                        tables.wrapping_add(b[4].number()),
                    ));
                    cells.push((
                        cosine(
                            &crate::race::owner::race().shared.buffers.engine_tables,
                            b[4].number() / 4,
                        ),
                        tables.wrapping_add(b[4].number() + 0x1000),
                    ));
                    staged(&cells, || {
                        super::square_x87(
                            &crate::race::owner::race().shared.buffers.engine_tables,
                            &crate::d3d::vertices_store::VERTICES,
                            b[3].number(),
                            b[4].number() / 4,
                            dw,
                            cs,
                        )
                    })
                })
            })
        });
    }
}
