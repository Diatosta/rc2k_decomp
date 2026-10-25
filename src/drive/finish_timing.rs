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

use crate::math::x87::loaded;
use crate::physics::flow::{c0, g, p};
use crate::race::state_store::RACE_STATE;
use crate::scene::draw_store::DRAW;
use crate::scene::x87_mem::{fild, fld, fstp};

const T: usize = 0x64BB64;

/// `flt_64BB64` = the clock less the start (`dword_62400C`) over `flt_5C5D18`, over the race
/// block at `rc`.
fn time(rc: usize) {
    let rc = rc as *mut u8;
    unsafe { fstp(rc, 0x24BB64, fild(rc, 0x22400C) / fld(rc, 0x1C5D18)) }
}

/// The eased time between the two limits: t less ((t - a) / (b - a))^2 x ((b - a) x half).
fn eased_middle(rc: usize) {
    let rc = rc as *mut u8;
    unsafe {
        let q = (fld(rc, 0x24BB64) - fld(rc, 0x24BB44)) / (fld(rc, 0x24BB48) - fld(rc, 0x24BB44));
        let square = q * q;
        let span = (fld(rc, 0x24BB48) - fld(rc, 0x24BB44)) * fld(rc, 0x1C5E40);
        let product = square * span;
        fstp(rc, 0x24BB68, -product + fld(rc, 0x24BB64));
    }
}

/// The eased time past the second limit: a + (b - a) x half.
fn eased_end(rc: usize) {
    let rc = rc as *mut u8;
    unsafe {
        let span = (fld(rc, 0x24BB48) - fld(rc, 0x24BB44)) * fld(rc, 0x1C5E40);
        fstp(rc, 0x24BB68, span + fld(rc, 0x24BB44));
    }
}

/// The share between the last two limits: (t - c) / (d - c).
fn share(rc: usize) {
    let rc = rc as *mut u8;
    unsafe {
        let q = (fld(rc, 0x24BB64) - fld(rc, 0x24BB70)) / (fld(rc, 0x24BB74) - fld(rc, 0x24BB70));
        fstp(rc, 0x24BB6C, q);
    }
}

/// The core from the clock's reading `now` (already in `g_sceneClock`).
pub fn timing_from(now: u32) {
    let rc = crate::race::store::asm_base() as usize;
    let since = now.wrapping_sub(g(0x64BB60));
    DRAW.scratch.set(since);
    time(rc);
    if c0(g(T), g(0x64BB44)) {
        p(0x64BB68, loaded(g(T)));
    } else if c0(g(T), g(0x64BB48)) {
        eased_middle(rc);
    } else {
        eased_end(rc);
    }
    p(0x64BB6C, 0);
    let late = !c0(g(T), g(0x64BB70));
    if RACE_STATE.results.finished.get() != 0 && late {
        p(0x64BB6C, 0x3F80_0000);
        if c0(g(T), g(0x64BB74)) {
            share(rc);
        }
    }
    if late && RACE_STATE.results.finished.get() != 0 {
        p(0x5C2F88, 1);
    }
}

/// The Rust version (the exact one, for `drive::finish_run`).
pub unsafe fn finish_timing() {
    let now = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    timing_from(now)
}

#[cfg(test)]
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
                operands: vec![Operand::Buffer {
                    reads: &[(0x22400C, In::I), (0x1C5D18, In::F)],
                    writes: &[(0x24BB64, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_time(a[0]) },
                plain: |a| time(a[0]),
            },
            Block {
                name: "finish_timing middle",
                operands: vec![Operand::Buffer {
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
                plain: |a| eased_middle(a[0]),
            },
            Block {
                name: "finish_timing end",
                operands: vec![Operand::Buffer {
                    reads: &[(0x24BB48, In::F), (0x24BB44, In::F), (0x1C5E40, In::F)],
                    writes: &[(0x24BB68, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_end(a[0]) },
                plain: |a| eased_end(a[0]),
            },
            Block {
                name: "finish_timing share",
                operands: vec![Operand::Buffer {
                    reads: &[(0x24BB64, In::F), (0x24BB70, In::F), (0x24BB74, In::F)],
                    writes: &[(0x24BB6C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_share(a[0]) },
                plain: |a| share(a[0]),
            },
        ]
    }

    #[test]
    fn finish_timing_matches_the_assembly() {
        sparse(&blocks(), 3000, 202);
    }
}
