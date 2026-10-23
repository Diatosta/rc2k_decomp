//! `drive::finish_camera` (0x45941D): the finish routine's camera from the eased time
//! (`flt_64BB68`) and the share (`flt_64BB6C`): `dword_606748`, `606730`, `606734`,
//! `606744` (each a line in them, `flt_64BB4C` .. `flt_64BB7C`), `dword_60672C` 0.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::finish_run`.
//! Self-test: the original and the converted version on the figures by lot, exact at
//! double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::physics::flow::{base, p, x87_run};

/// The Rust version (the exact one, for `drive::finish_run`).
pub fn finish_camera() {
    x87_run!(
        "fld dword ptr [{b} + 0x24BB68]",
        "fmul dword ptr [{b} + 0x24BB4C]",
        "fstp dword ptr [{b} + 0x206748]",
        "fld dword ptr [{b} + 0x24BB68]",
        "fmul dword ptr [{b} + 0x24BB5C]",
        "fadd dword ptr [{b} + 0x24BB54]",
        "fld dword ptr [{b} + 0x24BB6C]",
        "fmul dword ptr [{b} + 0x24BB7C]",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        "fstp dword ptr [{b} + 0x206730]",
        "fld dword ptr [{b} + 0x24BB68]",
        "fmul dword ptr [{b} + 0x24BB58]",
        "fadd dword ptr [{b} + 0x24BB50]",
        "fstp dword ptr [{b} + 0x206734]";
        b = base()
    );
    p(0x60672C, 0);
    x87_run!(
        "fld dword ptr [{b} + 0x24BB6C]",
        "fmul dword ptr [{b} + 0x24BB78]",
        "fstp dword ptr [{b} + 0x206744]";
        b = base()
    );
}
