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

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::{base, g, p, x87_run};
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::drive::finish_camera::{FinishLines, finish_camera as finish_camera_core};

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

/// The converted version (`rc2k_core::drive::finish_camera`, plain floats): the hook's.
// Converted: `rc2k_core::drive::finish_camera::finish_camera`.
pub fn finish_camera_plain() {
    let f = |x: usize| f32::from_bits(g(x));
    let [a748, a730, a734, a744] = finish_camera_core(&FinishLines {
        k4c: f(0x64BB4C),
        k50: f(0x64BB50),
        k54: f(0x64BB54),
        k58: f(0x64BB58),
        k5c: f(0x64BB5C),
        time: f(0x64BB68),
        share: f(0x64BB6C),
        k78: f(0x64BB78),
        k7c: f(0x64BB7C),
    });
    p(0x606748, a748.to_bits());
    p(0x606730, a730.to_bits());
    p(0x606734, a734.to_bits());
    p(0x60672C, 0);
    p(0x606744, a744.to_bits());
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    finish_camera_plain();
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
