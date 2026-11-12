//! `physics::contact_force` (0x45F17E): the force of one contact into the record that
//! `contact_record` names. From the contact's point (`contact_point`, car frame), the car's speed
//! (`contact_car_speed`), its turn speed (`contact_turn_speed`) and the surface's normal
//! (`contact_normal`): the point's speed (turn x point + speed) goes into `contact_point_speed`; its
//! depth along the normal, held at least 1 - |normal y| + `hundredth_a`, into `contact_depth`; the push
//! the turn would give (point x normal turned by the inverse matrix, times `inertia`, turned back,
//! x point) along the normal goes into `contact_turn_push` and the share `contact_share_numerator` /
//! (it + 1) into `contact_share`; the normal push (`contact_normal_push`), the speed along the normal
//! and across it (`contact_normal_speed`, `contact_side_speed`; their lengths `contact_speed_length`,
//! `contact_value`), with `contact_step` a part of the normal x `contact_normal_factor` x it /
//! `contact_steps` added; the slide against the motion (`contact_slide`) held to
//! `contact_slide_limit` long; their sum (`contact_total`) turned by the inverse matrix and divided
//! by `step_time` goes into the record's force, the point turned into its point (+0xC ..). The turns
//! are `physics::turn_by_inverse` / `turn_by_car_matrix`, the products `scene::cross_product`,
//! `physics::dot_product`, `multiply_parts`, `scale_vector`, `vector_length`.
//!
//! The flow and the calls are Rust; each straight x87 run is plain Rust over `X`
//! (`math/x87_float.rs`) in the `x87_N` functions below, the original's instructions
//! being recorded in the tests' `reference_vectors`.
//!
//! Takes: the record the force goes to (`contact_record`, set by each caller just before)
//!
//! Callers: `physics::wall_push_back`, `physics::wall_edge_contact`,
//! `physics::crossed_edge_contact`, `physics::face_contact`,
//! `physics::crash_contact`, `physics::ground_contact`.

use crate::effects::cell_x87::CellX87;
use crate::math::x87::{compare_80, neg_rounded};
use crate::math::x87_float::compare;
use crate::math::x87_float::int;
use crate::math::x87_float::{X, f80};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::{ZERO_BITS as ZERO, hold_low, negate, p};
use crate::physics::multiply_parts::multiply_parts;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::ForceRecord;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::physics::vector_length::vector_length;
use crate::race::numbers_store::Numbers;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::view_matrix::cross_product;
use crate::session::Float;

/// Three floats copied as bits, in order.
fn copy3(to: &[Float], from: &[Float]) {
    for k in 0..3 {
        to[k].set(from[k].get());
    }
}

/// Three floats set to the bits `v`.
fn fill3(to: &[Float], v: u32) {
    for cell in &to[..3] {
        cell.set_bits(v);
    }
}

