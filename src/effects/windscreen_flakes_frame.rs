//! `effects::windscreen_flakes_frame` (0x4BF60F): the windscreen flakes' frame (as `effects::draw_plane`'s, with
//! the rate times `flt_8BC19E` and the lengths over `flt_8BC1A2` / `flt_8BC1A6`): the
//! rate, the place and two axes from the settings, made one long and square, scaled
//! (`effects::scale_axes`); nothing more unless the place's way faces the third axis away;
//! then `effects::run_pong`; unless the rate times `flt_8BC1AA` comes to 0 (`fistp`, as the
//! x87 is set) `effects::run_windscreen_flakes`, and with the settings' +0x2744 and `dword_7652A0`
//! `effects::windscreen_flakes_step`.
//!
//! Screen: the snow settling on the windscreen in the in-car view (owner, 2026-10-29: gone with `draw_windscreen_flake` cut). Called the squares until then.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4C74DA` (0x4C7534). The x87 runs are plain Rust on the x87 model (`math::x87_float`), in the original's order, over the
//! game's variables; the vector helpers, `scale_axes` and `windscreen_flakes_step` their Rust
//! versions. Refuses null settings and `windscreen_flakes_step`'s refusals.

use crate::effects::tuning::MARKS_KIND_FLOATS;
use crate::effects::windscreen_drop_state::WINDSCREEN_DROP_STATE;
use crate::effects::windscreen_drop_vars::HEX;
use crate::effects::windscreen_flakes_step::windscreen_flakes_step;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::pointers::GamePtr;
use crate::scene::cross_product3::cross;
use crate::scene::vector3::normalise;

/// The original's 0x4BF615 .. 0x4BF6AB: the rate (`flt_89CA34`), the place and the first axis from the settings, its length over `flt_8BC1A2` into `flt_89CEEE`.
fn frame_axes(s: u32) {
    frame_axes_x87(
        s as usize,
        &WINDSCREEN_FLAKE_TIMING,
        &WINDSCREEN_FLAKE_TABLES,
        &WINDSCREEN_FLAKE_STATE,
    )
}

/// The original's 0x4BF6B8 .. 0x4BF718: the second axis, its length over `flt_8BC1A6` into `flt_89CEF2`.
fn frame_up(s: u32) {
    frame_up_x87(
        s as usize,
        &WINDSCREEN_FLAKE_TABLES,
        &WINDSCREEN_FLAKE_STATE,
    )
}

/// The original's 0x4BF769 .. 0x4BF78D: the place as floats into `flt_8161A2` ...
fn frame_place(s: u32) {
    frame_place_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        s as usize,
    )
}

/// The original's 0x4BF7A4 .. 0x4BF7BE: the place's way dotted with the third axis into `flt_814718`.
fn frame_facing() {
    frame_facing_x87(
        crate::effects::windscreen_drop_vars::WindscreenDropVars::base(),
        &WINDSCREEN_FLAKE_TABLES,
    )
}

/// The Rust version (see the header).
pub fn windscreen_flakes_frame() -> Result<(), Refused> {
    let s: u32 = crate::effects::setup::SETTINGS.pointer.get();
    if s == 0 {
        return Err(BadInput("no marks settings").into());
    }
    frame_axes(s);
    let [a0, a1, a2] = [0, 1, 2].map(|n| &WINDSCREEN_FLAKE_TABLES.axes[n]);
    normalise(a0);
    frame_up(s);
    normalise(a1);
    cross(a1, a0, a2);
    normalise(a2);
    cross(a0, a2, a1);
    crate::effects::scale_axes::scale_axes();
    frame_place(s);
    normalise(&HEX.place);
    frame_facing();
    if WINDSCREEN_DROP_STATE.facing.bits() & 0x8000_0000 == 0 {
        return Ok(());
    }
    crate::effects::run_pong::run_pong()?;
    if x87::mul_to_int(
        WINDSCREEN_FLAKE_TIMING.rate.bits(),
        WINDSCREEN_FLAKE_STATE.share.bits(),
    ) == 0
    {
        return Ok(());
    }
    crate::effects::run_windscreen_flakes::run_windscreen_flakes()?;
    let s: u32 = crate::effects::setup::SETTINGS.pointer.get();
    if GamePtr::new(s + 0x2744).read::<u32>("a mark's field")? == 0 {
        return Ok(());
    }
    if MARKS_KIND_FLOATS.running.bits() == 0 {
        return Ok(());
    }
    windscreen_flakes_step()?;
    Ok(())
}

