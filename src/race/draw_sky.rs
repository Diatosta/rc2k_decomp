//! `race::draw_sky` (0x455540): the sky and the sun for a view (see `race/frame_draw.rs`):
//! `race::fetch_view_value`; unless the scene's mode is 1 or 2, the sky (object 1 on
//! `race::sky_matrix`, full light) drawn; then the sun's direction turned by the sky's
//! matrix (`scene::rotate_point`) and, in front, its place and size on the screen for the
//! flare (`dword_6415C4` .. `dword_6415DC`).
//!
//! In:  nothing
//! Out: the registers as its parts leave them (a `call` follows)
//!
//! In the game the render states (`sub_44E44D`) and the batch's flush (`sub_45FD1D`) are called
//! in the game (section 15); the drawing by `scene::draw_scene_object_in_game` and the depth
//! buffer by `scene::depth_buffer_on`'s Rust versions. Hook removed 2026-10-22 (direct calls):
//! Rust callers run its `entry` (`calls::call`, `routes.rs`); the registers above are the
//! original's, for reference.

use crate::framework::Regs;
use crate::race::frame_draw::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let after = unsafe { draw_sky(r) }?;
        let esp = r.esp;
        *r = after;
        r.esp = esp;
    }
    Ok(())
}
