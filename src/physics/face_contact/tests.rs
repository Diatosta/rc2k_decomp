use super::*;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "face_contact line 111",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5DD4, In::F),
                    (0x1C5E2C, In::F),
                    (0x2D3B4C, In::I),
                    (0x2D3B54, In::I),
                ],
                writes: &[(0x2D3B70, Out::I)],
            }],
            ulps: 0,
            plain: |_| {
                x87_1(
                    &mut crate::test_support::own::race()
                        .physics
                        .contact()
                        .borrow_mut(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
        Block {
            name: "face_contact line 121",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5D98, In::F),
                    (0x2D3090, In::I),
                    (0x2D3094, In::I),
                    (0x2D3098, In::I),
                    (0x31A170, In::F),
                    (0x31A174, In::F),
                    (0x31A178, In::F),
                ],
                writes: &[
                    (0x2D3B58, Out::F),
                    (0x2D3B5C, Out::F),
                    (0x2D3B60, Out::F),
                    (0x2D3B74, Out::I),
                ],
            }],
            ulps: 0,
            plain: |_| {
                x87_2(
                    &mut crate::test_support::own::race()
                        .physics
                        .contact()
                        .borrow_mut(),
                    crate::test_support::own::race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.face_contact", &blocks(), 1200, 100);
}
