//! `race::draw_skidmarks_if_on` (0x4585E3): `race::draw_skidmarks` unless `dword_6416B8`
//! is set (see `race/skidmarks.rs`).
//!
//! In:  nothing
//! Out: as `draw_skidmarks` leaves them, or all kept
//!
//! Caller: `sub_4557AB` (0x455853), each view drawn. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose call goes to a stand-in, the switch on and off.

use crate::framework::Regs;
use crate::race::skidmarks::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { draw_skidmarks_if_on(r, &mut |e| draw_skidmarks(e, &mut skid_part)) }?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
