//! `menu::time_trial_service`: the time trial's service area before its stage. The service
//! minutes are set to 0xA8C and `event_goes_on` to 1; for each counted player (`current_player`
//! 0 .. `player_count`): the inspection's screen (`menu::inspection_screen`), a second's pause,
//! the fade in, a pause of 250 ms (a key ends either), the time of the chosen repairs cleared, the figures
//! (`menu::driver_stats`), the service flag cleared, the service page
//! (`service::service_area_loop`; a yes to leaving ends it all), the repairs
//! (`service::apply_repairs`), the entry kind (`menu::set_player_entry_kind`); a player now out
//! (`entry` 2) gets the faults (`menu::inspection_faults`) and the inspection page
//! (`menu::inspection_area_loop`; a yes ends it all) and is counted in `event_out`. When every
//! player is out, `event_goes_on` is cleared. At the end `service_due` is cleared.
//!
//! `menu::service_pass` without its due test, its money and its keys. The calls go through
//! `event_flow::Call`. Refuses a player count of 0 or past 4 or a player missing before anything, a
//! current player past the four when it is read.

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::*;
use crate::menu::owner::Menus;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_loop::{Entry, run_screen};

/// The service area for a launch: each counted player's service without its screens, pauses and
/// repairs shown. `tune` is run for the current player where the area's choices would be made
/// (after the figures, before the entry kind is set); a player out at inspection is refused, as
/// there is no inspection page to show.
pub(super) fn time_trial_service_unshown(
    menus: &Menus,
    tune: &mut dyn FnMut(&Menus) -> Result<(), Refused>,
) -> Result<(), Refused> {
    use rc2k_core::menu::passes;
    let screens = &menus.screens;
    let event = &screens.event;
    crate::menu::standings::counted_players(&menus.screens, &menus.play)?;
    event.write().service.minutes = passes::TIME_TRIAL_MINUTES;
    event.write().event_goes_on = 1;
    let mut current = 0;
    loop {
        event.write().current_player = current;
        menus.service.repair_time.set(0);
        { crate::menu::driver_stats::driver_stats(menus, screens) }?;
        screens.state.write().menus.service_flag = false;
        tune(menus)?;
        crate::menu::set_player_entry_kind::set_player_entry_kind(menus)?;
        let serviced = event.read().current_player;
        if serviced >= 4 {
            return Err(BadInput("a current player past the four").into());
        }
        if passes::is_out(player(menus, serviced).get().entry) {
            return Err(BadInput("a launched time trial's car is out at the inspection").into());
        }
        current += 1;
        if current == event.read().player_count {
            break;
        }
    }
    event.write().service.due = 0;
    Ok(())
}

pub fn time_trial_service(menus: &Menus, entry: &Entry) -> Result<(), Refused> {
    let screens = &menus.screens;
    let event = &screens.event;
    // The logic is in `rc2k_core::menu::passes` (the minutes, the pauses, who is out,
    // `goes_on`).
    use rc2k_core::menu::passes;
    crate::menu::standings::counted_players(&menus.screens, &menus.play)?;
    event.write().service.minutes = passes::TIME_TRIAL_MINUTES;
    event.write().event_goes_on = 1;
    let mut current = 0;
    'all: {
        loop {
            event.write().current_player = current;
            { crate::menu::inspection_screen::inspection_screen(entry.owners) }?;
            crate::system::pause::pause(entry.owners, passes::SCREEN_PAUSE);
            crate::menu::fade_in::fade_in(entry.owners)?;
            crate::system::pause::pause(entry.owners, passes::FADE_PAUSE);
            menus.service.repair_time.set(0);
            { crate::menu::driver_stats::driver_stats(menus, screens) }?;
            screens.state.write().menus.service_flag = false;
            { run_screen(ScreenId::ServiceArea, entry) }?;
            if yes(&menus.tables.pages) {
                break 'all;
            }
            { crate::service::apply_repairs::apply_repairs_in_game_in(entry.owners) }?;
            crate::menu::set_player_entry_kind::set_player_entry_kind(menus)?;
            // The current player as the screens leave it (it is read again).
            let serviced = event.read().current_player;
            if serviced >= 4 {
                return Err(BadInput("a current player past the four").into());
            }
            let record = player(menus, serviced);
            if passes::is_out(record.get().entry) {
                crate::menu::inspection_faults::inspection_faults(menus)?;
                { run_screen(ScreenId::InspectionArea, entry) }?;
                if yes(&menus.tables.pages) {
                    break 'all;
                }
                {
                    let mut values = event.write();
                    values.event_out = values.event_out.wrapping_add(1);
                }
            }
            current += 1;
            if current == event.read().player_count {
                break;
            }
        }
        if !passes::goes_on(event.read().event_out, event.read().player_count) {
            event.write().event_goes_on = 0;
        }
    }
    event.write().service.due = 0;
    Ok(())
}
