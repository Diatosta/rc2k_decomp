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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_view::{rd, record};
use crate::physics::flow::{g, hold_high, hold_low, p, x87_run};

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn camera_height_bias() {
    let esi = record();
    if rd(esi + 0x54) == 0 {
        return;
    }
    x87_run!(
        "fld dword ptr [{rc} + 0x20631C]",
        "fdiv dword ptr [{rc} + 0x206290]",
        "fstp dword ptr [{rc} + 0x22400C]";
        rc = crate::race::store::asm_base()
    );
    hold_low(0x62400C, 0x5C5D90);
    hold_high(0x62400C, 0x5C5D38);
    if (rd(esi + 0x54) as i32) < 0 {
        x87_run!(
            ".byte 0xD9, 0xE8", // fld1
            "fsub dword ptr [{rc} + 0x22400C]",
            "fstp dword ptr [{rc} + 0x22400C]";
            rc = crate::race::store::asm_base()
        );
    }
    x87_run!(
        "fld dword ptr [{rc} + 0x22400C]",
        "fmul dword ptr [{rc} + 0x1C5D98]",
        "fmul dword ptr [{rc} + 0x20628C]",
        "fistp dword ptr [{rc} + 0x22400C]";
        rc = crate::race::store::asm_base()
    );
    let eax = g(0x62400C);
    p(0x606288, eax);
    p(0x71A680, g(0x71A680).wrapping_add(eax));
}
