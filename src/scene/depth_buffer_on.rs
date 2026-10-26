//! `scene::depth_buffer_on` (0x44E41F): the depth buffer used and written, compared with
//! less-or-equal (Direct3D render states 7 = 1, 14 = 1, 23 = 4).
//!
//! In:  nothing
//! Out: eax 4, ebx 0x17; the rest as `scene::set_render_state` leaves them (kept)
//!
//! Three calls of `scene::set_render_state`. 22 callers (the marks, the race's drawing).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::d3d::recorder::record;
use crate::framework::{Regs, SelfTest};
use crate::scene::set_render_state::set_render_state;

/// `scene::depth_buffer_on` (`on`) or `scene::depth_buffer_off`: states 7 and 14 1 or 0,
/// then 23 4.
pub unsafe fn depth_buffer(r: &mut Regs, on: bool) {
    unsafe {
        set_render_state(r, on as u32, 7);
        set_render_state(r, on as u32, 0xE);
        set_render_state(r, 4, 0x17);
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { depth_buffer(r, true) };
    Ok(())
}

/// One case: the original and `rust` recorded, nothing drawn; the registers and the
/// calls compared.
pub fn check(
    t: &mut SelfTest,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: &dyn Fn(&mut Regs),
) {
    let (theirs, their_calls) = record(true, || {
        let mut r = Regs::SENTINEL;
        unsafe { call_original(&mut r) };
        r
    });
    let (mine, my_calls) = record(true, || {
        let mut r = Regs::SENTINEL;
        rust(&mut r);
        r
    });
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    t.check(
        regs(&theirs) == regs(&mine) && their_calls == my_calls,
        || {
            format!(
                "original {:x?}, calls {their_calls:x?}; rust {:x?}, calls {my_calls:x?}",
                regs(&theirs),
                regs(&mine)
            )
        },
    );
}
