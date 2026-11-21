use super::*;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "point_hits_car line 47",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5D98, In::F),
                    (0x2D3B4C, In::I),
                    (0x2D3B50, In::I),
                    (0x2D3B54, In::I),
                ],
                writes: &[(0x2D3B78, Out::F), (0x2D3B7C, Out::F), (0x2D3B80, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                x87_1(
                    &mut crate::test_support::own::race()
                        .physics
                        .contact()
                        .borrow_mut(),
                    crate::test_support::own::race().course.numbers.get(),
                );
            },
        },
        Block {
            name: "point_hits_car line 63",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5D98, In::F),
                    (0x2D3B58, In::F),
                    (0x2D3B5C, In::F),
                    (0x2D3B60, In::F),
                ],
                writes: &[(0x2D3090, Out::I), (0x2D3094, Out::I), (0x2D3098, Out::I)],
            }],
            ulps: 0,
            plain: |_| {
                x87_2(
                    &mut crate::test_support::own::race()
                        .physics
                        .contact()
                        .borrow_mut(),
                    crate::test_support::own::race().course.numbers.get(),
                );
            },
        },
        Block {
            name: "point_hits_car line 80",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x1C5D98, In::F), (0x2D3090, In::I), (0x2D3098, In::I)],
                writes: &[(0x31A1FC, Out::F), (0x31A200, Out::F), (0x31A204, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                x87_3(
                    &crate::test_support::own::race().physics.contact().borrow(),
                    crate::test_support::own::race().course.numbers.get(),
                    crate::test_support::own::race().shared.car(),
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.point_hits_car", &blocks(), 1200, 100);
}