/// `record` the one the contact's force goes to (`contact_record`, set by each caller
/// just before).
pub fn contact_force(
    contact: &ContactWork,
    numbers: &Numbers,
    physics_tables: &PhysicsTables,
    car: &Car,
    record: &ForceRecord,
    scene: &crate::scene::owner::Scene,
) {
    let drawing = &scene.drawing;
    cross_product(
        &contact.contact_turn_speed,
        &contact.contact_point,
        &contact.contact_turn_cross,
    );
    x87_1(
        &contact.contact_turn_cross,
        &contact.contact_car_speed,
        &contact.contact_point_speed,
    );
    p(
        &contact.contact_depth,
        neg_rounded(dot_product(
            &contact.contact_point_speed,
            &contact.contact_normal,
        ))
        .to_bits(),
    );
    x87_2(contact, numbers, scene);
    hold_low(&contact.contact_depth, &drawing.scene.scratch);
    // 0x45F222: the push the turn would give.
    cross_product(
        &contact.contact_point,
        &contact.contact_normal,
        &contact.contact_scratch_a,
    );
    turn_by_inverse(car, &contact.contact_scratch_a);
    multiply_parts(
        &contact.contact_scratch_a,
        &physics_tables.inertia,
        &contact.contact_scratch_b,
    );
    turn_by_car_matrix(car, &contact.contact_scratch_b);
    cross_product(
        &contact.contact_point,
        &contact.contact_scratch_b,
        &contact.contact_scratch_a,
    );
    p(
        &contact.contact_turn_push,
        neg_rounded(dot_product(
            &contact.contact_scratch_a,
            &contact.contact_normal,
        ))
        .to_bits(),
    );
    x87_3(contact, numbers);
    // 0x45F2C2: the normal push, the speed along the normal and across it.
    copy3(&contact.contact_normal_push, &contact.contact_normal);
    let factor = extended(contact.contact_factor.get());
    scale_vector(&contact.contact_normal_push, factor);
    drawing.scene.scratch.set(rounded(factor));
    let along = { dot_product(&contact.contact_point_speed, &contact.contact_normal) };
    copy3(&contact.contact_normal_speed, &contact.contact_normal);
    scale_vector(&contact.contact_normal_speed, along);
    drawing.scene.scratch.set(rounded(along));
    copy3(&contact.contact_side_speed, &contact.contact_point_speed);
    x87_4(contact);
    contact
        .contact_speed_length
        .set_bits(rounded(vector_length(&contact.contact_normal_speed)));
    contact
        .contact_value
        .set_bits(rounded(vector_length(&contact.contact_side_speed)));
    fill3(&contact.contact_extra, 0);
    if physics_tables.contact_step.get() != 0 {
        fill3(&contact.contact_normal_push, 0);
        copy3(&contact.contact_extra, &contact.contact_normal);
        let part = x87_5(contact, physics_tables);
        scale_vector(&contact.contact_extra, part);
        drawing.scene.scratch.set(rounded(part));
    }
    x87_1(
        &contact.contact_normal_push,
        &contact.contact_extra,
        &contact.contact_normal_push,
    );
    copy3(&contact.contact_total, &contact.contact_normal_push);
    contact
        .contact_slide_limit_held
        .set(crate::math::x87::loaded(&contact.contact_slide_limit));
    // 0x45F48D: the slide against the motion, held to `contact_slide_limit_held` long.
    copy3(&contact.contact_slide, &contact.contact_side_speed);
    negate(&contact.contact_slide[0]);
    negate(&contact.contact_slide[1]);
    negate(&contact.contact_slide[2]);
    if compare_80(vector_length(&contact.contact_slide), ZERO) & 0x4000 != 0 {
        fill3(&contact.contact_slide, 0);
    }
    let status = x87_6(contact, scene);
    if status & 0x4100 == 0 {
        x87_7(contact, scene);
    }
    // 0x45F57E: the sum into the record.
    x87_1(
        &contact.contact_slide,
        &contact.contact_normal_push,
        &contact.contact_total,
    );
    turn_by_inverse(car, &contact.contact_total);
    x87_8(contact, physics_tables, &record.force);
    turn_by_inverse(car, &contact.contact_point);
    for k in 0..3 {
        record.point[k].set(contact.contact_point[k].get());
    }
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`: three floats added, `out[k] = a[k] +
/// b[k]` (each stored as made, so `out` may be one of the two).
fn x87_1(a: &[Float; 3], b: &[Float; 3], out: &[Float; 3]) {
    for k in 0..3 {
        out[k].st(a[k].ld() + b[k].ld());
    }
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = (1 - |[rc + 0x2D307C]|) + [hundredth_a]`
fn x87_2(contact: &ContactWork, numbers: &Numbers, scene: &crate::scene::owner::Scene) {
    let t0 = X(1.0);
    let t1 = contact.contact_normal[1].ld();
    let t2 = t1.abs();
    let t3 = t0 - t2;
    let t4 = numbers.hundredth_a.ld();
    let t5 = t3 + t4;
    scene.drawing.scene.scratch.st(t5);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[contact_share] = [contact_share_numerator] / ([contact_turn_push] + [one])`
/// `[contact_factor] = [contact_depth] * [contact_share]`
fn x87_3(contact: &ContactWork, numbers: &Numbers) {
    let t0 = contact.contact_share_numerator.ld();
    let t1 = contact.contact_turn_push.ld();
    let t2 = numbers.one.ld();
    let t3 = t1 + t2;
    let t4 = t0 / t3;
    contact.contact_share.st(t4);
    let t5 = contact.contact_depth.ld();
    let t6 = contact.contact_share.ld();
    let t7 = t5 * t6;
    contact.contact_factor.st(t7);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[contact_side_speed] = [contact_side_speed] - [contact_normal_speed]`
/// `[rc + 0x2D30F4] = [rc + 0x2D30F4] - [rc + 0x2D30E8]`
/// `[rc + 0x2D30F8] = [rc + 0x2D30F8] - [rc + 0x2D30EC]`
fn x87_4(contact: &ContactWork) {
    let t0 = contact.contact_side_speed[0].ld();
    let t1 = contact.contact_normal_speed[0].ld();
    let t2 = t0 - t1;
    contact.contact_side_speed[0].st(t2);
    let t3 = contact.contact_side_speed[1].ld();
    let t4 = contact.contact_normal_speed[1].ld();
    let t5 = t3 - t4;
    contact.contact_side_speed[1].st(t5);
    let t6 = contact.contact_side_speed[2].ld();
    let t7 = contact.contact_normal_speed[2].ld();
    let t8 = t6 - t7;
    contact.contact_side_speed[2].st(t8);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `ext [o + 0] = ([contact_normal_factor] * int [contact_step]) / int [contact_steps]`
fn x87_5(contact: &ContactWork, tables: &PhysicsTables) -> [u8; 10] {
    let t0 = contact.contact_normal_factor.ld();
    let t1 = int(tables.contact_step.get() as i32);
    let t2 = t0 * t1;
    let t3 = int(tables.contact_steps.get() as i32);
    let t4 = t2 / t3;
    f80(t4)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[scratch] = sqrt((([contact_slide] * [contact_slide]) + (([rc + 0x2D3144] * [rc + 0x2D3144]) + ([rc + 0x2D3148] * [rc + 0x2D3148]))))`
/// `status [s + 0] = status`
fn x87_6(contact: &ContactWork, scene: &crate::scene::owner::Scene) -> u16 {
    let t0 = contact.contact_slide[0].ld();
    let t1 = t0 * t0;
    let t2 = contact.contact_slide[1].ld();
    let t3 = t2 * t2;
    let t4 = contact.contact_slide[2].ld();
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    scene.drawing.scene.scratch.st(t8);
    let t9 = contact.contact_slide_limit_held.ld();
    compare(t8, t9)
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[contact_slide] = [contact_slide] * ([contact_slide_limit_held] / [scratch])`
/// `[rc + 0x2D3144] = [rc + 0x2D3144] * ([contact_slide_limit_held] / [scratch])`
/// `[rc + 0x2D3148] = [rc + 0x2D3148] * ([contact_slide_limit_held] / [scratch])`
/// `[scratch] = [contact_slide_limit_held] / [scratch]`
fn x87_7(contact: &ContactWork, scene: &crate::scene::owner::Scene) {
    let drawing = &scene.drawing;
    let t0 = contact.contact_slide_limit_held.ld();
    let t1 = drawing.scene.scratch.ld();
    let t2 = t0 / t1;
    let t3 = contact.contact_slide[0].ld();
    let t4 = t3 * t2;
    contact.contact_slide[0].st(t4);
    let t5 = contact.contact_slide[1].ld();
    let t6 = t5 * t2;
    contact.contact_slide[1].st(t6);
    let t7 = contact.contact_slide[2].ld();
    let t8 = t7 * t2;
    contact.contact_slide[2].st(t8);
    drawing.scene.scratch.st(t2);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[d + 0] = [contact_total] / [step_time]`
/// `[d + 4] = [rc + 0x2D3154] / [step_time]`
/// `[d + 8] = [rc + 0x2D3158] / [step_time]`
fn x87_8(contact: &ContactWork, tables: &PhysicsTables, d: &[Float; 3]) {
    let t0 = contact.contact_total[0].ld();
    let t1 = tables.step_time.ld();
    let t2 = t0 / t1;
    d[0].st(t2);
    let t3 = contact.contact_total[1].ld();
    let t4 = tables.step_time.ld();
    let t5 = t3 / t4;
    d[1].st(t5);
    let t6 = contact.contact_total[2].ld();
    let t7 = tables.step_time.ld();
    let t8 = t6 / t7;
    d[2].st(t8);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::x87_mem::{Base, view};
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "contact_force line 50",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x2D30A8, In::F),
                        (0x2D30AC, In::F),
                        (0x2D30B0, In::F),
                        (0x2D30C0, In::F),
                        (0x2D30C4, In::F),
                        (0x2D30C8, In::F),
                    ],
                    writes: &[(0x2D30CC, Out::F), (0x2D30D0, Out::F), (0x2D30D4, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_1(
                        &crate::race::owner::race()
                            .physics
                            .contact
                            .contact_turn_cross,
                        &crate::race::owner::race().physics.contact.contact_car_speed,
                        &crate::race::owner::race()
                            .physics
                            .contact
                            .contact_point_speed,
                    );
                },
            },
            Block {
                name: "contact_force line 66",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x1C5CFC, In::F), (0x2D307C, In::F)],
                    writes: &[(0x22400C, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_2(
                        &crate::race::owner::race().physics.contact,
                        &crate::race::owner::race().course.numbers,
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "contact_force line 86",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D38, In::F),
                        (0x2D1DF4, In::F),
                        (0x2D31D8, In::F),
                        (0x2D31DC, In::F),
                        (0x2D31E0, In::F),
                    ],
                    writes: &[(0x2D31E0, Out::F), (0x2D3204, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_3(
                        &crate::race::owner::race().physics.contact,
                        &crate::race::owner::race().course.numbers,
                    );
                },
            },
            Block {
                name: "contact_force line 107",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x2D30E4, In::F),
                        (0x2D30E8, In::F),
                        (0x2D30EC, In::F),
                        (0x2D30F0, In::F),
                        (0x2D30F4, In::F),
                        (0x2D30F8, In::F),
                    ],
                    writes: &[(0x2D30F0, Out::F), (0x2D30F4, Out::F), (0x2D30F8, Out::F)],
                }],
                ulps: 0,
                plain: |_| x87_4(&crate::race::owner::race().physics.contact),
            },
            Block {
                name: "contact_force line 126",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x214148, In::I), (0x21414C, In::I), (0x2D1DF8, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::T)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    a[1].set_tbyte(
                        0,
                        x87_5(&race().physics.contact, &race().shared.physics_tables),
                    );
                },
            },
            Block {
                name: "contact_force line 137",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x2D3130, In::F),
                        (0x2D3134, In::F),
                        (0x2D3138, In::F),
                        (0x2D3160, In::F),
                        (0x2D3164, In::F),
                        (0x2D3168, In::F),
                    ],
                    writes: &[(0x2D3130, Out::F), (0x2D3134, Out::F), (0x2D3138, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_1(
                        &crate::race::owner::race()
                            .physics
                            .contact
                            .contact_normal_push,
                        &crate::race::owner::race().physics.contact.contact_extra,
                        &crate::race::owner::race()
                            .physics
                            .contact
                            .contact_normal_push,
                    );
                },
            },
            Block {
                name: "contact_force line 160",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x2D30FC, In::F),
                            (0x2D3140, In::F),
                            (0x2D3144, In::F),
                            (0x2D3148, In::F),
                        ],
                        writes: &[(0x22400C, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::H)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    a[1].set_word(
                        0,
                        x87_6(
                            &crate::race::owner::race().physics.contact,
                            crate::scene::owner::scene(),
                        ),
                    );
                },
            },
            Block {
                name: "contact_force line 176",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x22400C, In::F),
                        (0x2D30FC, In::F),
                        (0x2D3140, In::F),
                        (0x2D3144, In::F),
                        (0x2D3148, In::F),
                    ],
                    writes: &[
                        (0x22400C, Out::F),
                        (0x2D3140, Out::F),
                        (0x2D3144, Out::F),
                        (0x2D3148, Out::F),
                    ],
                }],
                ulps: 0,
                plain: |_| {
                    x87_7(
                        &crate::race::owner::race().physics.contact,
                        crate::scene::owner::scene(),
                    );
                },
            },
            Block {
                name: "contact_force line 193",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x2D3130, In::F),
                        (0x2D3134, In::F),
                        (0x2D3138, In::F),
                        (0x2D3140, In::F),
                        (0x2D3144, In::F),
                        (0x2D3148, In::F),
                    ],
                    writes: &[(0x2D3150, Out::F), (0x2D3154, Out::F), (0x2D3158, Out::F)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_1(
                        &crate::race::owner::race().physics.contact.contact_slide,
                        &crate::race::owner::race()
                            .physics
                            .contact
                            .contact_normal_push,
                        &crate::race::owner::race().physics.contact.contact_total,
                    );
                },
            },
            Block {
                name: "contact_force line 207",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x21388C, In::F),
                            (0x2D3150, In::F),
                            (0x2D3154, In::F),
                            (0x2D3158, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F), (4, Out::F), (8, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    x87_8(
                        &race().physics.contact,
                        &race().shared.physics_tables,
                        view(a[1], 0),
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.contact_force", &blocks(), 1000, 100);
    }
}
