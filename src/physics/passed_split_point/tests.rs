use super::*;

/// The point in a test's operand buffer.
fn point_at(s: usize) -> [u32; 3] {
    // SAFETY: the operand buffer is leased for the run and holds the three dwords.
    unsafe {
        crate::test_support::vectors::block_vectors::at(s)
            .cast::<[u32; 3]>()
            .read_unaligned()
    }
}
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

#[test]
fn distance_and_angle_match_the_assembly() {
    let blocks = vec![
        Block {
            name: "passed_split_point distance",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x239824, In::I), (0x23982C, In::I), (0x1C5DA0, In::F)],
                    writes: &[(0x22400C, Out::I), (0x214D04, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::I), (8, In::I)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                distance(
                    &mut crate::test_support::own::race().shared.physics_tables_mut(),
                    &crate::test_support::race_places::numbers(),
                    &crate::test_support::own::race()
                        .physics
                        .ground_search()
                        .borrow(),
                    point_at(a[1]),
                );
                // The original leaves the last gap in the shared work dword.
                let ground_z = crate::test_support::own::race()
                    .physics
                    .ground_search()
                    .borrow()
                    .z;
                crate::test_support::own::scene().draw_work_mut().scratch =
                    (point_at(a[1])[2] as i32).wrapping_sub(ground_z as i32) as u32;
            },
        },
        Block {
            name: "passed_split_point angle",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x239824, In::I),
                        (0x23982C, In::I),
                        (0x1C5E40, In::F),
                        (0x1C5D98, In::F),
                    ],
                    writes: &[(0x22400C, Out::I)],
                },
                Operand::Buffer {
                    reads: &[(0, In::I), (8, In::I)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                let turns = angle(
                    &crate::test_support::race_places::numbers(),
                    &crate::test_support::own::race()
                        .physics
                        .ground_search()
                        .borrow(),
                    point_at(a[1]),
                );
                crate::test_support::own::scene().draw_work_mut().scratch = turns;
            },
        },
    ];
    sparse("physics.passed_split_point", &blocks, 6000, 202);
}
