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
    let draw_work = &o.menus.screens.draw_work;
    draw_work.write().escape_pressed = false;
    { run_screen(ScreenId::SelectStage, &Entry::new(o)) }?;
    if !draw_work.read().escape_pressed {
        let stage = race.shared.state.settings.stage.get() as u8;
        vars.own.update(|r| r.set_byte(record::STAGE, stage));
        // The direction is bit 0 of the byte; the bots' setting in the others stays.
        let direction = rc2k_core::net::bots::with_direction(
            vars.own.get().byte(record::DIRECTION),
            race.shared.state.settings.direction.get() as u8,
        );
        vars.own
            .update(|r| r.set_byte(record::DIRECTION, direction));
    }
    crate::net::show_background::show_background(o)?;
    Ok(())
}
