//! `drive::finish_camera` (0x45941D): the finish routine's camera from the eased time
//! (`flt_64BB68`) and the share (`flt_64BB6C`): `dword_606748`, `606730`, `606734`,
//! `606744` (each a line in them, `flt_64BB4C` .. `flt_64BB7C`), `dword_60672C` 0.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::finish_run`.

use crate::physics::flow::{p, x87_run};

/// The Rust version (the exact one, for `drive::finish_run`).
pub fn finish_camera() {
    x87_run!(
        "fld dword ptr [{rc} + 0x24BB68]",
        "fmul dword ptr [{rc} + 0x24BB4C]",
        "fstp dword ptr [{rc} + 0x206748]",
        "fld dword ptr [{rc} + 0x24BB68]",
        "fmul dword ptr [{rc} + 0x24BB5C]",
        "fadd dword ptr [{rc} + 0x24BB54]",
        "fld dword ptr [{rc} + 0x24BB6C]",
        "fmul dword ptr [{rc} + 0x24BB7C]",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        "fstp dword ptr [{rc} + 0x206730]",
        "fld dword ptr [{rc} + 0x24BB68]",
        "fmul dword ptr [{rc} + 0x24BB58]",
        "fadd dword ptr [{rc} + 0x24BB50]",
        "fstp dword ptr [{rc} + 0x206734]";
        rc = crate::race::store::asm_base()
    );
    p(0x60672C, 0);
    x87_run!(
        "fld dword ptr [{rc} + 0x24BB6C]",
        "fmul dword ptr [{rc} + 0x24BB78]",
        "fstp dword ptr [{rc} + 0x206744]";
        rc = crate::race::store::asm_base()
    );
}
