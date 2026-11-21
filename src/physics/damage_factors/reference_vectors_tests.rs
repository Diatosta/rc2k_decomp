use super::*;
use crate::math::cell_x87::CellX87;
use crate::math::x87_float::compare;
use crate::physics::store::laid;
use crate::test_support::own::race;

/// The status word of the engine's speed against its limit times the factor.
fn over_limit_status(car: &Car, m: &DamageModel) -> u16 {
    compare(
        car.engine_speed_limit.ld() * crate::math::x87_float::ld(m.engine_limit_factor),
        car.engine_speed.ld(),
    )
}

/// The damage model as the test's buffer holds it.
fn dm(base: usize) -> &'static mut DamageModel {
    crate::physics::store::model_in_test_buffer(base)
}
use crate::math::x87_mem::Base;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse_at};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "damage_factors line 39",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x319FF8, In::I), (0x31A008, In::I), (0x31A00C, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B35C), In::F)],
                    writes: &[
                        (laid!(0x35B3F8), Out::F),
                        (laid!(0x35B3FC), Out::F),
                        (laid!(0x35B400), Out::F),
                    ],
                },
            ],
            ulps: 0,
            plain: |a| scale_car_figures(race().shared.car(), dm(a[1])),
        },
        Block {
            name: "damage_factors line 53",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B34C), In::F),
                        (laid!(0x35B3FC), In::F),
                        (laid!(0x35B474), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A420, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                wheel_stiffness_from_share(
                    dm(a[0]),
                    dm(a[0]).wheel_stiffness_lerp[0],
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_stiffness
                        .float_at(0),
                );
            },
        },
        Block {
            name: "damage_factors line 63",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B3F8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A410, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                wheel_ride_height_from_share(
                    dm(a[0]),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_ride_height
                        .float_at(0),
                );
            },
        },
        Block {
            name: "damage_factors line 71",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B350), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A410, Out::F), (0x31A420, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                fail_wheel(
                    dm(a[0]),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_stiffness
                        .float_at(0),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_ride_height
                        .float_at(0),
                );
            },
        },
        Block {
            name: "damage_factors line 81",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B34C), In::F),
                        (laid!(0x35B3FC), In::F),
                        (laid!(0x35B478), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A424, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                wheel_stiffness_from_share(
                    dm(a[0]),
                    dm(a[0]).wheel_stiffness_lerp[1],
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_stiffness
                        .float_at(1),
                );
            },
        },
        Block {
            name: "damage_factors line 91",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B3F8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A414, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                wheel_ride_height_from_share(
                    dm(a[0]),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_ride_height
                        .float_at(1),
                );
            },
        },
        Block {
            name: "damage_factors line 99",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B350), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A414, Out::F), (0x31A424, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                fail_wheel(
                    dm(a[0]),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_stiffness
                        .float_at(1),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_ride_height
                        .float_at(1),
                );
            },
        },
        Block {
            name: "damage_factors line 109",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B34C), In::F),
                        (laid!(0x35B3FC), In::F),
                        (laid!(0x35B47C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A428, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                wheel_stiffness_from_share(
                    dm(a[0]),
                    dm(a[0]).wheel_stiffness_lerp[2],
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_stiffness
                        .float_at(2),
                );
            },
        },
        Block {
            name: "damage_factors line 119",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B3F8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A418, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                wheel_ride_height_from_share(
                    dm(a[0]),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_ride_height
                        .float_at(2),
                );
            },
        },
        Block {
            name: "damage_factors line 127",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B350), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A418, Out::F), (0x31A428, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                fail_wheel(
                    dm(a[0]),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_stiffness
                        .float_at(2),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_ride_height
                        .float_at(2),
                );
            },
        },
        Block {
            name: "damage_factors line 137",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B34C), In::F),
                        (laid!(0x35B3FC), In::F),
                        (laid!(0x35B480), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A42C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                wheel_stiffness_from_share(
                    dm(a[0]),
                    dm(a[0]).wheel_stiffness_lerp[3],
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_stiffness
                        .float_at(3),
                );
            },
        },
        Block {
            name: "damage_factors line 147",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B3F8), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A41C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                wheel_ride_height_from_share(
                    dm(a[0]),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_ride_height
                        .float_at(3),
                );
            },
        },
        Block {
            name: "damage_factors line 155",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B350), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[],
                    writes: &[(0x31A41C, Out::F), (0x31A42C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                fail_wheel(
                    dm(a[0]),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_stiffness
                        .float_at(3),
                    &crate::test_support::own::race()
                        .shared
                        .car()
                        .wheel_ride_height
                        .float_at(3),
                );
            },
        },
        Block {
            name: "damage_factors line 164",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x31A018, In::F),
                        (0x31A1B0, In::F),
                        (0x31A410, In::F),
                        (0x31A414, In::F),
                        (0x31A418, In::F),
                        (0x31A41C, In::F),
                        (0x31A420, In::F),
                        (0x31A424, In::F),
                        (0x31A428, In::F),
                        (0x31A42C, In::F),
                    ],
                    writes: &[(0x31A430, Out::F), (0x31A434, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B364), In::F)],
                    writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3E4), Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| means_and_engine_speed(race().shared.car(), dm(a[1])),
        },
        Block {
            name: "damage_factors line 186",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B34C), In::F), (laid!(0x35B36C), In::F)],
                    writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B398), Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A184, In::F), (0x31A1B4, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| engine_speed_share_step(race().shared.car(), dm(a[0])),
        },
        Block {
            name: "damage_factors line 198",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B34C), In::F),
                        (laid!(0x35B36C), In::F),
                        (laid!(0x35B378), In::F),
                    ],
                    writes: &[
                        (laid!(0x35B36C), Out::F),
                        (laid!(0x35B39C), Out::F),
                        (laid!(0x35B3E0), Out::F),
                    ],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A184, In::F), (0x31A190, In::F), (0x31A930, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| speed_share_step(race().shared.car(), dm(a[0])),
        },
        Block {
            name: "damage_factors line 213",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B36C), In::F), (laid!(0x35B378), In::F)],
                    writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3A0), Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A184, In::F), (0x31A194, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| drive_a_slip_step(race().shared.car(), dm(a[0])),
        },
        Block {
            name: "damage_factors line 225",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B36C), In::F),
                        (laid!(0x35B378), In::F),
                        (laid!(0x35B6F4), In::F),
                    ],
                    writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3A4), Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A184, In::F), (0x31A190, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                dm(a[0]).drive_b_slip_share = drive_extrapolation_step(
                    race().shared.car(),
                    dm(a[0]),
                    &crate::test_support::own::race().shared.car().drive_speed_a,
                    &crate::test_support::own::race().shared.car().drive_speed_a,
                );
            },
        },
        Block {
            name: "damage_factors line 239",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B36C), In::F),
                        (laid!(0x35B378), In::F),
                        (laid!(0x35B6F4), In::F),
                    ],
                    writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3A8), Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A184, In::F), (0x31A194, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                dm(a[0]).drive_a_extrapolated_share = drive_extrapolation_step(
                    race().shared.car(),
                    dm(a[0]),
                    &crate::test_support::own::race().shared.car().drive_speed_b,
                    &crate::test_support::own::race().shared.car().drive_speed_b,
                );
            },
        },
        Block {
            name: "damage_factors line 253",
            operands: vec![Operand::Buffer {
                reads: &[(laid!(0x35B36C), In::F), (laid!(0x35B378), In::F)],
                writes: &[(laid!(0x35B3AC), Out::F), (laid!(0x35B3F4), Out::F)],
            }],
            ulps: 0,
            plain: |a| finish_shares(dm(a[0])),
        },
        Block {
            name: "damage_factors line 262",
            operands: vec![Operand::Buffer {
                reads: &[],
                writes: &[(laid!(0x35B3F4), Out::F)],
            }],
            ulps: 0,
            plain: |a| dm(a[0]).wheels_down = 1.0,
        },
        Block {
            name: "damage_factors line 268",
            operands: vec![Operand::Buffer {
                reads: &[],
                writes: &[(laid!(0x35B3B0), Out::F)],
            }],
            ulps: 0,
            plain: |a| dm(a[0]).brake_share = 0.0,
        },
        Block {
            name: "damage_factors line 276",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x31A614, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B35C), In::F)],
                    writes: &[(laid!(0x35B3B0), Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| brake_share_step(race().shared.car(), dm(a[1])),
        },
        Block {
            name: "damage_factors line 287",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x31A1B0, In::F), (0x31A1B4, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B37C), In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::H)],
                },
            ],
            ulps: 0,
            plain: |a| a[2].set_word(0, over_limit_status(race().shared.car(), dm(a[1]))),
        },
        Block {
            name: "damage_factors line 302",
            operands: vec![Operand::Buffer {
                reads: &[],
                writes: &[(laid!(0x35B3B4), Out::F)],
            }],
            ulps: 0,
            plain: |a| dm(a[0]).on_surface_5 = 0.0,
        },
        Block {
            name: "damage_factors line 322",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x31A180, In::F), (0x31A208, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B378), In::F),
                        (laid!(0x35B394), In::F),
                        (laid!(0x35B624), In::F),
                    ],
                    writes: &[(laid!(0x35B36C), Out::F), (laid!(0x35B3C8), Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| hit_size_and_dive(race().shared.car(), dm(a[1])),
        },
        Block {
            name: "damage_factors line 337",
            operands: vec![Operand::Buffer {
                reads: &[(laid!(0x35B36C), In::F), (laid!(0x35B620), In::F)],
                writes: &[(laid!(0x35B36C), Out::F)],
            }],
            ulps: 0,
            plain: |a| square_dive(dm(a[0])),
        },
        Block {
            name: "damage_factors line 350",
            operands: vec![Operand::Buffer {
                reads: &[
                    (laid!(0x35B360), In::F),
                    (laid!(0x35B364), In::F),
                    (laid!(0x35B36C), In::F),
                    (laid!(0x35B3CC), In::F),
                    (laid!(0x35B3D0), In::F),
                    (laid!(0x35B3D4), In::F),
                    (laid!(0x35B3D8), In::F),
                    (laid!(0x35B3DC), In::F),
                ],
                writes: &[
                    (laid!(0x35B36C), Out::F),
                    (laid!(0x35B3CC), Out::F),
                    (laid!(0x35B3D0), Out::F),
                    (laid!(0x35B3D4), Out::F),
                    (laid!(0x35B3D8), Out::F),
                    (laid!(0x35B3DC), Out::F),
                ],
            }],
            ulps: 0,
            plain: |a| blend_wheel_loads(dm(a[0])),
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    // The plain code rounds to nearest, the mode the damage model always runs in.
    sparse_at("physics.damage_factors", &blocks(), 1200, 100, &[0x027F]);
}
