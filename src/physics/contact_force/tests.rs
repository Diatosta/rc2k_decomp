use super::*;
use crate::math::x87_mem::{Base, view};
use crate::memory::session::PlainDword;
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

/// Three floats of a test buffer at `off`.
fn three(base: usize, off: usize) -> &'static [PlainFloat; 3] {
    view::<[PlainFloat; 3]>(base, off)
}

/// One float of a test buffer at `off`.
fn one(base: usize, off: usize) -> &'static PlainFloat {
    view::<PlainFloat>(base, off)
}

/// The three floats' bits.
fn bits(cells: &[PlainFloat; 3]) -> [u32; 3] {
    crate::math::cells::load(cells)
}

/// Three buffer floats set to the bits `v`.
fn set(cells: &[PlainFloat; 3], v: [u32; 3]) {
    for k in 0..3 {
        cells[k].set_bits(v[k]);
    }
}

/// `out = a + b`, as the vector sum steps.
fn add_into(a: &[PlainFloat; 3], b: [u32; 3], out: &[PlainFloat; 3]) {
    set(out, add(bits(a), b));
}

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "contact_force line 50",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x2D30A8, In::F), (0x2D30AC, In::F), (0x2D30B0, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F), (8, In::F)],
                    writes: &[(12, Out::F), (16, Out::F), (20, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                add_into(
                    three(a[1], 0),
                    race().physics.contact().borrow().contact_car_speed,
                    three(a[1], 12),
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
                let normal_y = race().physics.contact().borrow().contact_normal[1];
                let depth = x87_2(normal_y, race().course.numbers.get());
                crate::test_support::own::scene().draw_work_mut().scratch = depth;
            },
        },
        Block {
            name: "contact_force line 86",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x1C5D38, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F), (8, In::F), (12, In::F)],
                    writes: &[(12, Out::F), (16, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let (share, factor) = x87_3(
                    one(a[1], 0).ld(),
                    race().course.numbers.get(),
                    one(a[1], 4).bits(),
                    one(a[1], 8).bits(),
                );
                one(a[1], 12).set_bits(share);
                one(a[1], 16).set_bits(factor);
            },
        },
        Block {
            name: "contact_force line 107",
            operands: vec![Operand::Buffer {
                reads: &[
                    (0, In::F),
                    (4, In::F),
                    (8, In::F),
                    (12, In::F),
                    (16, In::F),
                    (20, In::F),
                ],
                writes: &[(12, Out::F), (16, Out::F), (20, Out::F)],
            }],
            ulps: 0,
            plain: |a| {
                let side = x87_4(bits(three(a[0], 12)), bits(three(a[0], 0)));
                set(three(a[0], 12), side);
            },
        },
        Block {
            name: "contact_force line 126",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x214148, In::I), (0x21414C, In::I)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::T)],
                },
            ],
            ulps: 0,
            plain: |a| {
                a[2].set_tbyte(
                    0,
                    x87_5(one(a[1], 0).ld(), race().shared.physics_tables.get()),
                );
            },
        },
        Block {
            name: "contact_force line 137",
            operands: vec![Operand::Buffer {
                reads: &[
                    (0, In::F),
                    (4, In::F),
                    (8, In::F),
                    (12, In::F),
                    (16, In::F),
                    (20, In::F),
                ],
                writes: &[(0, Out::F), (4, Out::F), (8, Out::F)],
            }],
            ulps: 0,
            plain: |a| add_into(three(a[0], 0), bits(three(a[0], 12)), three(a[0], 0)),
        },
        Block {
            name: "contact_force line 160",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F), (8, In::F), (12, In::F)],
                    writes: &[(16, Out::F)],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::H)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let (length, status) = x87_6(bits(three(a[0], 4)), one(a[0], 0).bits());
                view::<PlainDword>(a[0], 16).set(length);
                a[1].set_word(0, status);
            },
        },
        Block {
            name: "contact_force line 176",
            operands: vec![Operand::Buffer {
                reads: &[(0, In::F), (4, In::F), (8, In::F), (12, In::F), (16, In::F)],
                writes: &[(0, Out::F), (8, Out::F), (12, Out::F), (16, Out::F)],
            }],
            ulps: 0,
            plain: |a| {
                let (slide, held) = x87_7(
                    one(a[0], 4).bits(),
                    bits(three(a[0], 8)),
                    view::<PlainDword>(a[0], 0).get(),
                );
                set(three(a[0], 8), slide);
                view::<PlainDword>(a[0], 0).set(held);
            },
        },
        Block {
            name: "contact_force line 193",
            operands: vec![Operand::Buffer {
                reads: &[
                    (0, In::F),
                    (4, In::F),
                    (8, In::F),
                    (12, In::F),
                    (16, In::F),
                    (20, In::F),
                ],
                writes: &[(24, Out::F), (28, Out::F), (32, Out::F)],
            }],
            ulps: 0,
            plain: |a| add_into(three(a[0], 12), bits(three(a[0], 0)), three(a[0], 24)),
        },
        Block {
            name: "contact_force line 207",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x21388C, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F), (4, In::F), (8, In::F)],
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
                    bits(three(a[1], 0)),
                    race().shared.physics_tables.get(),
                    three(a[2], 0),
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.contact_force", &blocks(), 1000, 100);
}
