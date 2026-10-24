//! `physics::ease_view_angles` (0x43846D): with the mode's record's change time (+0x3C)
//! not 0: the share done (`physics/camera_view.rs`) times the record's three angle
//! changes (+0x40 .. +0x48) x `flt_5C5D98` (`fistp`) << 16 added to `unk_71A694`,
//! `unk_71A698`, `unk_71A69C`.
//!
//! In:  nothing
//! Out: esi the mode's record; eax the last change added (or the time held, kept when
//!      the record has no time); the rest kept; no flags read
//!
//! Caller: `sub_437A53`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::camera_view::{rd, record, share_done};
use crate::physics::flow::{g, p, x87_run};

/// The Rust version (the exact one, for `physics::update_camera`).
pub unsafe fn ease_view_angles() {
    let esi = record();
    if rd(esi + 0x3C) == 0 {
        return;
    }
    share_done(esi);
    for (k, to) in [0x71A694usize, 0x71A698, 0x71A69C].iter().enumerate() {
        x87_run!(
            "fld dword ptr [{s}]",
            "fmul dword ptr [{rc} + 0x205260]",
            "fmul dword ptr [{rc} + 0x1C5D98]",
            "fistp dword ptr [{rc} + 0x22400C]";
            s = esi + 0x40 + 4 * k as u32, rc = crate::race::store::asm_base()
        );
        p(*to, g(*to).wrapping_add(g(0x62400C) << 16));
    }
}
