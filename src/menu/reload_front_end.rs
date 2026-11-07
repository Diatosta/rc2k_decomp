//! `menu::reload_front_end` (0x48140D): when `dword_7587E8` is 1 (the scene was let go for
//! a race), the front end's scene set up again: `scene::set_up_engine`,
//! `scene::set_up_front_end_scene`, `scene::set_up_engine`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! 36 callers (the menus after their races).

use crate::errors::Refused;
use crate::menu::owner::Menus;

/// The Rust version (see the header).
pub fn reload_front_end(menus: &Menus) -> Result<(), Refused> {
    if { menus.play.arcade.scene_let_go.get() } == 1 {
        crate::scene::set_up_engine::set_up_engine()?;
        crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
        crate::scene::set_up_engine::set_up_engine()?;
    }
    Ok(())
}
