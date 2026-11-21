use super::*;
use crate::physics::store::laid;

/// The damage model as the test's buffer holds it.
fn dm(base: usize) -> &'static mut DamageModel {
    crate::physics::store::model_in_test_buffer(base)
}
use crate::math::x87_mem::view;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse_at};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "damage_wear line 36",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B390), In::F),
                        (laid!(0x35B3B0), In::F),
                        (laid!(0x35B584), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A284, In::F)],
                    writes: &[(0x31A284, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .brake_heat
                    .float_at(0);
                g.set(add_wear(
                    dm(a[0]),
                    dm(a[0]).wear_rates[3],
                    dm(a[0]).brake_share,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_wear line 47",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B390), In::F),
                        (laid!(0x35B3B0), In::F),
                        (laid!(0x35B584), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A288, In::F)],
                    writes: &[(0x31A288, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .brake_heat
                    .float_at(1);
                g.set(add_wear(
                    dm(a[0]),
                    dm(a[0]).wear_rates[3],
                    dm(a[0]).brake_share,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_wear line 58",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B390), In::F), (laid!(0x35B588), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A284, In::F)],
                    writes: &[(0x31A284, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .brake_heat
                    .float_at(0);
                g.set(subtract_rate(dm(a[0]), dm(a[0]).wear_rates[4], g.get()));
            },
        },
        Block {
            name: "damage_wear line 69",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B390), In::F), (laid!(0x35B588), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A288, In::F)],
                    writes: &[(0x31A288, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .brake_heat
                    .float_at(1);
                g.set(subtract_rate(dm(a[0]), dm(a[0]).wear_rates[4], g.get()));
            },
        },
        Block {
            name: "damage_wear line 80",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B390), In::F),
                        (laid!(0x35B3A8), In::F),
                        (laid!(0x35B58C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A27C, In::F)],
                    writes: &[(0x31A27C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .damper_wear
                    .float_at(0);
                g.set(add_wear(
                    dm(a[0]),
                    dm(a[0]).wear_rates[5],
                    dm(a[0]).drive_a_extrapolated_share,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_wear line 91",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B390), In::F),
                        (laid!(0x35B3AC), In::F),
                        (laid!(0x35B58C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A280, In::F)],
                    writes: &[(0x31A280, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .damper_wear
                    .float_at(1);
                g.set(add_wear(
                    dm(a[0]),
                    dm(a[0]).wear_rates[5],
                    dm(a[0]).drive_b_extrapolated_share,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_wear line 102",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B390), In::F), (laid!(0x35B590), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A27C, In::F)],
                    writes: &[(0x31A27C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .damper_wear
                    .float_at(0);
                g.set(subtract_rate(dm(a[0]), dm(a[0]).wear_rates[6], g.get()));
            },
        },
        Block {
            name: "damage_wear line 113",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B390), In::F), (laid!(0x35B590), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A280, In::F)],
                    writes: &[(0x31A280, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race()
                    .shared
                    .car()
                    .damper_wear
                    .float_at(1);
                g.set(subtract_rate(dm(a[0]), dm(a[0]).wear_rates[6], g.get()));
            },
        },
        Block {
            name: "damage_wear line 124",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B390), In::F),
                        (laid!(0x35B3B4), In::F),
                        (laid!(0x35B594), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A278, In::F)],
                    writes: &[(0x31A278, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().brake_fade;
                g.set(add_wear(
                    dm(a[0]),
                    dm(a[0]).wear_rates[7],
                    dm(a[0]).on_surface_5,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_wear line 135",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B390), In::F), (laid!(0x35B598), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A278, In::F)],
                    writes: &[(0x31A278, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().brake_fade;
                g.set(subtract_rate(dm(a[0]), dm(a[0]).wear_rates[8], g.get()));
            },
        },
        Block {
            name: "damage_wear line 146",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B390), In::F),
                        (laid!(0x35B3B0), In::F),
                        (laid!(0x35B59C), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A278, In::F)],
                    writes: &[(0x31A278, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().brake_fade;
                g.set(subtract_wear(
                    dm(a[0]),
                    dm(a[0]).wear_rates[9],
                    dm(a[0]).brake_share,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_wear line 159",
            operands: vec![Operand::Buffer {
                reads: &[(laid!(0x35B398), In::F), (laid!(0x35B460), In::F)],
                writes: &[(laid!(0x35B464), Out::F)],
            }],
            ulps: 0,
            plain: |a| lerp_gap(dm(a[0])),
        },
        Block {
            name: "damage_wear line 166",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B390), In::F),
                        (laid!(0x35B464), In::F),
                        (laid!(0x35B578), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A28C, In::F)],
                    writes: &[(0x31A28C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().engine_heat;
                g.set(add_wear(
                    dm(a[0]),
                    dm(a[0]).wear_rates[0],
                    dm(a[0]).overheat_excess,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_wear line 177",
            operands: vec![
                Operand::Buffer {
                    reads: &[(laid!(0x35B390), In::F), (laid!(0x35B57C), In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A28C, In::F)],
                    writes: &[(0x31A28C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().engine_heat;
                g.set(subtract_rate(dm(a[0]), dm(a[0]).wear_rates[1], g.get()));
            },
        },
        Block {
            name: "damage_wear line 188",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (laid!(0x35B390), In::F),
                        (laid!(0x35B39C), In::F),
                        (laid!(0x35B580), In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A28C, In::F)],
                    writes: &[(0x31A28C, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let g = &crate::test_support::own::race().shared.car().engine_heat;
                g.set(subtract_wear(
                    dm(a[0]),
                    dm(a[0]).wear_rates[2],
                    dm(a[0]).speed_share,
                    g.get(),
                ));
            },
        },
        Block {
            name: "damage_wear line 201",
            operands: vec![Operand::Buffer {
                reads: &[(laid!(0x35B44C), In::F)],
                writes: &[(laid!(0x35B44C), Out::F)],
            }],
            ulps: 0,
            plain: |a| {
                square_into(&mut dm(a[0]).stiffness_body_factor);
            },
        },
        Block {
            name: "damage_wear line 208",
            operands: vec![Operand::Buffer {
                reads: &[(laid!(0x35B448), In::F)],
                writes: &[(laid!(0x35B448), Out::F)],
            }],
            ulps: 0,
            plain: |a| {
                square_into(&mut dm(a[0]).stiffness_suspension_factor);
            },
        },
        Block {
            name: "damage_wear line 215",
            operands: vec![Operand::Buffer {
                reads: &[(laid!(0x35B468), In::F)],
                writes: &[(laid!(0x35B468), Out::F)],
            }],
            ulps: 0,
            plain: |a| {
                square_into(&mut dm(a[0]).ride_height_body_factor);
            },
        },
        Block {
            name: "damage_wear line 223",
            operands: vec![Operand::Buffer {
                reads: &[
                    (laid!(0x35B39C), In::F),
                    (laid!(0x35B3E0), In::F),
                    (laid!(0x35B3F4), In::F),
                    (laid!(0x35B448), In::F),
                    (laid!(0x35B44C), In::F),
                    (laid!(0x35B468), In::F),
                    (laid!(0x35B6F8), In::F),
                ],
                writes: &[
                    (laid!(0x35B450), Out::F),
                    (laid!(0x35B454), Out::F),
                    (laid!(0x35B458), Out::F),
                ],
            }],
            ulps: 0,
            plain: |a| hit_figures(dm(a[0])),
        },
        Block {
            name: "damage_wear line 252",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(laid!(0x35B3F0), In::F), (laid!(0x35B8BC), In::F)],
                    writes: &[(laid!(0x35B484), Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::I)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                blend_wear_figures(*view::<i32>(a[0], 0), dm(a[1]), *view::<i32>(a[2], 0));
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    // The plain code rounds to nearest, the mode the damage model always runs in.
    sparse_at("physics.damage_wear", &blocks(), 1200, 100, &[0x027F]);
}
