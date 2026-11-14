//! The lobby's stage item (the host's): runs the stage screen; unless left with Escape, the
//! chosen stage and its direction go into the player's record. The background is drawn again.

use crate::errors::Refused;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::net::store::NetShared;
use crate::owners::Owners;
use rc2k_core::net::record;

/// The stage screen; the lobby loop's leftovers it carries (see `net::choose_car`) are all 0
/// here.
pub fn choose_stage(o: Owners<'static>, vars: &NetShared) -> Result<(), Refused> {
    let race = o.race;
    let escape_pressed = &o.menus.screens.draw_work.escape_pressed;
    escape_pressed.set(false);
    { run_screen(ScreenId::SelectStage, &Entry::new(o)) }?;
    if !escape_pressed.is_set() {
        vars.own
            .head
            .set(record::STAGE, race.shared.state.settings.stage.get() as u8);
        vars.own.head.set(
            record::DIRECTION,
            race.shared.state.settings.direction.get() as u8,
        );
    }
    crate::net::show_background::show_background(o)?;
    Ok(())
}
