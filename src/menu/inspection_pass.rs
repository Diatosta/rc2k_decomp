//! `menu::inspection_pass`: the inspection after a stage. `event.event_goes_on` 1;
//! for each counted player (`event.current_player` 0 .. `event.player_count`) taking part
//! (`entry` is `TAKES_PART`): its figures (`menu::driver_stats`), the verdict (`menu::inspection_verdict`) and
//! the faults (`menu::inspection_faults`); one that fails (`entry` now 2) counts in
//! `event.event_out` and is shown the inspection: its screen (`menu::inspection_screen`), a
//! second's pause (a key ends it), the fade in, the inspection page (`menu::inspection_area_loop`; a yes to
//! leaving ends it all), the fade out, a pause of 1250 ms (a key ends it). When every player has failed
//! (`event.event_out` the player count), `event.event_goes_on` 0 (the event is over).
//!
//! Refuses a player count of 0 or past 4, a player missing, a taking-part player's car without its
//! table, before anything.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;

pub(super) fn inspection_pass(menus: &Menus, entry: &Entry) -> Result<(), Refused> {
    let screens = &menus.screens;
    use rc2k_core::menu::passes;
    let players = counted_players(&menus.screens, &menus.play)?;
    for &p in &players {
        if passes::takes_part(p.get().entry) {
            crate::menu::driver_stats::car_table(menus, p)?;
        }
    }
    menus.screens.event.event_goes_on.set(1u32);
    for (n, &p) in players.iter().enumerate() {
        menus.screens.event.current_player.set(n as u32);
        if !passes::takes_part(p.get().entry) {
            continue;
        }
        crate::menu::driver_stats::driver_stats(menus, screens)?;
        crate::menu::inspection_verdict::inspection_verdict(menus)?;
        crate::menu::inspection_faults::inspection_faults(menus)?;
        if !passes::is_out(p.get().entry) {
            continue;
        }
        menus
            .screens
            .event
            .event_out
            .set(menus.screens.event.event_out.get().wrapping_add(1));
        crate::menu::inspection_screen::inspection_screen(entry.owners)?;
        crate::system::pause::pause(entry.owners, passes::SCREEN_PAUSE);
        crate::menu::fade_in::fade_in(entry.owners)?;
        run_screen(ScreenId::InspectionArea, entry)?;
        if menus.tables.pages.read().yes_no_answer == 1 {
            return Ok(());
        }
        crate::menu::fades::fade(entry.owners, true)?;
        crate::system::pause::pause(entry.owners, passes::INSPECTION_PAUSE);
    }
    if !passes::goes_on(
        menus.screens.event.event_out.get(),
        menus.screens.event.player_count.get(),
    ) {
        menus.screens.event.event_goes_on.set(0u32);
    }
    Ok(())
}
