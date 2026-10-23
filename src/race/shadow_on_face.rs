//! `race::shadow_on_face` (0x45773C): the car's shadow cut to one course face and drawn on
//! it (see `race/car_shadow.rs`): the hull's list as the polygon, `race::clip_shadow_to_face`,
//! each vertex raised onto the face (`sub_457B86`, covered here: `physics::face_shares` and
//! `physics::plane_height`), projected (`scene::project_polygon`, `scene::fog_polygon`),
//! cut by depth and to the screen, drawn in mode 5 (`scene::draw_polygon_in_game`).
//!
//! In:  nothing (the face's corners `dword_641938` ..)
//! Out: the registers as it leaves them (its caller calls it inside `pusha`)
//!
//! Refuses (before anything) a hull list past 0x40; what the projection and the clips
//! refuse (after the work is set). Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose drawing call goes to a stand-in, the rest real, against the
//! Rust version on the same; the test square and faces of `clip_shadow_to_face`'s test;
//! all put back (skipped without a race: the projection needs the view).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::car_shadow::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { shadow_on_face(r) }?;
    Ok(())
}
