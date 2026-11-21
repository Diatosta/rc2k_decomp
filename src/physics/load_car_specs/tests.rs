use super::*;
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

/// A record with the bytes `from .. to` of the test buffer at `base` (the rest zero):
/// the buffers hold only the offsets the run reads.
fn spec_at(base: usize, from: usize, to: usize) -> Box<CarSpec> {
    // SAFETY: cells and byte arrays only, all-zero is valid; the buffer holds `to` bytes.
    let mut spec: Box<CarSpec> = Box::new(unsafe { std::mem::zeroed() });
    use crate::test_support::vectors::block_vectors::{LAID_ROOM, at};
    assert!(from <= to && to <= size_of::<CarSpec>() && to <= LAID_ROOM);
    // SAFETY: the buffer at `base` holds at least `LAID_ROOM` bytes (`block_vectors`), so
    // `from .. to` (asserted inside it) lies in it; the record is this function's own box,
    // written through its unique pointer inside its size (asserted), any bytes valid for its
    // cells and byte arrays; the two do not overlap.
    unsafe {
        std::ptr::copy_nonoverlapping(
            at(base + from),
            (&raw mut *spec).cast::<u8>().add(from),
            to - from,
        );
    }
    spec
}

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "load_car_specs line 100",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (0x64, In::I),
                        (0x68, In::I),
                        (0x6C, In::I),
                        (0x70, In::I),
                        (0x74, In::I),
                        (0x78, In::I),
                        (0xA4, In::F),
                        (0xA8, In::F),
                        (0xAC, In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5DDC, In::F),
                        (0x213E88, In::F),
                        (0x213EB8, In::F),
                        (0x213F10, In::F),
                        (0x213FE4, In::F),
                        (0x31A1B4, In::F),
                    ],
                    writes: &[
                        (0x1CB958, Out::F),
                        (0x1CB95C, Out::F),
                        (0x1CB960, Out::F),
                        (0x1CB964, Out::F),
                        (0x1CB968, Out::F),
                        (0x1CB96C, Out::F),
                        (0x1CB970, Out::F),
                        (0x1CB974, Out::F),
                        (0x1CB978, Out::F),
                        (0x214104, Out::F),
                        (0x31A1B4, Out::F),
                        (0x31A1B8, Out::F),
                    ],
                },
            ],
            ulps: 0,
            plain: |a| {
                load_figures_and_shift_points(
                    race().shared.car(),
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                    race().gear(),
                    &spec_at(a[0], 0x64, 0xB0),
                );
            },
        },
        Block {
            name: "load_car_specs line 145",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x40, In::I), (0x44, In::I), (0x48, In::I)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D18, In::F), (0x1C5D98, In::F)],
                    writes: &[(0x214BC0, Out::I), (0x214BC4, Out::I), (0x214BC8, Out::I)],
                },
            ],
            ulps: 0,
            plain: |a| {
                scale_record_ratios(
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                    &spec_at(a[0], 0x40, 0x4C),
                );
            },
        },
        Block {
            name: "load_car_specs line 162",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5D18, In::F),
                    (0x1C5D98, In::F),
                    (0x213F04, In::I),
                    (0x213F08, In::I),
                    (0x213F0C, In::I),
                ],
                writes: &[(0x214BC0, Out::I), (0x214BC4, Out::I), (0x214BC8, Out::I)],
            }],
            ulps: 0,
            plain: |_| {
                scale_default_ratios(
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "load_car_specs line 182",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x38, In::F), (0x3C, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x213FC8, In::F)],
                    writes: &[(0x214BAC, Out::F), (0x214BB0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                scale_record_grips(
                    race().shared.physics_tables.get(),
                    &spec_at(a[0], 0x38, 0x40),
                );
            },
        },
        Block {
            name: "load_car_specs line 194 idx k=0",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x1C5D14, In::F), (0x213FB8, In::F)],
                    writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                },
                Operand::Value(0),
            ],
            ulps: 0,
            plain: |a| {
                set_rev_limits(
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                    a[1],
                );
            },
        },
        Block {
            name: "load_car_specs line 194 idx k=1",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x1C5D14, In::F), (0x213FBC, In::F)],
                    writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                },
                Operand::Value(1),
            ],
            ulps: 0,
            plain: |a| {
                set_rev_limits(
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                    a[1],
                );
            },
        },
        Block {
            name: "load_car_specs line 194 idx k=2 (camera_drop)",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x1C5D14, In::F), (0x213FC0, In::F)],
                    writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                },
                Operand::Value(2),
            ],
            ulps: 0,
            plain: |a| {
                set_rev_limits(
                    race().shared.physics_tables.get(),
                    &crate::test_support::race_places::numbers(),
                    a[1],
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.load_car_specs", &blocks(), 1200, 100);
}
