//! `menu::inspection_pass` (0x4A8556): the inspection after a stage. `dword_75592C` 1;
//! for each counted player (`dword_75598C` 0 .. `dword_755984`, the original's `do ..
//! while`) taking part (+0xD0 1): its figures (`menu::driver_stats`), the verdict
//! (`menu::inspection_verdict`) and the faults (`menu::inspection_faults`); one that fails
//! (+0xD0 now 2) counts in `dword_755978` and is shown the inspection: its screen
//! (`menu::inspection_screen`), a second's sleep, the fade in, the inspection page
//! (`menu::inspection_area_loop`; a yes to leaving ends it all), the fade out, 1250 ms'
//! sleep. When every player has failed (`dword_755978` the player count), `dword_75592C`
//! 0 (the event is over).
//!
//! Callers: the event drivers. Refuses a
//! player count of 0 or past 4, a player missing, a taking-part player's car without its table,
//! before anything.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;

pub fn inspection_pass(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Converted: `rc2k_core::menu::passes` (who is looked at, the pauses, `goes_on`).
    use rc2k_core::menu::passes;
    let players = counted_players(screens)?;
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
        crate::menu::inspection_screen::inspection_screen()?;
        crate::system::sleep::sleep(passes::SCREEN_PAUSE);
        crate::menu::fade_in::fade_in(screens)?;
        run_screen(
            &crate::menu::inspection_area_loop::SCREEN,
            &Entry::new(menus),
        )?;
        if menus.tables.pages.yes_no_answer.get() == 1 {
            return Ok(());
        }
        crate::menu::fades::fade(screens, true)?;
        crate::system::sleep::sleep(passes::INSPECTION_PAUSE);
    }
    if !passes::goes_on(menus.screens.event.event_out.get(), PLAYER_COUNT_RVA.get()) {
        menus.screens.event.event_goes_on.set(0u32);
    }
    Ok(())
}
