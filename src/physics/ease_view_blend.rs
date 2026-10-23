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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, ESI, Regs};
use crate::physics::camera_view::{rd, record, share_done};
use crate::physics::flow::{base, x87_run};
use crate::screens::harness::Left;

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn ease_view_blend(_: &Regs) -> Left {
    let esi = record();
    let left = Left::nothing().with(ESI, esi);
    if rd(esi + 0x58) == 0 || rd(esi + 0x3C) == 0 {
        return left;
    }
    let eax = share_done(esi);
    if (rd(esi + 0x58) as i32) < 0 {
        x87_run!(
            ".byte 0xD9, 0xE8", // fld1
            "fsub dword ptr [{b} + 0x205260]",
            "fstp dword ptr [{b} + 0x205260]";
            b = base()
        );
    }
    x87_run!(
        "fld dword ptr [{b} + 0x208B24]",
        "fsub dword ptr [{b} + 0x208B20]",
        "fmul dword ptr [{b} + 0x205260]",
        "fadd dword ptr [{b} + 0x208B20]",
        "fstp dword ptr [{b} + 0x31A6A0]",
        "fld dword ptr [{b} + 0x208B2C]",
        "fsub dword ptr [{b} + 0x208B28]",
        "fmul dword ptr [{b} + 0x205260]",
        "fadd dword ptr [{b} + 0x208B28]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        ".byte 0xD9, 0xE0", // fchs
        "fistp dword ptr [{b} + 0x208B38]";
        b = base()
    );
    left.with(EAX, eax)
}
