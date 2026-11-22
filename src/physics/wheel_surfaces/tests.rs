use super::*;
use crate::math::x87_mem::view;
use crate::test_support::own::race;
use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

/// The dwords of a test buffer a run with index `k` reads (`k + 8` of them).
fn table(base: usize, k: usize) -> &'static [u32] {
    // SAFETY: the buffer holds the offsets up to `k * 4 + 0x1C` and 16 bytes more,
    // lent for the run and aligned for its cells.
    unsafe {
        std::slice::from_raw_parts(
            crate::test_support::vectors::block_vectors::at(base) as *const u32,
            k + 8,
        )
    }
}

fn blocks() -> Vec<Block> {
    vec![
        Block {
            name: "wheel_surfaces line 68 idx c=0",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::F)],
                    writes: &[],
                },
                Operand::Value(0),
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F)],
                    writes: &[(0x22400C, Out::I)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let whole = x87_1(
                    &crate::test_support::race_places::numbers(),
                    view(a[0], 0),
                    a[1],
                );
                crate::test_support::own::scene().draw_work_mut().scratch = whole;
            },
        },
        Block {
            name: "wheel_surfaces line 68 idx c=1",
            operands: vec![
                Operand::Buffer {
                    reads: &[(4, In::F)],
                    writes: &[],
                },
                Operand::Value(1),
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F)],
                    writes: &[(0x22400C, Out::I)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let whole = x87_1(
                    &crate::test_support::race_places::numbers(),
                    view(a[0], 0),
                    a[1],
                );
                crate::test_support::own::scene().draw_work_mut().scratch = whole;
            },
        },
        Block {
            name: "wheel_surfaces line 68 idx c=4",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x10, In::F)],
                    writes: &[],
                },
                Operand::Value(4),
                Operand::RaceBlock {
                    reads: &[(0x1C5D98, In::F)],
                    writes: &[(0x22400C, Out::I)],
                },
            ],
            ulps: 0,
            plain: |a| {
                let whole = x87_1(
                    &crate::test_support::race_places::numbers(),
                    view(a[0], 0),
                    a[1],
                );
                crate::test_support::own::scene().draw_work_mut().scratch = whole;
            },
        },
        Block {
            name: "wheel_surfaces line 134 idx c=0",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x224040, In::I),
                        (0x224044, In::I),
                        (0x224048, In::I),
                        (0x2D2A14, In::F),
                        (0x2D2B14, In::F),
                    ],
                    writes: &[(0x31A984, Out::F)],
                },
                Operand::Value(0),
            ],
            ulps: 0,
            plain: |a| {
                x87_2(
                    &crate::test_support::own::race().physics.probes().borrow(),
                    &crate::test_support::race_places::numbers(),
                    &mut crate::test_support::own::race().shared.car_mut(),
                    a[1],
                    crate::test_support::own::scene(),
                );
            },
        },
        Block {
            name: "wheel_surfaces line 134 idx c=1",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x224040, In::I),
                        (0x224044, In::I),
                        (0x224048, In::I),
                        (0x2D2A18, In::F),
                        (0x2D2B18, In::F),
                    ],
                    writes: &[(0x31A988, Out::F)],
                },
                Operand::Value(1),
            ],
            ulps: 0,
            plain: |a| {
                x87_2(
                    &crate::test_support::own::race().physics.probes().borrow(),
                    &crate::test_support::race_places::numbers(),
                    &mut crate::test_support::own::race().shared.car_mut(),
                    a[1],
                    crate::test_support::own::scene(),
                );
            },
        },
        Block {
            name: "wheel_surfaces line 134 idx c=4",
            operands: vec![
                Operand::RaceBlock {
                    reads: &[
                        (0x1C5E2C, In::F),
                        (0x224040, In::I),
                        (0x224044, In::I),
                        (0x224048, In::I),
                        (0x2D2A24, In::F),
                        (0x2D2B24, In::F),
                    ],
                    writes: &[(0x31A994, Out::F)],
                },
                Operand::Value(4),
            ],
            ulps: 0,
            plain: |a| {
                x87_2(
                    &crate::test_support::own::race().physics.probes().borrow(),
                    &crate::test_support::race_places::numbers(),
                    &mut crate::test_support::own::race().shared.car_mut(),
                    a[1],
                    crate::test_support::own::scene(),
                );
            },
        },
        Block {
            name: "wheel_surfaces line 161 idx c=0",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x28, In::F), (0x2C, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0x28, In::F), (0x2C, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A984, In::F)],
                    writes: &[(0x2142C0, Out::F), (0x31A920, Out::F)],
                },
                Operand::Value(0),
            ],
            ulps: 0,
            plain: |a| {
                x87_3(
                    &race().shared,
                    &mut race().shared.car_mut(),
                    view(a[0], 0),
                    view(a[1], 0),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 161 idx c=1",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x28, In::F), (0x2C, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0x28, In::F), (0x2C, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A988, In::F)],
                    writes: &[(0x2142C4, Out::F), (0x31A924, Out::F)],
                },
                Operand::Value(1),
            ],
            ulps: 0,
            plain: |a| {
                x87_3(
                    &race().shared,
                    &mut race().shared.car_mut(),
                    view(a[0], 0),
                    view(a[1], 0),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 161 idx c=4",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0x28, In::F), (0x2C, In::F)],
                    writes: &[],
                },
                Operand::Buffer {
                    reads: &[(0x28, In::F), (0x2C, In::F)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x31A994, In::F)],
                    writes: &[(0x2142D0, Out::F), (0x31A930, Out::F)],
                },
                Operand::Value(4),
            ],
            ulps: 0,
            plain: |a| {
                x87_3(
                    &race().shared,
                    &mut race().shared.car_mut(),
                    view(a[0], 0),
                    view(a[1], 0),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 178 idx k=0",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I), (0x1C, In::I)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                    writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::I), (4, In::I), (0x1C, In::I)],
                    writes: &[],
                },
                Operand::Value(0),
            ],
            ulps: 0,
            plain: |a| {
                x87_4(
                    &race().shared,
                    &race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    table(a[0], a[3]),
                    table(a[2], a[3]),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 178 idx k=1",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I), (8, In::I), (0x20, In::I)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                    writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::I), (8, In::I), (0x20, In::I)],
                    writes: &[],
                },
                Operand::Value(1),
            ],
            ulps: 0,
            plain: |a| {
                x87_4(
                    &race().shared,
                    &race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    table(a[0], a[3]),
                    table(a[2], a[3]),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 178 idx k=4",
            operands: vec![
                Operand::Buffer {
                    reads: &[(0, In::I), (0x14, In::I), (0x2C, In::I)],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[(0x1C5CFC, In::F), (0x31A258, In::F)],
                    writes: &[(0x21411C, Out::F), (0x214120, Out::F)],
                },
                Operand::Buffer {
                    reads: &[(0, In::I), (0x14, In::I), (0x2C, In::I)],
                    writes: &[],
                },
                Operand::Value(4),
            ],
            ulps: 0,
            plain: |a| {
                x87_4(
                    &race().shared,
                    &race().shared.car(),
                    &crate::test_support::race_places::numbers(),
                    table(a[0], a[3]),
                    table(a[2], a[3]),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 201 idx c=0",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (4, In::F),
                        (8, In::F),
                        (0xC, In::F),
                        (0x10, In::F),
                        (0x14, In::F),
                        (0x20, In::F),
                        (0x24, In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[
                        (0x21411C, In::F),
                        (0x214120, In::F),
                        (0x214BB4, In::F),
                        (0x214BB8, In::F),
                        (0x214BBC, In::F),
                        (0x31A984, In::F),
                    ],
                    writes: &[
                        (0x214284, Out::F),
                        (0x214298, Out::F),
                        (0x2142AC, Out::F),
                        (0x2146C0, Out::F),
                        (0x2146D4, Out::F),
                        (0x2146E8, Out::F),
                        (0x2146FC, Out::F),
                    ],
                },
                Operand::Buffer {
                    reads: &[
                        (4, In::F),
                        (8, In::F),
                        (0xC, In::F),
                        (0x10, In::F),
                        (0x14, In::F),
                        (0x20, In::F),
                        (0x24, In::F),
                    ],
                    writes: &[],
                },
                Operand::Value(0),
            ],
            ulps: 0,
            plain: |a| {
                x87_5(
                    &race().shared,
                    &race().shared.car(),
                    view(a[0], 0),
                    view(a[2], 0),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 201 idx c=1",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (4, In::F),
                        (8, In::F),
                        (0xC, In::F),
                        (0x10, In::F),
                        (0x14, In::F),
                        (0x20, In::F),
                        (0x24, In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[
                        (0x21411C, In::F),
                        (0x214120, In::F),
                        (0x214BB4, In::F),
                        (0x214BB8, In::F),
                        (0x214BBC, In::F),
                        (0x31A988, In::F),
                    ],
                    writes: &[
                        (0x214288, Out::F),
                        (0x21429C, Out::F),
                        (0x2142B0, Out::F),
                        (0x2146C4, Out::F),
                        (0x2146D8, Out::F),
                        (0x2146EC, Out::F),
                        (0x214700, Out::F),
                    ],
                },
                Operand::Buffer {
                    reads: &[
                        (4, In::F),
                        (8, In::F),
                        (0xC, In::F),
                        (0x10, In::F),
                        (0x14, In::F),
                        (0x20, In::F),
                        (0x24, In::F),
                    ],
                    writes: &[],
                },
                Operand::Value(1),
            ],
            ulps: 0,
            plain: |a| {
                x87_5(
                    &race().shared,
                    &race().shared.car(),
                    view(a[0], 0),
                    view(a[2], 0),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 201 idx c=4",
            operands: vec![
                Operand::Buffer {
                    reads: &[
                        (4, In::F),
                        (8, In::F),
                        (0xC, In::F),
                        (0x10, In::F),
                        (0x14, In::F),
                        (0x20, In::F),
                        (0x24, In::F),
                    ],
                    writes: &[],
                },
                Operand::RaceBlock {
                    reads: &[
                        (0x21411C, In::F),
                        (0x214120, In::F),
                        (0x214BB4, In::F),
                        (0x214BB8, In::F),
                        (0x214BBC, In::F),
                        (0x31A994, In::F),
                    ],
                    writes: &[
                        (0x214294, Out::F),
                        (0x2142A8, Out::F),
                        (0x2142BC, Out::F),
                        (0x2146D0, Out::F),
                        (0x2146E4, Out::F),
                        (0x2146F8, Out::F),
                        (0x21470C, Out::F),
                    ],
                },
                Operand::Buffer {
                    reads: &[
                        (4, In::F),
                        (8, In::F),
                        (0xC, In::F),
                        (0x10, In::F),
                        (0x14, In::F),
                        (0x20, In::F),
                        (0x24, In::F),
                    ],
                    writes: &[],
                },
                Operand::Value(4),
            ],
            ulps: 0,
            plain: |a| {
                x87_5(
                    &race().shared,
                    &race().shared.car(),
                    view(a[0], 0),
                    view(a[2], 0),
                    a[3],
                );
            },
        },
        Block {
            name: "wheel_surfaces line 258",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x213FEC, In::F),
                    (0x214284, In::F),
                    (0x214288, In::F),
                    (0x21428C, In::F),
                    (0x214290, In::F),
                ],
                writes: &[(0x214268, Out::F)],
            }],
            ulps: 0,
            plain: |_| x87_6(&crate::test_support::race_places::numbers(), &race().shared),
        },
        Block {
            name: "wheel_surfaces line 299",
            operands: vec![Operand::RaceBlock {
                reads: &[
                    (0x1C5E2C, In::F),
                    (0x213F5C, In::F),
                    (0x213F60, In::F),
                    (0x213F64, In::F),
                    (0x213F68, In::F),
                    (0x2146D0, In::F),
                    (0x2146E4, In::F),
                    (0x2146F8, In::F),
                    (0x21470C, In::F),
                    (0x214C04, In::F),
                    (0x224040, In::F),
                    (0x224044, In::F),
                    (0x31A00C, In::I),
                    (0x31A038, In::F),
                    (0x31A258, In::F),
                ],
                writes: &[
                    (0x214230, Out::F),
                    (0x214760, Out::F),
                    (0x214764, Out::F),
                    (0x223DA8, Out::F),
                    (0x223DAC, Out::F),
                    (0x224040, Out::F),
                    (0x224044, Out::F),
                    (0x31A038, Out::F),
                ],
            }],
            ulps: 0,
            plain: |_| {
                let numbers = crate::test_support::race_places::numbers();
                let inputs = crate::physics::step_inputs::StepInputs::of(race()).wheels();
                x87_7(
                    &WheelInputs {
                        numbers: &numbers,
                        ..inputs
                    },
                    crate::test_support::own::scene(),
                    &mut race().shared.car_mut(),
                );
            },
        },
    ]
}

#[test]
fn x87_blocks_matches_the_assembly() {
    sparse("physics.wheel_surfaces", &blocks(), 550, 100);
}
