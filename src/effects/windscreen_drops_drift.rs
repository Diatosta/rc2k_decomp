//! `effects::windscreen_drops_drift` (0x4BCBE9): the windscreen drops' drift from the view's moves.
//! The drift is set to 0; unless this is the first time (a flag, then set), and unless it is
//! the second (another flag, then set), the view's move (the settings' +0x26AC .. less the
//! kept place, times the clock constant) less the last move (plus the last times the decay)
//! is turned by the adjugate of the view's turn (the view matrix copied by columns,
//! `effects::adjugate3`), the matrix made a turn about x by the drift's angle
//! (`scene::turn_matrix3_x`) and the move turned back by it, negated, into the drops' move
//! and times the drift's scales into the drift; then the move is kept, and the place kept.
//!
//! Works on the effects settings.
//!
//! Caller: `effects::windscreen_drops_frame` The x87 runs are plain Rust on the x87
//! model (`math::x87_float`), in the original's order, over the game's variables (the three values
//! it keeps on the x87 stack across the two calls kept as 80 bits); the calls their Rust versions.
//! Refuses null settings, before anything.

use crate::effects::adjugate3::adjugate3;
use crate::effects::cell_x87::CellX87;
use crate::effects::setup::MarksClock;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;
use crate::race::effects_settings_store::EffectsSettings;
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::{Axis, extended, turn_matrix3};
use crate::session::{PlainBytes, PlainFloat};

/// The view's move (the settings' +0x26AC .. less the kept place, times the clock constant),
/// less the last move, plus the last move times the drift's decay.
fn view_moved(
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    effects: &crate::effects::owner::Effects,
) {
    view_moved_x87(drops, m, &effects.setup.clock);
}

/// The view's move kept for the next frame.
fn move_kept(
    drops: &WindscreenDrops,
    m: &EffectsSettings,
    effects: &crate::effects::owner::Effects,
) {
    move_kept_x87(drops, m, &effects.setup.clock);
}

pub fn windscreen_drops_drift(
    frame: &Frame,
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let m = effects.settings()?;
    let marks = effects.settings()?;
    for n in 0..3 {
        drops.tilt.point[n].set_bits(0);
    }
    let first: u32 = drops.hex.started.get();
    drops.hex.started.set(1);
    if first != 0 {
        let second: u32 = drops.hex.moved.get();
        drops.hex.moved.set(1);
        if second != 0 {
            view_moved(drops, m, effects);
            let view = &frame.matrices.view_place;
            for col in 0..3usize {
                let v = [view[col].bits(), view[4 + col].bits(), view[8 + col].bits()];
                for (k, x) in v.into_iter().enumerate() {
                    drops.hex.turn[3 * col + k].set_bits(x);
                }
            }
            adjugate3(&drops.hex.turn, &drops.hex.adjugate);
            // The two temporaries `[esp]` held and the three 80-bit values kept across the two calls.
            let temps = [PlainFloat::new(0.0), PlainFloat::new(0.0)];
            let kept = PlainBytes::new([0u8; 30]);
            drift_turned_x87(drops, &drops.hex.adjugate, &temps, &kept);
            turn_matrix3(Axis::X, &drops.hex.turn, extended(drops.hex.angle.get()))?;
            drift_back_x87(drops, &drops.hex.turn, &temps, &kept);
        }
        move_kept(drops, m, effects);
    }
    for (k, v) in [
        marks.listener_car_place[0].bits(),
        marks.listener_car_place[1].bits(),
        marks.listener_car_place[2].bits(),
    ]
    .into_iter()
    .enumerate()
    {
        drops.hex.kept_place[k].set_bits(v);
    }
    Ok(())
}

use crate::effects::plain::{ld_cell, ld80_at, st_cell, st80_at};
use crate::math::x87_float as bh;

