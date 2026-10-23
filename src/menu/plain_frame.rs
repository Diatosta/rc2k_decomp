//! Frame callbacks that only call, in order, a few of the game's functions, some of which
//! end the frame with a set carry (`menu::picture_frame`, `menu::fade_frame`): the steps,
//! and their test (a copy whose calls go to `menu::frame_test`'s stand-ins, each failing
//! in turn, against the Rust version on the same).

use crate::framework::{Regs, rva};
use crate::menu::screen_loop::Env;

pub const LOCK_SURFACE: usize = rva(0x4058AD); // gfx::lock_surface
pub const UNLOCK_SURFACE: usize = rva(0x405949); // gfx::unlock_surface
pub const COPY_PICTURE: usize = rva(0x4060E3); // gfx::copy_picture_to_surface
pub const BEGIN_SCENE: usize = rva(0x44E8A1); // scene::begin_scene
pub const END_SCENE: usize = rva(0x44E952); // scene::end_scene
pub const SHOW_FRAME: usize = rva(0x4057D1); // gfx::show_frame

/// A step: the function, and whether a set carry after it ends the frame.
pub type Step = (usize, bool);

/// The steps in order (every register kept, `pusha` .. `popa`): whether the frame was
/// made (the original's carry clear).
pub unsafe fn run_steps(entry: &Regs, steps: &[Step], env: &mut dyn Env) -> bool {
    let mut g = *entry;
    for &(function, checked) in steps {
        env.call(function, &mut g);
        if checked && g.carry() {
            return false;
        }
    }
    true
}
