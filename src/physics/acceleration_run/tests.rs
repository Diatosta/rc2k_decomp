use super::*;
use crate::math::x87_mem::view;
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "test_drive_run line 42",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[(0x1C5E34, In::F), (0x1CB968, In::F), (0x214748, In::F)],
                    writes: &[(0x214750, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_1(
                    &mut race().shared.physics_tables_mut(),
                    race().gear(),
                    &crate::test_support::race_places::numbers(),
                    view::<crate::memory::session::PlainFloat>(a[1], 0).get(),
                );
            },
        },
        Block {
            name: "test_drive_run line 51",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E34, In::F),
                    (0x21388C, In::F),
                    (0x213FCC, In::F),
                    (0x213FD0, In::F),
                    (0x214750, In::F),
                    (0x31A190, In::F),
                ],
                writes: &[(0x22400C, Out::F), (0x31A190, Out::F), (0x31A194, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                let drag = x87_2(
                    &mut race().shared.car_mut(),
                    &race().shared.physics_tables(),
                    &crate::test_support::race_places::numbers(),
                );
                // The original keeps the drag, stored as a float, in the shared work dword.
                crate::test_support::own::scene().draw_work_mut().scratch =
                    crate::math::x87_float::st(drag);
            },
        },
        Block {
            name: "test_drive_run line 93",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5D00, In::F),
                    (0x1C5D14, In::F),
                    (0x1CB97C, In::I),
                    (0x1CB980, In::I),
                    (0x31A190, In::F),
                ],
                writes: &[(0x1CB97C, Out::F), (0x1CB980, Out::F), (0x1CB984, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                x87_3(
                    &race().shared.car(),
                    race().gear(),
                    &crate::test_support::race_places::numbers(),
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    // The recorded vectors are filed under the function's earlier name.
    sparse("physics.test_drive_run", &blocks(), 1200, 100);
}