/// `view_moved`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
fn view_moved_x87(drops: &WindscreenDrops, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&m.listener_car_place[0]);
    let t1 = t0 - ld_cell(&drops.hex.kept_place[0]);
    let t2 = t1 * c.reciprocal.ld();
    let t3 = ld_cell(&m.listener_car_place[1]);
    let t4 = t3 - ld_cell(&drops.hex.kept_place[1]);
    let t5 = t4 * c.reciprocal.ld();
    let t6 = ld_cell(&m.listener_car_place[2]);
    let t7 = t6 - ld_cell(&drops.hex.kept_place[2]);
    let t8 = t7 * c.reciprocal.ld();
    st_cell(&drops.hex.view_move[2], t8);
    st_cell(&drops.hex.view_move[1], t5);
    st_cell(&drops.hex.view_move[0], t2);
    let t9 = ld_cell(&drops.hex.view_move[0]);
    let t10 = t9 - ld_cell(&drops.hex.kept_move[0]);
    let t11 = ld_cell(&drops.hex.view_move[1]);
    let t12 = t11 - ld_cell(&drops.hex.kept_move[1]);
    let t13 = ld_cell(&drops.hex.view_move[2]);
    let t14 = t13 - ld_cell(&drops.hex.kept_move[2]);
    st_cell(&drops.hex.move_change[2], t14);
    st_cell(&drops.hex.move_change[1], t12);
    st_cell(&drops.hex.move_change[0], t10);
    let t15 = ld_cell(&drops.hex.kept_move[0]);
    let t16 = t15 * ld_cell(&drops.hex.decay[0]);
    let t17 = t16 + ld_cell(&drops.hex.move_change[0]);
    st_cell(&drops.hex.move_change[0], t17);
    let t18 = ld_cell(&drops.hex.kept_move[1]);
    let t19 = t18 * ld_cell(&drops.hex.decay[1]);
    let t20 = t19 + ld_cell(&drops.hex.move_change[1]);
    st_cell(&drops.hex.move_change[1], t20);
    let t21 = ld_cell(&drops.hex.kept_move[2]);
    let t22 = t21 * ld_cell(&drops.hex.decay[2]);
    let t23 = t22 + ld_cell(&drops.hex.move_change[2]);
    st_cell(&drops.hex.move_change[2], t23);
}

