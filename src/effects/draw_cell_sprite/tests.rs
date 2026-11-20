use crate::effects::asm_layout::with_laid;
use crate::effects::asm_test::Kind::*;
use crate::effects::asm_test::{Op, Spec, recorded_writes};
use crate::effects::blocks_store::CornerSlopes;
use crate::effects::frame_state_store::DrawWork;
use crate::effects::ring_store::RingWork;
use crate::math::engine_tables::{cosine, sine};

use crate::effects::asm_test::vertex_cell;

/// Runs `f` with each cell holding the test buffer's dword at its address; what `f`
/// leaves goes back to the buffer, the cells as they were.
fn staged<R>(
    cells: &[(
        &dyn crate::memory::session::Cell32,
        crate::effects::asm_test::Operand,
    )],
    f: impl FnOnce() -> R,
) -> R {
    let host = |a: crate::effects::asm_test::Operand| a.at(0, 4).cast::<u32>();
    let saved: Vec<u32> = cells.iter().map(|(c, _)| c.bits()).collect();
    for &(c, a) in cells {
        // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a
        // `u32`.
        c.set_bits(unsafe { host(a).read_unaligned() });
    }
    let out = f();
    for (&(c, a), &old) in cells.iter().zip(&saved) {
        // SAFETY: `at` checks that the four bytes lie in the operand's test buffer, which the
        // recorded run keeps alive and nothing else reaches meanwhile; unaligned, any bits a
        // `u32`.
        unsafe { host(a).write_unaligned(c.bits()) };
        c.set_bits(old);
    }
    out
}

/// The three ten-byte values at `at` of the test buffer.
fn values(at: crate::effects::asm_test::Operand) -> [[u8; 10]; 3] {
    // SAFETY: `host` checks that the byte lies in the operand's test buffer, which the recorded
    // run keeps alive and nothing else reaches meanwhile.
    std::array::from_fn(|n| std::array::from_fn(|i| unsafe { (at + 10 * n + i).host().read() }))
}

#[test]
fn turn_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[
                    (0x1EA2CC, F32),
                    (0x1EA2D0, F32),
                    (0x1EA2D4, F32),
                    (0x1EA2DC, F32),
                    (0x1EA2E0, F32),
                    (0x1EA2E4, F32),
                    (0x1EA2EC, F32),
                    (0x1EA2F0, F32),
                    (0x1EA2F4, F32),
                    (0x1EA2FC, F32),
                    (0x1EA300, F32),
                    (0x1EA304, F32),
                ],
                writes: &[],
            },
            Op::Base {
                lo: 0x186931,
                hi: 0x1D0826,
                reads: &[],
                writes: &[
                    (0x186995, F32),
                    (0x186999, F32),
                    (0x18699D, F32),
                    (0x1D0780, I32),
                ],
            },
            Op::Base {
                lo: 0x0,
                hi: 0x1E,
                reads: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                writes: &[],
            },
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_cell_sprite.turn_x87", &SPEC, |b| {
        with_laid::<RingWork, _>(b[1], |w| {
            with_laid::<DrawWork, _>(b[1], |dw| {
                super::turn_x87(
                    &crate::test_support::own::race()
                        .frame
                        .matrices
                        .get()
                        .view_matrix,
                    &values(b[2]),
                    w,
                    dw,
                );
            });
        });
    });
}

#[test]
fn square_x87_matches() {
    const SPEC: Spec = Spec {
        operands: &[
            Op::RaceBlock {
                reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                writes: &[],
            },
            Op::Base {
                lo: 0x0,
                hi: 0xC8544,
                reads: &[
                    (0xC3544, F32),
                    (0xC3548, F32),
                    (0xC35A8, F32),
                    (0xC4544, F32),
                    (0xC4548, F32),
                    (0xC45A8, F32),
                    (0xC7540, F32),
                    (0xC8540, F32),
                ],
                writes: &[
                    (0x0, F32),
                    (0x4, F32),
                    (0x8, F32),
                    (0x20, F32),
                    (0x24, F32),
                    (0x28, F32),
                    (0x40, F32),
                    (0x44, F32),
                    (0x48, F32),
                    (0x60, F32),
                    (0x64, F32),
                    (0x68, F32),
                ],
            },
            Op::Base {
                lo: 0x186931,
                hi: 0x1BDBF4,
                reads: &[
                    (0x18697D, F32),
                    (0x18697D, I32),
                    (0x18697D, U32),
                    (0x186981, F32),
                    (0x186981, U32),
                    (0x186985, F32),
                    (0x186989, F32),
                    (0x186995, F32),
                    (0x186999, F32),
                    (0x18699D, F32),
                    (0x186A2D, F32),
                    (0x1BB7F4, F32),
                    (0x1BB804, F32),
                    (0x1BB824, F32),
                    (0x1BBBE4, F32),
                    (0x1BBBF4, F32),
                ],
                writes: &[
                    (0x18697D, F32),
                    (0x186981, F32),
                    (0x186985, F32),
                    (0x186985, U32),
                    (0x186989, F32),
                    (0x186989, U32),
                ],
            },
            Op::Index(&[0, 16, 48, 1008, 1024]),
            Op::Index(&[0, 4, 100, 16380]),
        ],
        sincos: false,
    };
    recorded_writes("effects.draw_cell_sprite.square_x87", &SPEC, |b| {
        with_laid::<DrawWork, _>(b[2], |dw| {
            with_laid::<std::cell::RefCell<CornerSlopes>, _>(b[2], |cs| {
                let vertices: Vec<(
                    &dyn crate::memory::session::Cell32,
                    crate::effects::asm_test::Operand,
                )> = [
                    0x0, 0x4, 0x8, 0x20, 0x24, 0x28, 0x40, 0x44, 0x48, 0x60, 0x64, 0x68,
                ]
                .into_iter()
                .map(|o| {
                    (
                        vertex_cell(o) as &dyn crate::memory::session::Cell32,
                        b[1].wrapping_add(o),
                    )
                })
                .collect();
                let tables = b[1].wrapping_add(0xC3544);
                let mut cells = vertices;
                cells.push((
                    sine(
                        &crate::test_support::own::race()
                            .shared
                            .buffers
                            .reads
                            .get()
                            .engine_tables,
                        b[4].number() / 4,
                    ),
                    tables.wrapping_add(b[4].number()),
                ));
                cells.push((
                    cosine(
                        &crate::test_support::own::race()
                            .shared
                            .buffers
                            .reads
                            .get()
                            .engine_tables,
                        b[4].number() / 4,
                    ),
                    tables.wrapping_add(b[4].number() + 0x1000),
                ));
                staged(&cells, || {
                    super::square_x87(
                        crate::owners::Owners::the()
                            .scene
                            .scratch
                            .front_end_polygon(),
                        &crate::test_support::own::race()
                            .shared
                            .buffers
                            .reads
                            .get()
                            .engine_tables,
                        &crate::d3d::vertices_store::VERTICES,
                        b[3].number(),
                        b[4].number() / 4,
                        dw,
                        &cs.borrow(),
                    )
                })
            })
        })
    });
}
