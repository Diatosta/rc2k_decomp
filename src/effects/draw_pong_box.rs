//! `effects::draw_pong_box`: one box of the pong game (ball, bat or a score cell) drawn on the
//! squares' frame: its height along the second axis times the flakes' second length into the sides
//! (and times a constant into the second pair), its width along the first times the first length
//! (likewise); the texture places from the pong's constants; its place (along, across) on the frame
//! into the corners; unless the first corner's depth is below the nearest depth: projected, depth
//! times the depth scale, colour -1, `scene::draw_fan`.
//!
//! The x87 runs are plain Rust on the x87 model (`math::x87_float`).

use crate::effects::cell_x87::CellX87;
use crate::effects::pong::{Pong, PongState};
use crate::effects::tuning::MarksFloats;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::math::x87;

/// The sides, from the four values; the place's two values (left untouched) popped after.
fn sides(
    pong: &Pong,
    fl: &WindscreenFlakes,
    v: &[[u8; 10]; 4],
    effects: &crate::effects::owner::Effects,
) {
    sides_x87(v, &pong.state, &effects.setup.floats, &fl.tables);
}

/// The corners, from the place.
fn corners(pong: &Pong, fl: &WindscreenFlakes, v: &[[u8; 10]; 4]) {
    corners_x87(v, &pong.state, &fl.tables);
}

/// The corners projected, and their depth.
fn projected(
    front: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    fl: &WindscreenFlakes,
) {
    projected_x87(front, buffer, &fl.tables);
}

pub fn draw_pong_box(
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    pong: &Pong,
    fl: &WindscreenFlakes,
    v: [[u8; 10]; 4],
    scene: &crate::scene::owner::Scene,
    effects: &crate::effects::owner::Effects,
) {
    sides(pong, fl, &v, effects);
    let [pa, pb, pc, pd] = pong
        .rules
        .box_places
        .each_ref()
        .map(crate::session::PlainFloat::get);
    for (n, (u, w)) in [(pa, pb), (pc, pb), (pc, pd), (pa, pd)]
        .into_iter()
        .enumerate()
    {
        buffer.vertices[n].u.set(u);
        buffer.vertices[n].v.set(w);
    }
    corners(pong, fl, &v);
    if x87::compare_status(&fl.tables.corners[0][2], &pong.rules.near) == 0x100 {
        return;
    }
    projected(scene.scratch.front_end_polygon(), buffer, fl);
    for item in buffer.vertices.iter().take(4) {
        item.colour.set(0xFFFF_FFFFu32);
    }
    crate::effects::clip_quad::draw_fan(buffer, scene);
}

use crate::effects::plain::st_cell;
use crate::effects::windscreen_flakes_store::WindscreenFlakeTables;
use crate::math::x87_float as bh;

/// `sides`'s arithmetic as plain Rust on the x87 model, in the same order.
fn sides_x87(v: &[[u8; 10]; 4], p: &PongState, f: &MarksFloats, sq: &WindscreenFlakeTables) {
    let _t0 = bh::from_f80(v[3]);
    let _t1 = bh::from_f80(v[2]);
    let t2 = bh::from_f80(v[1]);
    let t3 = bh::from_f80(v[0]);
    let t4 = sq.axes[1][0].ld();
    let t5 = t4 * t3;
    let t6 = t5 * sq.lengths[1].ld();
    p.box_sides[3].st(t6);
    let t7 = t6 * f.half.ld();
    p.box_sides[9].st(t7);
    let t8 = sq.axes[1][1].ld();
    let t9 = t8 * t3;
    let t10 = t9 * sq.lengths[1].ld();
    p.box_sides[4].st(t10);
    let t11 = t10 * f.half.ld();
    p.box_sides[10].st(t11);
    let t12 = sq.axes[1][2].ld();
    let t13 = t3 * t12;
    let t14 = t13 * sq.lengths[1].ld();
    p.box_sides[5].st(t14);
    let t15 = t14 * f.half.ld();
    p.box_sides[11].st(t15);
    let t16 = sq.axes[0][0].ld();
    let t17 = t16 * t2;
    let t18 = t17 * sq.lengths[0].ld();
    p.box_sides[0].st(t18);
    let t19 = t18 * f.half.ld();
    p.box_sides[6].st(t19);
    let t20 = sq.axes[0][1].ld();
    let t21 = t20 * t2;
    let t22 = t21 * sq.lengths[0].ld();
    p.box_sides[1].st(t22);
    let t23 = t22 * f.half.ld();
    p.box_sides[7].st(t23);
    let t24 = sq.axes[0][2].ld();
    let t25 = t2 * t24;
    let t26 = t25 * sq.lengths[0].ld();
    p.box_sides[2].st(t26);
    let t27 = t26 * f.half.ld();
    p.box_sides[8].st(t27);
}

