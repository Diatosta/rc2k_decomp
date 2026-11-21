use super::*;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "nearest_on_segment differences",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (0x00, In::F),
                        (0x08, In::F),
                        (0x0C, In::F),
                        (0x14, In::F),
                        (0x18, In::F),
                        (0x20, In::F),
                    ],
                    writes: &[
                        (0x34, Out::F),
                        (0x38, Out::F),
                        (0x3C, Out::F),
                        (0x40, Out::F),
                        (0x4C, Out::F),
                        (0x50, Out::F),
                    ],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::T)],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::T)],
                },
            ],
            ulps: 0,
            plain: |a| differences(a[0], a[1], a[2]),
        },
        Block {
            name: "nearest_on_segment ratio",
            operands: vec![
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0x30, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::T)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::T)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| ratio(a[0], a[1], a[2]),
        },
        Block {
            name: "nearest_on_segment point",
            operands: vec![Operand::Buffer {
                reads: &[
                    (0x00, In::F),
                    (0x04, In::F),
                    (0x08, In::F),
                    (0x0C, In::F),
                    (0x10, In::F),
                    (0x14, In::F),
                    (0x30, In::F),
                ],
                writes: &[(0x24, Out::F), (0x28, Out::F), (0x2C, Out::F)],
            }],
            ulps: 0,
            plain: |a| point(a[0]),
        },
    ]
}

#[test]
fn nearest_on_segment_matches_the_assembly() {
    sparse("math.nearest_on_segment", &blocks(), 1500, 208);
}
