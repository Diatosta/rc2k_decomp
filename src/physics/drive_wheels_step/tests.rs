use super::*;
use crate::math::x87_mem::{Base, view};
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "drive_wheels_step line 33",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D38, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                one_less_abs(
                    &crate::test_support::race_places::numbers(),
                    view::<PlainFloat>(a[0], 0).ld(),
                    view::<PlainFloat>(a[2], 0),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 47",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::H)],
                },
            ],
            ulps: 0,
            plain: |a| a[1].set_word(0, abs_status(*view(a[0], 0))),
        },
        Block {
            name: "drive_wheels_step line 73",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x214BA8, In::F), (0x31A184, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                slip_ratio(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    view::<PlainFloat>(a[0], 0),
                    view::<PlainFloat>(a[2], 0),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 87",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[(0, Out::F)],
                },
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5D38, In::F),
                        (0x1C5E40, In::F),
                        (0x21388C, In::F),
                        (0x214BA8, In::F),
                    ],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                apply_slip_to_drive(
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                    view::<PlainFloat>(a[0], 0),
                    view::<PlainFloat>(a[2], 0).bits(),
                    view::<PlainFloat>(a[3], 0),
                    view::<PlainFloat>(a[4], 0),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 106",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x214BA8, In::F), (0x31A184, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                slip_ratio_size(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    view::<PlainFloat>(a[0], 0),
                    view::<PlainFloat>(a[2], 0),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 127",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x1C5D38, In::F), (0x2140B4, In::F)],
                writes: &[(0x2140B8, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                signed_share(
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 147 idx e=0",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x21388C, In::F),
                        (0x214170, In::F),
                        (0x214284, In::F),
                        (0x214288, In::F),
                        (0x21428C, In::F),
                        (0x214290, In::F),
                        (0x214298, In::F),
                        (0x21429C, In::F),
                        (0x2142A0, In::F),
                        (0x2142A4, In::F),
                        (0x214774, In::F),
                        (0x214794, In::F),
                        (0x2147B4, In::F),
                        (0x2147D4, In::F),
                        (0x31A3D8, In::F),
                        (0x31A3DC, In::F),
                    ],
                    writes: &[(0x21423C, Out::F), (0x214248, Out::F), (0x21424C, Out::F)],
                },
                Operand::Value(0),
            ],
            ulps: 0,
            plain: |a| {
                drive_forces_from_grips(
                    race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    race().shared.physics_tables.get(),
                    race().physics.wheel_work(),
                    if a[1] == 0 {
                        &crate::test_support::own::race()
                            .shared
                            .physics_tables
                            .get()
                            .wheel_grips
                    } else {
                        &crate::test_support::own::race()
                            .shared
                            .physics_tables
                            .get()
                            .wheel_grips_b
                    },
                );
            },
        },
        Block {
            name: "drive_wheels_step line 147 idx e=0x14 (braking)",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x21388C, In::F),
                        (0x214170, In::F),
                        (0x214284, In::F),
                        (0x214288, In::F),
                        (0x21428C, In::F),
                        (0x214290, In::F),
                        (0x2142AC, In::F),
                        (0x2142B0, In::F),
                        (0x2142B4, In::F),
                        (0x2142B8, In::F),
                        (0x214774, In::F),
                        (0x214794, In::F),
                        (0x2147B4, In::F),
                        (0x2147D4, In::F),
                        (0x31A3D8, In::F),
                        (0x31A3DC, In::F),
                    ],
                    writes: &[(0x21423C, Out::F), (0x214248, Out::F), (0x21424C, Out::F)],
                },
                Operand::Value(0x14),
            ],
            ulps: 0,
            plain: |a| {
                drive_forces_from_grips(
                    race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    race().shared.physics_tables.get(),
                    race().physics.wheel_work(),
                    if a[1] == 0 {
                        &crate::test_support::own::race()
                            .shared
                            .physics_tables
                            .get()
                            .wheel_grips
                    } else {
                        &crate::test_support::own::race()
                            .shared
                            .physics_tables
                            .get()
                            .wheel_grips_b
                    },
                );
            },
        },
        Block {
            name: "drive_wheels_step line 180",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E2C, In::F),
                    (0x213EC4, In::F),
                    (0x214170, In::F),
                    (0x21423C, In::F),
                    (0x214268, In::F),
                    (0x22400C, In::I),
                    (0x31A17C, In::F),
                ],
                writes: &[(0x214240, Out::F), (0x22400C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                throttle_drive_limit(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 203",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E40, In::F),
                    (0x21388C, In::F),
                    (0x214750, In::F),
                    (0x214BA8, In::F),
                    (0x31A190, In::F),
                    (0x31A194, In::F),
                ],
                writes: &[(0x31A190, Out::F), (0x31A194, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                push_both_drive_speeds(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 220",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x21388C, In::F),
                    (0x214750, In::F),
                    (0x214BA8, In::F),
                    (0x31A190, In::F),
                ],
                writes: &[(0x31A190, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                push_drive_speed_a(
                    race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    race().shared.physics_tables.get(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 231",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E2C, In::F),
                    (0x21388C, In::F),
                    (0x214BA8, In::F),
                    (0x214BB0, In::F),
                    (0x214BF0, In::F),
                    (0x31A3D0, In::F),
                    (0x31A614, In::I),
                ],
                writes: &[(0x21422C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                brake_step_for_a(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 245",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E2C, In::F),
                    (0x21388C, In::F),
                    (0x214BA8, In::F),
                    (0x214BB0, In::F),
                    (0x214BF0, In::F),
                    (0x31A3D4, In::F),
                    (0x31A614, In::I),
                ],
                writes: &[(0x21422C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                brake_step_for_b(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 258",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E2C, In::F),
                    (0x21388C, In::F),
                    (0x214170, In::F),
                    (0x214BA8, In::F),
                    (0x214BAC, In::F),
                    (0x31A618, In::I),
                ],
                writes: &[(0x21422C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                handbrake_step_for_b(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 273",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F), (8, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::H)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let v: [u32; 3] = *view(a[0], 0);
                a[1].set_word(0, handbrake_stop_status(v[0], v[1], v[2]));
            },
        },
        Block {
            name: "drive_wheels_step line 292",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x31A184, In::F), (0x31A190, In::F), (0x31A194, In::F)],
                writes: &[(0x214234, Out::F), (0x214238, Out::F)],
            }],
            ulps: 0,
            plain: |_| slip_gaps(race().shared.car(), race().physics.wheel_work()),
        },
        Block {
            name: "drive_wheels_step line 308",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x2140B0, In::F),
                    (0x2140B4, In::F),
                    (0x214244, In::F),
                    (0x214248, In::F),
                    (0x21424C, In::F),
                ],
                writes: &[(0x214244, Out::F), (0x214248, Out::F), (0x21424C, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                scale_axle_forces(
                    race().shared.physics_tables.get(),
                    race().physics.wheel_work(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 331",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5DA4, In::F),
                    (0x21388C, In::F),
                    (0x22400C, In::F),
                    (0x31A17C, In::F),
                ],
                writes: &[
                    (0x214250, Out::F),
                    (0x2148D0, Out::F),
                    (0x2148F0, Out::F),
                    (0x214910, Out::F),
                    (0x214930, Out::F),
                ],
            }],
            ulps: 0,
            plain: |_| {
                side_slip_forces(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    race().physics.wheel_work(),
                    &crate::test_support::race_places::numbers(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 347",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x213ED0, In::F),
                    (0x214250, In::F),
                    (0x214254, In::F),
                    (0x214264, In::F),
                    (0x214BFC, In::F),
                ],
                writes: &[(0x2141D4, Out::F), (0x214254, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                slip_side_share(
                    &crate::test_support::race_places::numbers(),
                    race().shared.physics_tables.get(),
                    race().physics.wheel_work(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 362",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x213ED4, In::F), (0x214BFC, In::F)],
                writes: &[(0x2141D8, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                spin_push_b(
                    &crate::test_support::race_places::numbers(),
                    race().shared.physics_tables.get(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 372",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x214764, In::F), (0x31A17C, In::F), (0x31A184, In::F)],
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
                    speed_over_ground(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        &crate::test_support::own::scene().scratch_cell(),
                    ),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 385",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x214760, In::F),
                    (0x22400C, In::F),
                    (0x31A17C, In::F),
                    (0x31A184, In::F),
                ],
                writes: &[(0x2148B0, Out::F), (0x2148B8, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                rolling_drag_forces(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    &crate::test_support::own::scene().scratch_cell(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 403",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E40, In::F),
                    (0x2148D0, In::F),
                    (0x2148F0, In::F),
                    (0x214910, In::F),
                    (0x214930, In::F),
                    (0x2149D0, In::F),
                    (0x2149F0, In::F),
                ],
                writes: &[
                    (0x2148D0, Out::F),
                    (0x2148F0, Out::F),
                    (0x214910, Out::F),
                    (0x214930, Out::F),
                ],
            }],
            ulps: 0,
            plain: |_| {
                share_wheel_forces(
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 426",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x213EDC, In::F), (0x31A184, In::F), (0x31A190, In::F)],
                writes: &[(0x22401C, Out::F), (0x31A158, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                slip_figure_a(
                    race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    crate::test_support::own::scene(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 436",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x213ED8, In::F), (0x31A17C, In::F)],
                writes: &[(0x224020, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                slip_figure_b(
                    race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    crate::test_support::own::scene(),
                );
            },
        },
        Block {
            name: "drive_wheels_step line 444",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x22401C, In::F),
                    (0x224020, In::F),
                    (0x31A17C, In::F),
                    (0x31A184, In::F),
                    (0x31A190, In::F),
                    (0x31A194, In::F),
                ],
                writes: &[
                    (0x22401C, Out::F),
                    (0x224020, Out::F),
                    (0x31A260, Out::F),
                    (0x31AF18, Out::F),
                    (0x31AF1C, Out::F),
                ],
            }],
            ulps: 0,
            plain: |_| slip_lengths(race().shared.car(), crate::test_support::own::scene()),
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.drive_wheels_step", &blocks(), 550, 100);
}
