//! `drive::finish_timing` (0x45948A): the finish routine's time and shares. The clock
//! (`scene::restart_clock`) less the start `dword_64BB60` / `flt_5C5D18` into
//! `flt_64BB64` (t); the eased time `flt_64BB68`: t below `flt_64BB44` (or unordered) t;
//! below `flt_64BB48` t less ((t - a) / (b - a))^2 x (b - a) / 2; else a + (b - a) / 2;
//! the share `flt_64BB6C`: 0, with `dword_75B108` and t from `flt_64BB70` on 1, below
//! `flt_64BB74` (t - c) / (d - c); and with `dword_75B108` and t from `flt_64BB70` on,
//! `dword_5C2F88` 1.
//!
//! In:  nothing
//! Out: eax the clock less the start; the rest kept; no flags read
//!
//! Caller: `drive::finish_run`.

use crate::drive::finish_store::FINISH;
use crate::math::x87::loaded;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::flow::c0;
use crate::race::numbers_store::NUMBERS;
use crate::race::state_store::RACE_STATE;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::scene::draw_store::DRAW;

/// `flt_64BB64` = the clock less the start (`dword_62400C`) over `flt_5C5D18`.
fn time() {
    FINISH.time.set_bits(st(
        int(DRAW.scratch.bits() as i32) / ld(NUMBERS.thousand.bits())
    ));
}

/// The eased time between the two limits: t less ((t - a) / (b - a))^2 x ((b - a) x half).
fn eased_middle() {
    let q = (ld(FINISH.time.bits()) - ld(FINISH.limit_a.bits()))
        / (ld(FINISH.limit_b.bits()) - ld(FINISH.limit_a.bits()));
    let square = q * q;
    let span = (ld(FINISH.limit_b.bits()) - ld(FINISH.limit_a.bits())) * ld(NUMBERS.half.bits());
    let product = square * span;
    FINISH.eased.set_bits(st(-product + ld(FINISH.time.bits())));
}

/// The eased time past the second limit: a + (b - a) x half.
fn eased_end() {
    let span = (ld(FINISH.limit_b.bits()) - ld(FINISH.limit_a.bits())) * ld(NUMBERS.half.bits());
    FINISH.eased.set_bits(st(span + ld(FINISH.limit_a.bits())));
}

/// The share between the last two limits: (t - c) / (d - c).
fn share() {
    let q = (ld(FINISH.time.bits()) - ld(FINISH.limit_c.bits()))
        / (ld(FINISH.limit_d.bits()) - ld(FINISH.limit_c.bits()));
    FINISH.share.set_bits(st(q));
}

/// The core from the clock's reading `now` (already in `g_sceneClock`).
pub fn timing_from(now: u32) {
    let since = now.wrapping_sub(FINISH.start_clock.get());
    DRAW.scratch.set(since);
    time();
    if c0(FINISH.time.bits(), FINISH.limit_a.bits()) {
        FINISH.eased.set_bits(loaded(FINISH.time.bits()));
    } else if c0(FINISH.time.bits(), FINISH.limit_b.bits()) {
        eased_middle();
    } else {
        eased_end();
    }
    FINISH.share.set_bits(0);
    let late = !c0(FINISH.time.bits(), FINISH.limit_c.bits());
    if RACE_STATE.results.finished.get() != 0 && late {
        FINISH.share.set_bits(0x3F80_0000);
        if c0(FINISH.time.bits(), FINISH.limit_d.bits()) {
            share();
        }
    }
    if late && RACE_STATE.results.finished.get() != 0 {
        VIEWER_PANEL.race_done.set(1);
    }
}

/// The Rust version (the exact one, for `drive::finish_run`).
pub fn finish_timing() {
    let now = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    timing_from(now);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_time(rc)
        "fild dword ptr [{rc} + 0x22400C]",
        "fdiv dword ptr [{rc} + 0x1C5D18]",
        "fstp dword ptr [{rc} + 0x24BB64]");
    reference!(ref_middle(rc)
        "fld dword ptr [{rc} + 0x24BB64]",
        "fsub dword ptr [{rc} + 0x24BB44]",
        "fld dword ptr [{rc} + 0x24BB48]",
        "fsub dword ptr [{rc} + 0x24BB44]",
        ".byte 0xDE, 0xF9", // fdivp st(1), st
        ".byte 0xD8, 0xC8", // fmul st, st
        "fld dword ptr [{rc} + 0x24BB48]",
        "fsub dword ptr [{rc} + 0x24BB44]",
        "fmul dword ptr [{rc} + 0x1C5E40]",
        ".byte 0xDE, 0xC9", // fmulp st(1), st
        ".byte 0xD9, 0xE0", // fchs
        "fadd dword ptr [{rc} + 0x24BB64]",
        "fstp dword ptr [{rc} + 0x24BB68]");
    reference!(ref_end(rc)
        "fld dword ptr [{rc} + 0x24BB48]",
        "fsub dword ptr [{rc} + 0x24BB44]",
        "fmul dword ptr [{rc} + 0x1C5E40]",
        "fadd dword ptr [{rc} + 0x24BB44]",
        "fstp dword ptr [{rc} + 0x24BB68]");
    reference!(ref_share(rc)
        "fld dword ptr [{rc} + 0x24BB64]",
        "fsub dword ptr [{rc} + 0x24BB70]",
        "fld dword ptr [{rc} + 0x24BB74]",
        "fsub dword ptr [{rc} + 0x24BB70]",
        ".byte 0xDE, 0xF9", // fdivp st(1), st
        "fstp dword ptr [{rc} + 0x24BB6C]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "finish_timing time",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x22400C, In::I), (0x1C5D18, In::F)],
                    writes: &[(0x24BB64, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_time(a[0]) },
                plain: |_| time(),
            },
            Block {
                name: "finish_timing middle",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x24BB64, In::F),
                        (0x24BB44, In::F),
                        (0x24BB48, In::F),
                        (0x1C5E40, In::F),
                    ],
                    writes: &[(0x24BB68, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_middle(a[0]) },
                plain: |_| eased_middle(),
            },
            Block {
                name: "finish_timing end",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x24BB48, In::F), (0x24BB44, In::F), (0x1C5E40, In::F)],
                    writes: &[(0x24BB68, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_end(a[0]) },
                plain: |_| eased_end(),
            },
            Block {
                name: "finish_timing share",
                operands: vec![Operand::RaceBlock {
                    reads: &[(0x24BB64, In::F), (0x24BB70, In::F), (0x24BB74, In::F)],
                    writes: &[(0x24BB6C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_share(a[0]) },
                plain: |_| share(),
            },
        ]
    }

    #[test]
    fn finish_timing_matches_the_assembly() {
        sparse(&blocks(), 3000, 202);
    }
}
