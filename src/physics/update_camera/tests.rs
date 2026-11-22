use super::*;
use crate::math::x87_mem::view;
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "update_camera line 64",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I), (8, In::I)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F), (0x1C5DD4, In::F), (0x1C5E28, In::F)],
                    writes: &[(0x31A688, Out::I), (0x31A690, Out::I)],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_1(
                    race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    *view(a[0], 0),
                );
            },
        },
        Block {
            name: "update_camera line 89",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I), (8, In::I)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F)],
                    writes: &[(0x2062D8, Out::F), (0x2062DC, Out::F), (0x2062E0, Out::F)],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_2(
                    &crate::test_support::race_places::numbers(),
                    &crate::test_support::own::race().frame.view_and_records,
                    *view(a[0], 0),
                );
            },
        },
        Block {
            name: "update_camera line 109",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x1C5D48, In::F), (0x20631C, In::F), (0x31A6A0, In::I)],
                writes: &[(0x31A6A0, Out::F)],
            }],
            ulps: 0,
            plain: |_| {
                let car = crate::test_support::own::race().shared.car();
                // The block reads the zoom as a whole number (the 384 the step puts there).
                x87_3(
                    car,
                    &crate::test_support::race_places::numbers(),
                    &crate::test_support::own::race().frame.view_and_records,
                    car.camera_zoom.get().to_bits() as i32,
                );
            },
        },
        Block {
            name: "update_camera line 133",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (0, In::I),
                        (4, In::I),
                        (8, In::I),
                        (0x10, In::I),
                        (0x14, In::I),
                        (0x18, In::I),
                        (0x20, In::I),
                        (0x24, In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5DA0, In::F), (0x213FC0, In::F)],
                    writes: &[
                        (0x206570, Out::F),
                        (0x206574, Out::F),
                        (0x2065C8, Out::F),
                        (0x2065CC, Out::F),
                        (0x2065D0, Out::F),
                        (0x206624, Out::F),
                        (0x206628, Out::F),
                        (0x20662C, Out::F),
                    ],
                },
            ],
            ulps: 0,
            plain: |a| {
                x87_4(
                    &race().shared.physics_tables(),
                    &crate::test_support::race_places::numbers(),
                    &race().frame.view_and_records,
                    &core::array::from_fn(|k| {
                        if k < 10 {
                            crate::math::x87_mem::Base::dword(a[0], 4 * k)
                        } else {
                            0
                        }
                    }),
                );
            },
        },
        Block {
            name: "update_camera line 190",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x319FE0, In::I), (0x31A668, In::I), (0x31A670, In::I)],
                writes: &[(0x31A674, Out::F)],
            }],
            ulps: 0,
            plain: |_| x87_5(race().shared.car()),
        },
        Block {
            name: "update_camera line 205",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x208B5C, In::I),
                    (0x208B60, In::I),
                    (0x208B64, In::I),
                    (0x208B68, In::I),
                    (0x208B6C, In::I),
                    (0x208B70, In::I),
                    (0x31A674, In::F),
                ],
                writes: &[(0x208B50, Out::I), (0x208B54, Out::I), (0x208B58, Out::I)],
            }],
            ulps: 0,
            plain: |_| x87_6(&mut race().physics.camera_mut(), race().shared.car()),
        },
        Block {
            name: "update_camera line 235",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F), (0x31A674, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[],
                    writes: &[(0, Out::I)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let result = x87_7(
                    race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    view::<crate::memory::session::Float>(a[0], 0).get(),
                    view::<crate::memory::session::Float>(a[1], 0).get(),
                );
                view::<crate::memory::session::Dword>(a[3], 0).set(result);
            },
        },
        Block {
            name: "update_camera line 257",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x208BF0, In::I)],
                writes: &[(0x31A6A0, Out::F)],
            }],
            ulps: 0,
            plain: |_| x87_8(&race().physics.camera(), race().shared.car()),
        },
        Block {
            name: "update_camera line 266",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x208B30, In::I), (0x208B34, In::I), (0x208B38, In::I)],
                writes: &[(0x1CCD80, Out::I)],
            }],
            ulps: 0,
            plain: |_| {
                let flat_depth = x87_9(&crate::test_support::own::race().physics.camera());
                crate::test_support::own::race()
                    .course
                    .course
                    .borrow_mut()
                    .flat_depth = flat_depth;
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.update_camera", &blocks(), 1200, 100);
}