use crate::effects::plain::{ld, ldi, st};
use crate::effects::windscreen_flakes_store::{
    WINDSCREEN_FLAKE_STATE, WINDSCREEN_FLAKE_TABLES, WINDSCREEN_FLAKE_TIMING, WindscreenFlakeState,
    WindscreenFlakeTables, WindscreenFlakeTiming,
};

/// `frame_axes`'s x87 block as plain Rust on the x87 model (29 instructions), same order and memory places.
fn frame_axes_x87(
    s: usize,
    sti: &WindscreenFlakeTiming,
    sq: &WindscreenFlakeTables,
    ss: &WindscreenFlakeState,
) {
    let t0 = ldi(s, 0x2740);
    let t1 = t0 * bh::ld(ss.rate_share.bits());
    sti.rate.set_bits(bh::st(t1));
    let t2 = ldi(s, 0x2720);
    let t3 = ldi(s, 0x2724);
    let t4 = ldi(s, 0x2728);
    sq.place[2].set_bits(bh::st(t4));
    sq.place[1].set_bits(bh::st(t3));
    sq.place[0].set_bits(bh::st(t2));
    let t5 = ldi(s, 0x272C);
    let t6 = t5 - ldi(s, 0x2720);
    sq.axes[0][0].set_bits(bh::st(t6));
    let t7 = ldi(s, 0x2730);
    let t8 = t7 - ldi(s, 0x2724);
    sq.axes[0][1].set_bits(bh::st(t8));
    let t9 = ldi(s, 0x2734);
    let t10 = t9 - ldi(s, 0x2728);
    sq.axes[0][2].set_bits(bh::st(t10));
    let t11 = bh::ld(sq.axes[0][0].bits());
    let t12 = t11 * t11;
    let t13 = bh::ld(sq.axes[0][1].bits());
    let t14 = t13 * t13;
    let t15 = bh::ld(sq.axes[0][2].bits());
    let t16 = t15 * t15;
    let t17 = t14 + t16;
    let t18 = t12 + t17;
    let t19 = t18.sqrt();
    let t20 = t19 / bh::ld(ss.length_divisors[0].bits());
    sq.lengths[0].set_bits(bh::st(t20));
}

/// `frame_up`'s x87 block as plain Rust on the x87 model (20 instructions), same order and memory places.
fn frame_up_x87(s: usize, sq: &WindscreenFlakeTables, ss: &WindscreenFlakeState) {
    let t0 = ldi(s, 0x2708);
    let t1 = t0 - ldi(s, 0x2720);
    sq.axes[1][0].set_bits(bh::st(t1));
    let t2 = ldi(s, 0x270C);
    let t3 = t2 - ldi(s, 0x2724);
    sq.axes[1][1].set_bits(bh::st(t3));
    let t4 = ldi(s, 0x2710);
    let t5 = t4 - ldi(s, 0x2728);
    sq.axes[1][2].set_bits(bh::st(t5));
    let t6 = bh::ld(sq.axes[1][0].bits());
    let t7 = t6 * t6;
    let t8 = bh::ld(sq.axes[1][1].bits());
    let t9 = t8 * t8;
    let t10 = bh::ld(sq.axes[1][2].bits());
    let t11 = t10 * t10;
    let t12 = t9 + t11;
    let t13 = t7 + t12;
    let t14 = t13.sqrt();
    let t15 = t14 / bh::ld(ss.length_divisors[1].bits());
    sq.lengths[1].set_bits(bh::st(t15));
}

