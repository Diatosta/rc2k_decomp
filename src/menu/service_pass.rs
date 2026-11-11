//! `menu::service_pass`: the service area between stages. The money is cleared, the service
//! minutes go up by 900 and `event_goes_on` is set to 1; for each counted player
//! (`current_player` 0 .. `player_count`, the original's `do .. while`; ebx adds up their
//! +0xD0) taking part (+0xD0 1) when the service is due (`service_due` 1): the inspection's
//! screen (`menu::inspection_screen`), a second's sleep, the fade in, 250 ms' sleep, the time
//! of the chosen repairs cleared, the figures (`menu::driver_stats`), the service flag
//! cleared, the drivers' condition reset (`menu::reset_driver_condition`), the repair flags
//! cleared (`service::clear_repair_flags`), six keys taken (`input::take_key`), the service
//! page (`service::service_area_loop`; a yes to leaving ends it all), the repairs
//! (`service::apply_repairs`), the entry kind (`menu::set_player_entry_kind`); a player now
//! out (+0xD0 2) gets the faults (`menu::inspection_faults`) and the inspection page
//! (`menu::inspection_area_loop`; a yes ends it all) and is counted in `event_out`; its
//! +0x288 then takes the money x 100. When every player is out, `event_goes_on` is cleared.
//! At the end `service_due` is cleared.
//!
//! Callers: the event drivers. Refuses a player count of 0 or past 4, a player missing, a
//! taking-part player's car without its table, before anything.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;
// service::service_area_loop
pub fn service_pass(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    // The logic is in `rc2k_core::menu::passes` (who is serviced, the minutes, the money, the
    // pauses, `goes_on`).
    use rc2k_core::menu::passes;
    let players = counted_players(menus)?;
    let due = { menus.screens.event.service_due.get() } == 1;
    for &p in &players {
        if due && passes::takes_part(p.get().entry) {
            crate::menu::driver_stats::car_table(menus, p)?;
        }
    }
    menus.screens.state.driver_stats.money.set(0u32);
    menus
        .screens
        .event
        .service_minutes
        .set(passes::more_minutes(
            menus.screens.event.service_minutes.get(),
        ));
    menus.screens.event.event_goes_on.set(1u32);
    'all: {
        for (n, &p) in players.iter().enumerate() {
            menus.screens.event.current_player.set(n as u32);
            let entry = p.get().entry;
            if !passes::serviced(entry, || menus.screens.event.service_due.get()) {
                continue;
            }
            crate::menu::inspection_screen::inspection_screen()?;
            crate::system::sleep::sleep(passes::SCREEN_PAUSE);
            crate::menu::fade_in::fade_in(screens)?;
            crate::system::sleep::sleep(passes::FADE_PAUSE);
            menus.screens.state.driver_stats.repair_time.set(0);
            crate::menu::driver_stats::driver_stats(menus, screens)?;
            menus.screens.state.menus.service_flag.set(false);
            crate::menu::reset_driver_condition::reset_driver_condition(menus)?;
            crate::service::clear_repair_flags::clear_repair_flags(menus)?;
            for _ in 0..passes::DROPPED_KEYS {
                crate::input::take_key::take_key();
            }
            run_screen(
                &crate::service::service_area_loop::SCREEN,
                &Entry::new(menus),
            )?;
            if menus.tables.pages.yes_no_answer.get() == 1 {
                break 'all;
            }
            crate::service::apply_repairs::apply_repairs_in_game(menus)?;
            crate::menu::set_player_entry_kind::set_player_entry_kind(menus)?;
            if passes::is_out(p.get().entry) {
                crate::menu::inspection_faults::inspection_faults(menus)?;
                run_screen(
                    &crate::menu::inspection_area_loop::SCREEN,
                    &Entry::new(menus),
                )?;
                if menus.tables.pages.yes_no_answer.get() == 1 {
                    break 'all;
                }
                menus
                    .screens
                    .event
                    .event_out
                    .set(menus.screens.event.event_out.get().wrapping_add(1));
            }
            let spent = passes::spent(menus.screens.state.driver_stats.money.get());
            p.update(|q| q.service_spent = spent);
        }
        if !passes::goes_on(menus.screens.event.event_out.get(), PLAYER_COUNT_RVA.get()) {
            menus.screens.event.event_goes_on.set(0u32);
        }
    }
    menus.screens.event.service_due.set(0u32);
    Ok(())
}