/// `corners`'s arithmetic as plain Rust on the x87 model, in the same order.
fn corners_x87(v: &[[u8; 10]; 4], p: &PongState, sq: &WindscreenFlakeTables) {
    let t0 = bh::from_f80(v[3]);
    let t1 = bh::from_f80(v[2]);
    let t2 = t1 * sq.lengths[1].ld();
    let t3 = t0 * sq.lengths[0].ld();
    let t4 = t3 * sq.axes[0][0].ld();
    let t5 = t2 * sq.axes[1][0].ld();
    let t6 = t4 + t5;
    let t7 = t6 + sq.place[0].ld();
    sq.corners[0][0].st(t7);
    let t8 = t3 * sq.axes[0][1].ld();
    let t9 = t2 * sq.axes[1][1].ld();
    let t10 = t8 + t9;
    let t11 = t10 + sq.place[1].ld();
    sq.corners[0][1].st(t11);
    let t12 = t2 * sq.axes[1][2].ld();
    let t13 = t3 * sq.axes[0][2].ld();
    let t14 = t12 + t13;
    let t15 = t14 + sq.place[2].ld();
    sq.corners[0][2].st(t15);
    let t16 = sq.corners[0][0].ld();
    sq.corners[0][0].st(t16);
    let t17 = t16 + p.box_sides[3].ld();
    sq.corners[1][0].st(t17);
    let t18 = t17 + p.box_sides[0].ld();
    sq.corners[2][0].st(t18);
    let t19 = t18 - p.box_sides[3].ld();
    sq.corners[3][0].st(t19);
    let t20 = sq.corners[0][1].ld();
    sq.corners[0][1].st(t20);
    let t21 = t20 + p.box_sides[4].ld();
    sq.corners[1][1].st(t21);
    let t22 = t21 + p.box_sides[1].ld();
    sq.corners[2][1].st(t22);
    let t23 = t22 - p.box_sides[4].ld();
    sq.corners[3][1].st(t23);
    let t24 = sq.corners[0][2].ld();
    sq.corners[0][2].st(t24);
    let t25 = t24 + p.box_sides[5].ld();
    sq.corners[1][2].st(t25);
    let t26 = t25 + p.box_sides[2].ld();
    sq.corners[2][2].st(t26);
    let t27 = t26 - p.box_sides[5].ld();
    sq.corners[3][2].st(t27);
}

