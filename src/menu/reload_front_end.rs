//! `menu::reload_front_end` (0x48140D): when `arcade.scene_let_go` is 1 (the scene was let go for
//! a race), the front end's scene set up again: `scene::set_up_engine`,
//! `scene::set_up_front_end_scene`, `scene::set_up_engine`.
//!
//! 36 callers (the menus after their races).

use crate::errors::Refused;
use crate::owners::Owners;

pub fn reload_front_end(o: Owners<'static>) -> Result<(), Refused> {
    if { o.menus.play.arcade.scene_let_go.get() } == 1 {
        crate::scene::set_up_engine::set_up_engine_in(o.race, o.scene)?;
        crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
        crate::scene::set_up_engine::set_up_engine_in(o.race, o.scene)?;
    }
    Ok(())
}
