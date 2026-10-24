//! `physics::ease_view_blend` (0x4385EB): with the mode's record's blend (+0x58) and
//! change time (+0x3C) not 0: the share done (`physics/camera_view.rs`; 1 less it for a
//! blend below 0) between `flt_608B20` and `flt_608B24` into `unk_71A6A0`, and between
//! `flt_608B28` and `flt_608B2C`, x `flt_5C5D98`, negated, `fistp`ed into
//! `dword_608B38`.
//!
//! In:  nothing
//! Out: esi the mode's record; eax the time held when it ran; the rest kept; no flags
//!      read
//!
//! Caller: `sub_437A53`.
//! Self-test: `physics/camera_view.rs` (`view_test_floats`), modes and times by lot,
//! exact at double precision.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_view::{rd, record, share_done};
use crate::physics::flow::x87_run;

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn ease_view_blend() {
    let esi = record();
    if rd(esi + 0x58) == 0 || rd(esi + 0x3C) == 0 {
        return;
    }
    share_done(esi);
    if (rd(esi + 0x58) as i32) < 0 {
        x87_run!(
            ".byte 0xD9, 0xE8", // fld1
            "fsub dword ptr [{rc} + 0x205260]",
            "fstp dword ptr [{rc} + 0x205260]";
            rc = crate::race::store::asm_base()
        );
    }
    x87_run!(
        "fld dword ptr [{rc} + 0x208B24]",
        "fsub dword ptr [{rc} + 0x208B20]",
        "fmul dword ptr [{rc} + 0x205260]",
        "fadd dword ptr [{rc} + 0x208B20]",
        "fstp dword ptr [{rc} + 0x31A6A0]",
        "fld dword ptr [{rc} + 0x208B2C]",
        "fsub dword ptr [{rc} + 0x208B28]",
        "fmul dword ptr [{rc} + 0x205260]",
        "fadd dword ptr [{rc} + 0x208B28]",
        "fmul dword ptr [{rc} + 0x1C5D98]",
        ".byte 0xD9, 0xE0", // fchs
        "fistp dword ptr [{rc} + 0x208B38]";
        rc = crate::race::store::asm_base()
    );
}
