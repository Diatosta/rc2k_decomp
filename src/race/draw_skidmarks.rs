//! `race::draw_skidmarks` (0x458C03): the skidmarks shown in the drawn view, a quad for
//! each link with a link before (see `race/skidmarks.rs`; the x87 runs the original's
//! instructions, the turn by `scene::rotate_point`'s Rust version), each faded
//! (`scene::fog_polygon`), cut by depth and to the screen and drawn in black in mode 3
//! without the depth test (`scene::depth_test_off`, `scene::draw_polygon_in_game`,
//! `scene::depth_test_on`).
//!
//! In:  nothing
//! Out: edx 0x400, eax, ecx, esi as it leaves them (a `retn` follows)
//!
//! Caller: `race::draw_skidmarks_if_on`. Refuses what the scene's functions refuse
//! (mid-way; not met with four vertices). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose depth-test and drawing calls go to stand-ins (the fade and the
//! clips real), against the Rust version on the same; chains of links of the test's in
//! the views and not; all put back (skipped without a race).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::skidmarks::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { draw_skidmarks(r, &mut skid_part) }?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
