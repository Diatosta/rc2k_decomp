use super::*;
use crate::math::x87_mem::view;
use crate::memory::session::PlainFloat;
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

/// Three floats of a test buffer at `off`.
fn three(base: usize, off: usize) -> &'static [PlainFloat; 3] {
    view::<[PlainFloat; 3]>(base, off)
}

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "bump_cars line 66",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x31A170, In::F), (0x31A174, In::F), (0x31A178, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F), (4, Out::F), (8, Out::F)],
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
                    three(a[1], 0),
                    view::<[crate::memory::session::Float; 3]>(a[2], 0x198),
                );
            },
        },
        Block {
            name: "bump_cars line 80",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A170, In::F), (0x31A174, In::F), (0x31A178, In::F)],
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
                    view::<PlainFloat>(a[0], 0).ld(),
                    view::<PlainFloat>(a[0], 4).ld(),
                    view::<[crate::memory::session::Float; 3]>(a[2], 0x198),
                );
            },
        },
        Block {
            name: "bump_cars line 121",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A10C, In::I), (0x31A110, In::I), (0x31A114, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::F), (4, Out::F), (8, Out::F)],
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
                    view::<PlainFloat>(a[1], 0).ld(),
                    three(a[3], 0),
                    &crate::test_support::race_places::numbers(),
                    view::<[crate::memory::session::Dword; 3]>(a[4], 0x134),
                );
            },
        },
        Block {
            name: "bump_cars line 142",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F), (8, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A170, In::F), (0x31A174, In::F), (0x31A178, In::F)],
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
                    three(a[0], 0),
                    view::<[crate::memory::session::Float; 3]>(a[2], 0x198),
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.bump_cars", &blocks(), 1200, 100);
}
