//! `race::shadow_faces` (0x457618): the car's shadow on the course faces round it (see
//! `race/car_shadow.rs`): the last course record's faces, then every record in the 3 x 3
//! cells round the work car; each face below kind 0x10 and without bit 7 of its +2 word
//! as two triangles by `race::shadow_on_face` (`sub_4576C9`, covered here).
//!
//! In:  nothing
//! Out: ebx the record count (when any); the rest kept (a `retn` follows)
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to a
//! copy of `sub_4576C9` whose face calls go to a stand-in that notes the corners, against
//! the Rust version noting the same; course records and faces of the test's (pointers
//! set), the car in cells round them; all put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_shadow::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let done = unsafe { shadow_faces(r, &mut |e| shadow_on_face(e, &mut face_part).map(|_| ())) };
    {
        let after = done?;
        r.ebx = after.ebx
    }
    Ok(())
}