/// `projected`'s arithmetic as plain Rust on the x87 model, in the same order.
fn projected_x87(
    front: &crate::scene::polygon_work::PolygonWork,
    buffer: &crate::d3d::vertices_store::PolygonVertices,
    sq: &WindscreenFlakeTables,
) {
    let t0 = bh::ld(front.window.view_scale.get());
    let t1 = t0 / sq.corners[3][2].ld();
    let t2 = bh::ld(front.window.view_scale.get());
    let t3 = t2 / sq.corners[2][2].ld();
    let t4 = bh::ld(front.window.view_scale.get());
    let t5 = t4 / sq.corners[1][2].ld();
    let t6 = bh::ld(front.window.view_scale.get());
    let t7 = t6 / sq.corners[0][2].ld();
    let t8 = sq.corners[0][1].ld();
    let t9 = t8 * t7;
    let t10 = t7 * sq.corners[0][0].ld();
    let t11 = -t9;
    let t12 = t11 + front.window.view_centre[1].ld();
    st_cell(&buffer.vertices[0].y, t12);
    let t13 = t10 + front.window.view_centre[0].ld();
    st_cell(&buffer.vertices[0].x, t13);
    let t14 = sq.corners[1][1].ld();
    let t15 = t14 * t5;
    let t16 = t5 * sq.corners[1][0].ld();
    let t17 = -t15;
    let t18 = t17 + front.window.view_centre[1].ld();
    st_cell(&buffer.vertices[1].y, t18);
    let t19 = t16 + front.window.view_centre[0].ld();
    st_cell(&buffer.vertices[1].x, t19);
    let t20 = sq.corners[2][1].ld();
    let t21 = t20 * t3;
    let t22 = t3 * sq.corners[2][0].ld();
    let t23 = -t21;
    let t24 = t23 + front.window.view_centre[1].ld();
    st_cell(&buffer.vertices[2].y, t24);
    let t25 = t22 + front.window.view_centre[0].ld();
    st_cell(&buffer.vertices[2].x, t25);
    let t26 = sq.corners[3][1].ld();
    let t27 = t26 * t1;
    let t28 = t1 * sq.corners[3][0].ld();
    let t29 = -t27;
    let t30 = t29 + front.window.view_centre[1].ld();
    st_cell(&buffer.vertices[3].y, t30);
    let t31 = t28 + front.window.view_centre[0].ld();
    st_cell(&buffer.vertices[3].x, t31);
    let t32 = sq.corners[0][2].ld();
    let t33 = t32 * sq.inverse.ld();
    let t34 = sq.corners[1][2].ld();
    let t35 = t34 * sq.inverse.ld();
    let t36 = sq.corners[2][2].ld();
    let t37 = t36 * sq.inverse.ld();
    let t38 = sq.corners[3][2].ld();
    let t39 = t38 * sq.inverse.ld();
    st_cell(&buffer.vertices[3].z, t39);
    st_cell(&buffer.vertices[2].z, t37);
    st_cell(&buffer.vertices[1].z, t35);
    st_cell(&buffer.vertices[0].z, t33);
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_layout::with_laid;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::staged_vertices as staged;

    use crate::effects::asm_test::{Op, Spec, recorded_writes};
    use crate::effects::pong::PongState;
    use crate::effects::tuning::MarksFloats;
    use crate::effects::windscreen_flakes_store::WindscreenFlakeTables;

    /// The four ten-byte values at `at` of the test buffer (0 where the buffer does not
    /// reach: a block reads only the last two).
    fn values(at: crate::effects::asm_test::Operand) -> [[u8; 10]; 4] {
        std::array::from_fn(|n| at.bytes_or_zero(10 * n, 10).try_into().unwrap())
    }

    #[test]
    fn sides_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x200,
                    hi: 0x13822A,
                    reads: &[
                        (0x218, F32),
                        (0x137FAE, F32),
                        (0x137FB2, F32),
                        (0x137FDA, F32),
                        (0x137FDE, F32),
                        (0x137FE2, F32),
                        (0x137FE6, F32),
                        (0x137FEA, F32),
                        (0x137FEE, F32),
                    ],
                    writes: &[
                        (0x3718, F32),
                        (0x371C, F32),
                        (0x3720, F32),
                        (0x3724, F32),
                        (0x3728, F32),
                        (0x372C, F32),
                        (0x3730, F32),
                        (0x3734, F32),
                        (0x3738, F32),
                        (0x373C, F32),
                        (0x3740, F32),
                        (0x3744, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x28,
                    reads: &[(0x0, F80), (0xA, F80), (0x14, F80), (0x1E, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_pong_box.sides_x87", &SPEC, |b| {
            with_laid::<MarksFloats, _>(b[0], |f| {
                with_laid::<WindscreenFlakeTables, _>(b[0], |sq| {
                    with_laid::<PongState, _>(b[0], |state| {
                        super::sides_x87(&values(b[1]), state, f, sq);
                    });
                });
            });
        });
    }

    #[test]
    fn corners_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x3718,
                    hi: 0x13822A,
                    reads: &[
                        (0x3718, F32),
                        (0x371C, F32),
                        (0x3720, F32),
                        (0x3724, F32),
                        (0x3728, F32),
                        (0x372C, F32),
                        (0x137FAE, F32),
                        (0x137FB2, F32),
                        (0x137FB6, F32),
                        (0x137FBA, F32),
                        (0x137FBE, F32),
                        (0x137FDA, F32),
                        (0x137FDE, F32),
                        (0x137FE2, F32),
                        (0x137FE6, F32),
                        (0x137FEA, F32),
                        (0x137FEE, F32),
                        (0x138052, F32),
                        (0x138056, F32),
                        (0x13805A, F32),
                    ],
                    writes: &[
                        (0x138052, F32),
                        (0x138056, F32),
                        (0x13805A, F32),
                        (0x13805E, F32),
                        (0x138062, F32),
                        (0x138066, F32),
                        (0x13806A, F32),
                        (0x13806E, F32),
                        (0x138072, F32),
                        (0x138076, F32),
                        (0x13807A, F32),
                        (0x13807E, F32),
                    ],
                },
                Op::Base {
                    lo: 0x14,
                    hi: 0x28,
                    reads: &[(0x14, F80), (0x1E, F80)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_pong_box.corners_x87", &SPEC, |b| {
            with_laid::<WindscreenFlakeTables, _>(b[0], |sq| {
                with_laid::<PongState, _>(b[0], |state| {
                    super::corners_x87(&values(b[1]), state, sq);
                });
            });
        });
    }

    #[test]
    fn projected_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::RaceBlock {
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x137B8A,
                    hi: 0x13822A,
                    reads: &[
                        (0x137FAA, F32),
                        (0x138052, F32),
                        (0x138056, F32),
                        (0x13805A, F32),
                        (0x13805E, F32),
                        (0x138062, F32),
                        (0x138066, F32),
                        (0x13806A, F32),
                        (0x13806E, F32),
                        (0x138072, F32),
                        (0x138076, F32),
                        (0x13807A, F32),
                        (0x13807E, F32),
                    ],
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
            ],
            sincos: false,
        };
        recorded_writes("effects.draw_pong_box.projected_x87", &SPEC, |b| {
            with_laid::<WindscreenFlakeTables, _>(b[1], |sq| {
                staged(
                    b[2],
                    &[
                        0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                    ],
                    || {
                        super::projected_x87(
                            crate::owners::Owners::the()
                                .scene
                                .scratch
                                .front_end_polygon(),
                            &crate::d3d::vertices_store::VERTICES,
                            sq,
                        );
                    },
                );
            });
        });
    }
}
