//! `race::draw_view_tint` (0x456092): the whole view (0 .. `dword_62408C`, 0 ..
//! `dword_624090`) as one screen quad in 0x204080, by `race::draw_screen_quad` (see
//! `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: eax 0x204080; the rest kept (a `retn` follows)
//!
//! Caller: `sub_4557AB` (0x455890). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose quad call goes to a copy of `draw_screen_quad` with its
//! drawing calls stood in, against the Rust version on the same; view sizes; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { draw_view_tint(r, &mut quad_part) }?;
        r.eax = after.eax
    }
    Ok(())
}
