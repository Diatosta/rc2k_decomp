use super::*;
use crate::math::x87_mem::view;
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "bump_cars line 66",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x31A170, In::F), (0x31A174, In::F), (0x31A178, In::F)],
                    writes: &[(0x2D3CD8, Out::F), (0x2D3CDC, Out::F), (0x2D3CE0, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                relative_speed(
                    race().shared.car(),
                    race().physics.contact.get(),
                    view::<[crate::memory::session::Float; 3]>(a[1], 0x198),
                );
            },
        },
        Block {
            name: "bump_cars line 80",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x2D3BAC, In::F),
                        (0x2D3BB0, In::F),
                        (0x31A170, In::F),
                        (0x31A174, In::F),
                        (0x31A178, In::F),
                    ],
                    writes: &[(0x31A170, Out::F), (0x31A174, Out::F), (0x31A178, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                    writes: &[(0x198, Out::F), (0x19C, Out::F), (0x1A0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                share_out_speeds(
                    race().shared.car(),
                    race().physics.contact.get(),
                    view::<[crate::memory::session::Float; 3]>(a[1], 0x198),
                );
            },
        },
        Block {
            name: "bump_cars line 121",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5D98, In::F),
                        (0x2D3B88, In::F),
                        (0x31A10C, In::I),
                        (0x31A110, In::I),
                        (0x31A114, In::I),
                    ],
                    writes: &[(0x2D3B9C, Out::F), (0x2D3BA0, Out::F), (0x2D3BA4, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0x134, In::I), (0x138, In::I), (0x13C, In::I)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                push_apart(
                    crate::test_support::own::race().shared.car(),
                    crate::test_support::own::race().physics.contact.get(),
                    crate::test_support::own::race().course.numbers.get(),
                    view::<[crate::memory::session::Dword; 3]>(a[1], 0x134),
                );
            },
        },
        Block {
            name: "bump_cars line 142",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x2D3B9C, In::F),
                        (0x2D3BA0, In::F),
                        (0x2D3BA4, In::F),
                        (0x31A170, In::F),
                        (0x31A174, In::F),
                        (0x31A178, In::F),
                    ],
                    writes: &[(0x31A170, Out::F), (0x31A174, Out::F), (0x31A178, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0x198, In::F), (0x19C, In::F), (0x1A0, In::F)],
                    writes: &[(0x198, Out::F), (0x19C, Out::F), (0x1A0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                add_push(
                    race().shared.car(),
                    race().physics.contact.get(),
                    view::<[crate::memory::session::Float; 3]>(a[1], 0x198),
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.bump_cars", &blocks(), 1200, 100);
}