/// `drift_turned`'s x87 block as plain Rust on the x87 model (33 instructions), same order and memory places.
fn drift_turned_x87(
    drops: &WindscreenDrops,
    m: &[PlainFloat; 9],
    t: &[PlainFloat; 2],
    l: &PlainBytes<30>,
) {
    let t0 = ld_cell(&drops.hex.move_change[2]);
    let t1 = ld_cell(&drops.hex.move_change[1]);
    let t2 = ld_cell(&drops.hex.move_change[0]);
    let t3 = t2 * ld_cell(&m[0]);
    let t4 = t1 * ld_cell(&m[1]);
    let t5 = t0 * ld_cell(&m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st_cell(&t[0], t7);
    let t8 = t2 * ld_cell(&m[3]);
    let t9 = t1 * ld_cell(&m[4]);
    let t10 = t0 * ld_cell(&m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st_cell(&t[1], t12);
    let t13 = t2 * ld_cell(&m[6]);
    let t14 = t1 * ld_cell(&m[7]);
    let t15 = t0 * ld_cell(&m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld_cell(&t[1]);
    let t19 = ld_cell(&t[0]);
    st80_at(l, 0, t19);
    st80_at(l, 10, t18);
    st80_at(l, 20, t17);
}

/// `drift_back`'s x87 block as plain Rust on the x87 model (40 instructions), same order and memory places.
fn drift_back_x87(
    drops: &WindscreenDrops,
    m: &[PlainFloat; 9],
    t: &[PlainFloat; 2],
    l: &PlainBytes<30>,
) {
    let t0 = bh::from_f80(ld80_at(l, 20));
    let t1 = bh::from_f80(ld80_at(l, 10));
    let t2 = bh::from_f80(ld80_at(l, 0));
    let t3 = t2 * ld_cell(&m[0]);
    let t4 = t1 * ld_cell(&m[1]);
    let t5 = t0 * ld_cell(&m[2]);
    let t6 = t4 + t5;
    let t7 = t3 + t6;
    st_cell(&t[0], t7);
    let t8 = t2 * ld_cell(&m[3]);
    let t9 = t1 * ld_cell(&m[4]);
    let t10 = t0 * ld_cell(&m[5]);
    let t11 = t9 + t10;
    let t12 = t8 + t11;
    st_cell(&t[1], t12);
    let t13 = t2 * ld_cell(&m[6]);
    let t14 = t1 * ld_cell(&m[7]);
    let t15 = t0 * ld_cell(&m[8]);
    let t16 = t13 + t15;
    let t17 = t14 + t16;
    let t18 = ld_cell(&t[1]);
    let t19 = ld_cell(&t[0]);
    let t20 = -t19;
    st_cell(&drops.hex.drift[0], t20);
    let t21 = t20 * ld_cell(&drops.hex.result_k[0]);
    drops.tilt.point[0].st(t21);
    st_cell(&drops.hex.drift[1], t18);
    let t22 = t18 * ld_cell(&drops.hex.result_k[1]);
    drops.tilt.point[1].st(t22);
    st_cell(&drops.hex.drift[2], t17);
    let t23 = t17 * ld_cell(&drops.hex.result_k[2]);
    drops.tilt.point[2].st(t23);
}

/// `move_kept`'s x87 block as plain Rust on the x87 model (12 instructions), same order and memory places.
fn move_kept_x87(drops: &WindscreenDrops, m: &EffectsSettings, c: &MarksClock) {
    let t0 = ld_cell(&m.listener_car_place[0]);
    let t1 = t0 - ld_cell(&drops.hex.kept_place[0]);
    let t2 = t1 * c.reciprocal.ld();
    let t3 = ld_cell(&m.listener_car_place[1]);
    let t4 = t3 - ld_cell(&drops.hex.kept_place[1]);
    let t5 = t4 * c.reciprocal.ld();
    let t6 = ld_cell(&m.listener_car_place[2]);
    let t7 = t6 - ld_cell(&drops.hex.kept_place[2]);
    let t8 = t7 * c.reciprocal.ld();
    st_cell(&drops.hex.kept_move[2], t8);
    st_cell(&drops.hex.kept_move[1], t5);
    st_cell(&drops.hex.kept_move[0], t2);
}

#[cfg(test)]
mod reference_vectors {
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::with_settings;
    use crate::effects::asm_test::{Op, Spec, recorded_writes};
    use crate::effects::tilt_shape::ShapeTilt;
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_vars::WindscreenDropVars;
    use crate::session::{PlainBytes, PlainFloat};

    /// Runs `f` with `HEX` holding the buffer's bytes (`typed_sync`).
    fn hex<R>(mk: usize, f: impl FnOnce() -> R) -> R {
        with_overlays(
            mk,
            &[overlay(
                &crate::effects::owner::effects().windscreen_drops.hex,
                WindscreenDropVars::START,
            )],
            f,
        )
    }

    /// The `N` floats at the test address `at`.
    fn floats<const N: usize>(at: usize) -> [PlainFloat; N] {
        std::array::from_fn(|i| {
            let word = unsafe {
                crate::pointers::host_at(at + 4 * i)
                    .cast::<u32>()
                    .read_unaligned()
            };
            PlainFloat::from_bits(word)
        })
    }

    /// The floats written back to the test address `at`.
    fn put_floats(at: usize, cells: &[PlainFloat]) {
        for (i, c) in cells.iter().enumerate() {
            unsafe {
                crate::pointers::host_at(at + 4 * i)
                    .cast::<u32>()
                    .write_unaligned(c.bits());
            }
        }
    }

    /// The 30 bytes at the test address `at` as a buffer.
    fn kept(at: usize) -> PlainBytes<30> {
        PlainBytes::new(std::array::from_fn(|i| unsafe {
            crate::pointers::host_at(at + i).read()
        }))
    }

    fn put_kept(at: usize, kept: &PlainBytes<30>) {
        for i in 0..30 {
            unsafe { crate::pointers::host_at(at + i).write(kept.get(i)) };
        }
    }

    #[test]
    fn view_moved_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xA0,
                    hi: 0xC9402,
                    reads: &[
                        (0xA0, F32),
                        (0xB1296, F32),
                        (0xB129A, F32),
                        (0xB129E, F32),
                        (0xB12A2, F32),
                        (0xB12A6, F32),
                        (0xB12AA, F32),
                        (0xB12AE, F32),
                        (0xB12B2, F32),
                        (0xB12B6, F32),
                        (0xB12BA, F32),
                        (0xB12BE, F32),
                        (0xB12C2, F32),
                        (0xC93E6, F32),
                        (0xC93EA, F32),
                        (0xC93EE, F32),
                    ],
                    writes: &[
                        (0xB12AE, F32),
                        (0xB12B2, F32),
                        (0xB12B6, F32),
                        (0xB12BA, F32),
                        (0xB12BE, F32),
                        (0xB12C2, F32),
                    ],
                },
                Op::Base {
                    lo: 0,
                    hi: 0x2768,
                    reads: &[(0x26AC, F32), (0x26B0, F32), (0x26B4, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes(
            "effects.windscreen_drops_drift.view_moved_x87",
            &SPEC,
            |b| {
                hex(b[0], || {
                    with_settings(b[1], |m| {
                        super::view_moved_x87(
                            &crate::effects::owner::effects().windscreen_drops,
                            m,
                            crate::effects::setup::clock_at(b[0]),
                        );
                    });
                });
            },
        );
    }

    #[test]
    fn drift_turned_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xB1252,
                    hi: 0xC9402,
                    reads: &[(0xB12BA, F32), (0xB12BE, F32), (0xB12C2, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x1E,
                    reads: &[],
                    writes: &[(0x0, F80), (0xA, F80), (0x14, F80)],
                },
            ],
            sincos: false,
        };
        recorded_writes(
            "effects.windscreen_drops_drift.drift_turned_x87",
            &SPEC,
            |b| {
                hex(b[0], || {
                    let (m, temps, l) = (floats::<9>(b[1]), floats::<2>(b[2]), kept(b[3]));
                    super::drift_turned_x87(
                        &crate::effects::owner::effects().windscreen_drops,
                        &m,
                        &temps,
                        &l,
                    );
                    put_floats(b[2], &temps);
                    put_kept(b[3], &l);
                });
            },
        );
    }

    #[test]
    fn drift_back_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xACC83,
                    hi: 0xC9402,
                    reads: &[(0xC93F6, F32), (0xC93FA, F32), (0xC93FE, F32)],
                    writes: &[
                        (0xACC9F, F32),
                        (0xACCA3, F32),
                        (0xACCA7, F32),
                        (0xB12DE, F32),
                        (0xB12E2, F32),
                        (0xB12E6, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x24,
                    reads: &[
                        (0x0, F32),
                        (0x4, F32),
                        (0x8, F32),
                        (0xC, F32),
                        (0x10, F32),
                        (0x14, F32),
                        (0x18, F32),
                        (0x1C, F32),
                        (0x20, F32),
                    ],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0x8,
                    reads: &[(0x0, F32), (0x4, F32)],
                    writes: &[(0x0, F32), (0x4, F32)],
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
        recorded_writes(
            "effects.windscreen_drops_drift.drift_back_x87",
            &SPEC,
            |b| {
                with_overlays(
                    b[0],
                    &[
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.tilt,
                            ShapeTilt::START,
                        ),
                        overlay(
                            &crate::effects::owner::effects().windscreen_drops.hex,
                            WindscreenDropVars::START,
                        ),
                    ],
                    || {
                        let (m, temps, l) = (floats::<9>(b[1]), floats::<2>(b[2]), kept(b[3]));
                        super::drift_back_x87(
                            &crate::effects::owner::effects().windscreen_drops,
                            &m,
                            &temps,
                            &l,
                        );
                        put_floats(b[2], &temps);
                    },
                );
            },
        );
    }

    #[test]
    fn move_kept_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xA0,
                    hi: 0xC9402,
                    reads: &[(0xA0, F32), (0xB1296, F32), (0xB129A, F32), (0xB129E, F32)],
                    writes: &[(0xB12A2, F32), (0xB12A6, F32), (0xB12AA, F32)],
                },
                Op::Base {
                    lo: 0,
                    hi: 0x2768,
                    reads: &[(0x26AC, F32), (0x26B0, F32), (0x26B4, F32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        recorded_writes("effects.windscreen_drops_drift.move_kept_x87", &SPEC, |b| {
            hex(b[0], || {
                with_settings(b[1], |m| {
                    super::move_kept_x87(
                        &crate::effects::owner::effects().windscreen_drops,
                        m,
                        crate::effects::setup::clock_at(b[0]),
                    );
                });
            });
        });
    }
}
