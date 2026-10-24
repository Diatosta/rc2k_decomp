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
//! Caller: `drive::finish_run`. It begins with a call: hooked by a private copy of its
//! 0x144 bytes (the call relocated) and a hand-made patch.
//! Shadow mode runs the original alone (the clock moves). Self-test: a copy whose clock
//! call goes to a stand-in giving the test's times, against the Rust version with the
//! same, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::math::x87::loaded;
use crate::physics::flow::{c0, g, p, x87_run};

const T: usize = 0x64BB64;

/// The core from the clock's reading `now` (already in `g_sceneClock`).
pub fn timing_from(now: u32) {
    let since = now.wrapping_sub(g(0x64BB60));
    p(0x62400C, since);
    x87_run!(
        "fild dword ptr [{rc} + 0x22400C]",
        "fdiv dword ptr [{rc} + 0x1C5D18]",
        "fstp dword ptr [{rc} + 0x24BB64]";
        rc = crate::race::store::asm_base()
    );
    if c0(g(T), g(0x64BB44)) {
        p(0x64BB68, loaded(g(T)));
    } else if c0(g(T), g(0x64BB48)) {
        x87_run!(
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
            "fstp dword ptr [{rc} + 0x24BB68]";
            rc = crate::race::store::asm_base()
        );
    } else {
        x87_run!(
            "fld dword ptr [{rc} + 0x24BB48]",
            "fsub dword ptr [{rc} + 0x24BB44]",
            "fmul dword ptr [{rc} + 0x1C5E40]",
            "fadd dword ptr [{rc} + 0x24BB44]",
            "fstp dword ptr [{rc} + 0x24BB68]";
            rc = crate::race::store::asm_base()
        );
    }
    p(0x64BB6C, 0);
    let late = !c0(g(T), g(0x64BB70));
    if g(0x75B108) != 0 && late {
        p(0x64BB6C, 0x3F80_0000);
        if c0(g(T), g(0x64BB74)) {
            x87_run!(
                "fld dword ptr [{rc} + 0x24BB64]",
                "fsub dword ptr [{rc} + 0x24BB70]",
                "fld dword ptr [{rc} + 0x24BB74]",
                "fsub dword ptr [{rc} + 0x24BB70]",
                ".byte 0xDE, 0xF9", // fdivp st(1), st
                "fstp dword ptr [{rc} + 0x24BB6C]";
                rc = crate::race::store::asm_base()
            );
        }
    }
    if late && g(0x75B108) != 0 {
        p(0x5C2F88, 1);
    }
}

/// The Rust version (the exact one, for `drive::finish_run`).
pub unsafe fn finish_timing() {
    let now = crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
    timing_from(now)
}
