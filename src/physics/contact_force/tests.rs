use super::*;
use crate::math::x87_mem::{Base, view};
use crate::test_support::own::race;
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
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
                        .contact_turn_cross,
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
                        .contact_car_speed,
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
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
                    crate::test_support::own::race().physics.contact.get(),
                    crate::test_support::own::race().course.numbers.get(),
                    &crate::test_support::own::scene().scratch_cell(),
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
                    crate::test_support::own::race().physics.contact.get(),
                    crate::test_support::own::race().course.numbers.get(),
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
            plain: |_| x87_4(crate::test_support::own::race().physics.contact.get()),
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
                    x87_5(
                        race().physics.contact.get(),
                        race().shared.physics_tables.get(),
                    ),
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
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
                        .contact_normal_push,
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
                        .contact_extra,
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
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
                        crate::test_support::own::race().physics.contact.get(),
                        &crate::test_support::own::scene().scratch_cell(),
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
                    crate::test_support::own::race().physics.contact.get(),
                    &crate::test_support::own::scene().scratch_cell(),
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
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
                        .contact_slide,
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
                        .contact_normal_push,
                    &crate::test_support::own::race()
                        .physics
                        .contact
                        .get()
                        .contact_total,
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
                    race().physics.contact.get(),
                    race().shared.physics_tables.get(),
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
