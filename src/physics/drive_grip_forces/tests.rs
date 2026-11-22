use super::*;
use crate::math::x87_mem::{HostCell, view};
use crate::memory::session::Cell32;
use crate::memory::session::PlainFloat;
use crate::test_support::own::race;
use crate::test_support::own::scene;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "drive_grip_forces line 39 idx e=0",
            operands: vec![
                Operand::Buffers {
                    reads: &[(0x223AE4, In::F)],
                    writes: &[],
                },
                Operand::Value(0),
                Operand::RaceBlock {
                    reads: &[(0x214BA0, In::F), (0x31A184, In::F)],
                    writes: &[(0x31A1E8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                steer_sine_force(
                    &mut race().shared.car_mut(),
                    &mut race().shared.physics_tables_mut(),
                    race().engine_tables(),
                    a[1],
                );
            },
        },
        Block {
            name: "drive_grip_forces line 39 idx e=1",
            operands: vec![
                Operand::Buffers {
                    reads: &[(0x223AE8, In::F)],
                    writes: &[],
                },
                Operand::Value(1),
                Operand::RaceBlock {
                    reads: &[(0x214BA0, In::F), (0x31A184, In::F)],
                    writes: &[(0x31A1E8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                steer_sine_force(
                    &mut race().shared.car_mut(),
                    &mut race().shared.physics_tables_mut(),
                    race().engine_tables(),
                    a[1],
                );
            },
        },
        Block {
            name: "drive_grip_forces line 39 idx e=4",
            operands: vec![
                Operand::Buffers {
                    reads: &[(0x223AF4, In::F)],
                    writes: &[],
                },
                Operand::Value(4),
                Operand::RaceBlock {
                    reads: &[(0x214BA0, In::F), (0x31A184, In::F)],
                    writes: &[(0x31A1E8, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                steer_sine_force(
                    &mut race().shared.car_mut(),
                    &mut race().shared.physics_tables_mut(),
                    race().engine_tables(),
                    a[1],
                );
            },
        },
        Block {
            name: "drive_grip_forces line 54",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x1C5D38, In::F), (0x22400C, In::I), (0x31A1CC, In::I)],
                writes: &[(0x214768, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                lever_share_from_timer(
                    &race().shared.car(),
                    &mut race().shared.physics_tables_mut(),
                    &crate::test_support::race_places::numbers(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 74",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x213FC8, In::F),
                        (0x2142A8, In::F),
                        (0x214B74, In::F),
                        (0x31A184, In::F),
                    ],
                    writes: &[(0x214260, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                grip_limit_for(
                    &race().shared.car(),
                    &mut race().shared.physics_tables_mut(),
                    view::<PlainFloat>(a[1], 0).get(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 98",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x22400C, In::F), (0x31A1E8, In::F)],
                writes: &[(0x22400C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                let scratch = scene().scratch_cell();
                divide_into_scratch(
                    crate::test_support::own::race().shared.car().grip_forces[0],
                    scratch.float(),
                    &scratch,
                );
            },
        },
        Block {
            name: "drive_grip_forces line 106",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x22400C, In::F), (0x31A61C, In::I)],
                writes: &[(0x31A61C, Out::I)],
            }],
            ulps: 0,
            plain: |_| {
                scale_steering(
                    &mut race().shared.car_mut(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 122",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x31A17C, In::F), (0x31A184, In::F)],
                writes: &[(0x22400C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                ground_speed_into_scratch(
                    &race().shared.car(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 134 idx e=0",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213F34, In::F),
                        (0x213F38, In::F),
                        (0x22400C, In::F),
                        (0x31A190, In::F),
                        (0x31A610, In::I),
                        (0x31A618, In::I),
                    ],
                    writes: &[(0x31A1F4, Out::F)],
                },
                Operand::Buffers {
                    reads: &[(0x223AE4, In::F)],
                    writes: &[],
                },
                Operand::Value(0),
            ],
            ulps: 0,
            plain: |a| {
                lean_force(
                    &mut race().shared.car_mut(),
                    &crate::test_support::race_places::numbers(),
                    race().engine_tables(),
                    a[2],
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 134 idx e=1",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213F34, In::F),
                        (0x213F38, In::F),
                        (0x22400C, In::F),
                        (0x31A190, In::F),
                        (0x31A610, In::I),
                        (0x31A618, In::I),
                    ],
                    writes: &[(0x31A1F4, Out::F)],
                },
                Operand::Buffers {
                    reads: &[(0x223AE8, In::F)],
                    writes: &[],
                },
                Operand::Value(1),
            ],
            ulps: 0,
            plain: |a| {
                lean_force(
                    &mut race().shared.car_mut(),
                    &crate::test_support::race_places::numbers(),
                    race().engine_tables(),
                    a[2],
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 134 idx e=4",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213F34, In::F),
                        (0x213F38, In::F),
                        (0x22400C, In::F),
                        (0x31A190, In::F),
                        (0x31A610, In::I),
                        (0x31A618, In::I),
                    ],
                    writes: &[(0x31A1F4, Out::F)],
                },
                Operand::Buffers {
                    reads: &[(0x223AF4, In::F)],
                    writes: &[],
                },
                Operand::Value(4),
            ],
            ulps: 0,
            plain: |a| {
                lean_force(
                    &mut race().shared.car_mut(),
                    &crate::test_support::race_places::numbers(),
                    race().engine_tables(),
                    a[2],
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 151",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x1C5DB4, In::F), (0x214BF0, In::F), (0x31A614, In::I)],
                writes: &[(0x22400C, Out::I)],
            }],
            ulps: 0,
            plain: |_| {
                braking_count_geared(
                    &race().shared.car(),
                    &mut race().shared.physics_tables_mut(),
                    &crate::test_support::race_places::numbers(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 163",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x214BF0, In::F), (0x31A614, In::I)],
                writes: &[(0x22400C, Out::I)],
            }],
            ulps: 0,
            plain: |_| {
                braking_count_neutral(
                    &race().shared.car(),
                    &mut race().shared.physics_tables_mut(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 176",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E2C, In::F),
                    (0x213ECC, In::F),
                    (0x214164, In::I),
                    (0x214B78, In::F),
                    (0x31A1A0, In::F),
                ],
                writes: &[(0x31A1EC, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                engine_braking_force(
                    &mut race().shared.car_mut(),
                    &mut race().shared.physics_tables_mut(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 195",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x31A184, In::F)],
                writes: &[(0x22401C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                abs_speed_into_work(&race().shared.car(), crate::test_support::own::scene());
            },
        },
        Block {
            name: "drive_grip_forces line 202",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x213E7C, In::F), (0x22401C, In::F)],
                writes: &[(0x22401C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                yaw_share_from_limit(
                    &crate::test_support::race_places::numbers(),
                    crate::test_support::own::scene(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 215",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x213E80, In::F), (0x22400C, In::F)],
                writes: &[(0x22400C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                let scratch = scene().scratch_cell();
                divide_into_scratch(
                    scratch.float(),
                    crate::test_support::race_places::numbers()
                        .grip_force_limit
                        .get(),
                    &scratch,
                );
            },
        },
        Block {
            name: "drive_grip_forces line 222 idx e=0",
            operands: vec![
                Operand::Buffers {
                    reads: &[(0x223AE4, In::F)],
                    writes: &[],
                },
                Operand::Value(0),
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213E78, In::F),
                        (0x214BA0, In::F),
                        (0x22400C, In::F),
                        (0x22401C, In::F),
                        (0x31A610, In::I),
                    ],
                    writes: &[(0x31A1F0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                yaw_force(
                    &mut race().shared.car_mut(),
                    &mut race().shared.physics_tables_mut(),
                    &crate::test_support::race_places::numbers(),
                    race().engine_tables(),
                    a[1],
                    crate::test_support::own::scene(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 222 idx e=1",
            operands: vec![
                Operand::Buffers {
                    reads: &[(0x223AE8, In::F)],
                    writes: &[],
                },
                Operand::Value(1),
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213E78, In::F),
                        (0x214BA0, In::F),
                        (0x22400C, In::F),
                        (0x22401C, In::F),
                        (0x31A610, In::I),
                    ],
                    writes: &[(0x31A1F0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                yaw_force(
                    &mut race().shared.car_mut(),
                    &mut race().shared.physics_tables_mut(),
                    &crate::test_support::race_places::numbers(),
                    race().engine_tables(),
                    a[1],
                    crate::test_support::own::scene(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 222 idx e=4",
            operands: vec![
                Operand::Buffers {
                    reads: &[(0x223AF4, In::F)],
                    writes: &[],
                },
                Operand::Value(4),
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x213E78, In::F),
                        (0x214BA0, In::F),
                        (0x22400C, In::F),
                        (0x22401C, In::F),
                        (0x31A610, In::I),
                    ],
                    writes: &[(0x31A1F0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                yaw_force(
                    &mut race().shared.car_mut(),
                    &mut race().shared.physics_tables_mut(),
                    &crate::test_support::race_places::numbers(),
                    race().engine_tables(),
                    a[1],
                    crate::test_support::own::scene(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_grip_forces line 235",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x31A1E8, In::F),
                    (0x31A1EC, In::F),
                    (0x31A1F0, In::F),
                    (0x31A1F4, In::F),
                ],
                writes: &[(0x31A1DC, Out::F)],
            }],
            ulps: 0,
            plain: |_| sum_grip_forces(&mut race().shared.car_mut()),
        },
        Block {
            name: "drive_grip_forces line 253",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E40, In::F),
                        (0x1C5E48, In::F),
                        (0x21388C, In::F),
                        (0x213FAC, In::F),
                        (0x2149D0, In::F),
                        (0x2149E4, In::F),
                        (0x2149F0, In::F),
                        (0x214A04, In::F),
                        (0x22400C, In::F),
                        (0x31A1D8, In::F),
                        (0x31A1DC, In::F),
                        (0x31ACC8, In::F),
                        (0x31ACD4, In::F),
                    ],
                    writes: &[
                        (0x2149D0, Out::F),
                        (0x2149E4, Out::F),
                        (0x2149F0, Out::F),
                        (0x214A04, Out::F),
                        (0x22400C, Out::F),
                        (0x31A1D8, Out::F),
                    ],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                ease_yaw_twist(
                    &mut race().shared.car_mut(),
                    &mut race().shared.physics_tables_mut(),
                    &crate::test_support::race_places::numbers(),
                    race().course.grip_change(),
                    HostCell(a[1]).bits(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.drive_grip_forces", &blocks(), 1000, 100);
}