/// `frame_place`'s x87 block as plain Rust on the x87 model (6 instructions), same order and memory places.
fn frame_place_x87(hx: usize, s: usize) {
    let t0 = ldi(s, 0x2720);
    st(hx, 0xB1262, t0);
    let t1 = ldi(s, 0x2724);
    st(hx, 0xB1266, t1);
    let t2 = ldi(s, 0x2728);
    st(hx, 0xB126A, t2);
}

/// `frame_facing`'s x87 block as plain Rust on the x87 model (9 instructions), same order and memory places.
fn frame_facing_x87(hx: usize, sq: &WindscreenFlakeTables) {
    let t0 = ld(hx, 0xB1262);
    let t1 = t0 * bh::ld(sq.axes[2][0].bits());
    let t2 = ld(hx, 0xB1266);
    let t3 = t2 * bh::ld(sq.axes[2][1].bits());
    let t4 = ld(hx, 0xB126A);
    let t5 = t4 * bh::ld(sq.axes[2][2].bits());
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    WINDSCREEN_DROP_STATE.facing.set_bits(bh::st(t7));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) fn frame_axes_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{s} + 0x2740]",
            "fmul dword ptr [{mk} + 0x15725E]",
            "fstp dword ptr [{mk} + 0x137AF4]",
            "fild dword ptr [{s} + 0x2720]",
            "fild dword ptr [{s} + 0x2724]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FBE]",
            "fstp dword ptr [{mk} + 0x137FBA]",
            "fstp dword ptr [{mk} + 0x137FB6]",
            "fild dword ptr [{s} + 0x272C]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0x137FDA]",
            "fild dword ptr [{s} + 0x2730]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0x137FDE]",
            "fild dword ptr [{s} + 0x2734]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FE2]",
            "fld dword ptr [{mk} + 0x137FDA]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FDE]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FE2]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{mk} + 0x157262]",
            "fstp dword ptr [{mk} + 0x137FAE]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) fn frame_up_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{s} + 0x2708]",
            "fisub dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0x137FE6]",
            "fild dword ptr [{s} + 0x270C]",
            "fisub dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0x137FEA]",
            "fild dword ptr [{s} + 0x2710]",
            "fisub dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0x137FEE]",
            "fld dword ptr [{mk} + 0x137FE6]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FEA]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x137FEE]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdiv dword ptr [{mk} + 0x157266]",
            "fstp dword ptr [{mk} + 0x137FB2]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) fn frame_place_x87(mk: usize, s: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{s} + 0x2720]",
            "fstp dword ptr [{mk} + 0xB1262]",
            "fild dword ptr [{s} + 0x2724]",
            "fstp dword ptr [{mk} + 0xB1266]",
            "fild dword ptr [{s} + 0x2728]",
            "fstp dword ptr [{mk} + 0xB126A]",
            mk = in(reg) mk,
            s = in(reg) s,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) fn frame_facing_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0xB1262]",
            "fmul dword ptr [{mk} + 0x137FF2]",
            "fld dword ptr [{mk} + 0xB1266]",
            "fmul dword ptr [{mk} + 0x137FF6]",
            "fld dword ptr [{mk} + 0xB126A]",
            "fmul dword ptr [{mk} + 0x137FFA]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fstp dword ptr [{mk} + 0xAF7D8]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::effects::asm_test::Kind::*;
    use crate::effects::asm_test::with_copy;
    use crate::effects::asm_test::{Op, Spec, differential};
    use crate::effects::typed_sync::{overlay, with_overlays};
    use crate::effects::windscreen_drop_state::{WINDSCREEN_DROP_STATE, WindscreenDropState};
    use crate::effects::windscreen_flakes_store::{
        WindscreenFlakeState, WindscreenFlakeTables, WindscreenFlakeTiming,
    };

    #[test]
    fn frame_axes_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137AF0,
                    hi: 0x157326,
                    reads: &[
                        (0x137FDA, F32),
                        (0x137FDE, F32),
                        (0x137FE2, F32),
                        (0x15725E, F32),
                        (0x157262, F32),
                    ],
                    writes: &[
                        (0x137AF4, F32),
                        (0x137FAE, F32),
                        (0x137FB6, F32),
                        (0x137FBA, F32),
                        (0x137FBE, F32),
                        (0x137FDA, F32),
                        (0x137FDE, F32),
                        (0x137FE2, F32),
                    ],
                },
                Op::Base {
                    lo: 0x2720,
                    hi: 0x2744,
                    reads: &[
                        (0x2720, I32),
                        (0x2724, I32),
                        (0x2728, I32),
                        (0x272C, I32),
                        (0x2730, I32),
                        (0x2734, I32),
                        (0x2740, I32),
                    ],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::frame_axes_x87(b[0], b[1]),
            |b| {
                with_copy::<WindscreenFlakeTiming, _>(b[0], WindscreenFlakeTiming::START, |sti| {
                    with_copy::<WindscreenFlakeTables, _>(
                        b[0],
                        WindscreenFlakeTables::START,
                        |sq| {
                            with_copy::<WindscreenFlakeState, _>(
                                b[0],
                                WindscreenFlakeState::START,
                                |ss| super::frame_axes_x87(b[1], sti, sq, ss),
                            )
                        },
                    )
                })
            },
        );
    }

    #[test]
    fn frame_up_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x137B8A,
                    hi: 0x157326,
                    reads: &[
                        (0x137FE6, F32),
                        (0x137FEA, F32),
                        (0x137FEE, F32),
                        (0x157266, F32),
                    ],
                    writes: &[
                        (0x137FB2, F32),
                        (0x137FE6, F32),
                        (0x137FEA, F32),
                        (0x137FEE, F32),
                    ],
                },
                Op::Base {
                    lo: 0x2708,
                    hi: 0x272C,
                    reads: &[
                        (0x2708, I32),
                        (0x270C, I32),
                        (0x2710, I32),
                        (0x2720, I32),
                        (0x2724, I32),
                        (0x2728, I32),
                    ],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::frame_up_x87(b[0], b[1]),
            |b| {
                with_copy::<WindscreenFlakeTables, _>(b[0], WindscreenFlakeTables::START, |sq| {
                    with_copy::<WindscreenFlakeState, _>(b[0], WindscreenFlakeState::START, |ss| {
                        super::frame_up_x87(b[1], sq, ss)
                    })
                })
            },
        );
    }

    #[test]
    fn frame_place_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0xB1262,
                    hi: 0xB126E,
                    reads: &[],
                    writes: &[(0xB1262, F32), (0xB1266, F32), (0xB126A, F32)],
                },
                Op::Base {
                    lo: 0x2720,
                    hi: 0x272C,
                    reads: &[(0x2720, I32), (0x2724, I32), (0x2728, I32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::frame_place_x87(b[0], b[1]),
            |b| super::frame_place_x87(b[0], b[1]),
        );
    }

    #[test]
    fn frame_facing_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0xAF7D0,
                hi: 0x13822A,
                reads: &[
                    (0xB1262, F32),
                    (0xB1266, F32),
                    (0xB126A, F32),
                    (0x137FF2, F32),
                    (0x137FF6, F32),
                    (0x137FFA, F32),
                ],
                writes: &[(0xAF7D8, F32)],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| reference::frame_facing_x87(b[0]),
            |b| {
                with_overlays(
                    b[0],
                    &[overlay(&WINDSCREEN_DROP_STATE, WindscreenDropState::START)],
                    || {
                        with_copy::<WindscreenFlakeTables, _>(
                            b[0],
                            WindscreenFlakeTables::START,
                            |sq| super::frame_facing_x87(b[0], sq),
                        )
                    },
                )
            },
        );
    }
}
