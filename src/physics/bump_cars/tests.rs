use super::*;
use crate::math::x87_mem::view;
use crate::memory::session::PlainFloat;
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

/// Three floats of a test buffer at `off`.
fn three(base: usize, off: usize) -> &'static [PlainFloat; 3] {
    view::<[PlainFloat; 3]>(base, off)
}

/// The floats of three cells.
fn floats(cells: &[PlainFloat; 3]) -> [f32; 3] {
    cells.each_ref().map(PlainFloat::get)
}

/// Three cells put the floats.
fn put(cells: &[PlainFloat; 3], values: [f32; 3]) {
    for (cell, value) in cells.iter().zip(values) {
        cell.set(value);
    }
}

type Row = crate::memory::plain_floats::PlainFloats<3>;

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
                let mut relative = floats(three(a[1], 0));
                relative_speed(
                    &race().shared.car(),
                    &mut relative,
                    &view::<Row>(a[2], 0x198).get(),
                );
                put(three(a[1], 0), relative);
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
                let other = view::<Row>(a[2], 0x198);
                let mut speed = other.get();
                share_out_speeds(
                    &mut race().shared.car_mut(),
                    crate::math::x87_float::ld(view::<PlainFloat>(a[0], 0).bits()),
                    crate::math::x87_float::ld(view::<PlainFloat>(a[0], 4).bits()),
                    &mut speed,
                );
                other.set(speed);
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
                let mut push = floats(three(a[3], 0));
                push_apart(
                    &crate::test_support::own::race().shared.car(),
                    crate::math::x87_float::ld(view::<PlainFloat>(a[1], 0).bits()),
                    &mut push,
                    &crate::test_support::race_places::numbers(),
                    &view::<Row>(a[4], 0x134).get(),
                );
                put(three(a[3], 0), push);
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
                let other = view::<Row>(a[2], 0x198);
                let mut speed = other.get();
                add_push(
                    &mut race().shared.car_mut(),
                    &floats(three(a[0], 0)),
                    &mut speed,
                );
                other.set(speed);
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.bump_cars", &blocks(), 1200, 100);
}
