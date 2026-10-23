//! `physics::camera_height_bias` (0x43852D): with the mode's record's +0x54 not 0:
//! `flt_60631C` / `flt_606290` held to 0 ..= 1 (1 less it for +0x54 below 0), x
//! `flt_5C5D98` x `flt_60628C` (`fistp`) into `dword_606288` and added to the camera's
//! height `unk_71A680`.
//!
//! In:  nothing
//! Out: esi the mode's record; eax the height added when it ran; the rest kept; no
//!      flags read
//!
//! Caller: `sub_437A53`.
//! Self-test: `physics/camera_view.rs` (`view_test_floats`), modes and figures by lot,
//! exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, ESI, Regs};
use crate::physics::camera_view::{rd, record};
use crate::physics::flow::{base, g, hold_high, hold_low, p, x87_run};
use crate::screens::harness::Left;

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn camera_height_bias(_: &Regs) -> Left {
    let esi = record();
    let left = Left::nothing().with(ESI, esi);
    if rd(esi + 0x54) == 0 {
        return left;
    }
    x87_run!(
        "fld dword ptr [{b} + 0x20631C]",
        "fdiv dword ptr [{b} + 0x206290]",
        "fstp dword ptr [{b} + 0x22400C]";
        b = base()
    );
    hold_low(0x62400C, 0x5C5D90);
    hold_high(0x62400C, 0x5C5D38);
    if (rd(esi + 0x54) as i32) < 0 {
        x87_run!(
            ".byte 0xD9, 0xE8", // fld1
            "fsub dword ptr [{b} + 0x22400C]",
            "fstp dword ptr [{b} + 0x22400C]";
            b = base()
        );
    }
    x87_run!(
        "fld dword ptr [{b} + 0x22400C]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fmul dword ptr [{b} + 0x20628C]",
        "fistp dword ptr [{b} + 0x22400C]";
        b = base()
    );
    let eax = g(0x62400C);
    p(0x606288, eax);
    p(0x71A680, g(0x71A680).wrapping_add(eax));
    left.with(EAX, eax)
}
