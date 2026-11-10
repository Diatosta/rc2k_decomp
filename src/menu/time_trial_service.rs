//! `menu::time_trial_service` (0x48E093): the time trial's service area before its stage.
//! `dword_75593C` 0xA8C (the service's minutes), `dword_75592C` 1; for each counted player
//! (`dword_75598C` 0 .. `dword_755984`, the original's `do .. while` on ecx, ebx 0): the
//! inspection's screen (`menu::inspection_screen`), a second's sleep, the fade in, 250 ms'
//! sleep, `dword_75AC8C` 0, the figures (`menu::driver_stats`), `dword_75AB38` 0, the
//! service page (`service::service_area_loop`; a yes to leaving ends it all), the repairs
//! (`service::apply_repairs`), the entry kind (`menu::set_player_entry_kind`); a player
//! now out (+0xD0 2) gets the faults (`menu::inspection_faults`) and the inspection page
//! (`menu::inspection_area_loop`; a yes ends it all) and counts in `dword_755978`. When
//! every player is out, `dword_75592C` 0. At the end `dword_755938` 0.
//!
//! `menu::service_pass` without its due test, its money and its keys. Callers: the time trial's
//! events. The calls through `event_flow::Call`. Refuses a player count of 0 or past 4 or a player
//! missing before anything, a current player past the four when it is read.

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::*;
use crate::menu::owner::Menus;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::session::Dword;

const SERVICE_TIME: &Dword = &crate::menu::owner::MENUS.screens.event.service_minutes; // dword_75593C
const GOES_ON: &Dword = &crate::menu::owner::MENUS.screens.event.event_goes_on; // dword_75592C
const OUT: &Dword = &crate::menu::owner::MENUS.screens.event.event_out; // dword_755978
const DUE: &Dword = &crate::menu::owner::MENUS.screens.event.service_due; // dword_755938

pub fn time_trial_service(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    // Converted: `rc2k_core::menu::passes` (the minutes, the pauses, who is out,
    // `goes_on`).
    use rc2k_core::menu::passes;
    crate::menu::standings::counted_players(screens)?;
    set(SERVICE_TIME, passes::TIME_TRIAL_MINUTES);
    set(GOES_ON, 1);
    let mut current = 0;
    'all: {
        loop {
            set(CURRENT, current);
            { crate::menu::inspection_screen::inspection_screen() }?;
            crate::system::sleep::sleep(passes::SCREEN_PAUSE);
            crate::menu::fade_in::fade_in(screens)?;
            crate::system::sleep::sleep(passes::FADE_PAUSE);
            screens.state.driver_stats.repair_time.set(0);
            { crate::menu::driver_stats::driver_stats(menus, screens) }?;
            screens.state.menus.service_flag.set(0);
            {
                run_screen(
                    &crate::service::service_area_loop::SCREEN,
                    &Entry::new(menus),
                )
            }?;
            if yes() {
                break 'all;
            }
            { crate::service::apply_repairs::apply_repairs_in_game(menus) }?;
            crate::menu::set_player_entry_kind::set_player_entry_kind(screens)?;
            // The current player as the screens leave it (the original reads it again).
            let serviced = get(CURRENT);
            if serviced >= 4 {
                return Err(BadInput("a current player past the four").into());
            }
            let record = player(serviced);
            if passes::is_out(record.get().entry) {
                crate::menu::inspection_faults::inspection_faults(menus)?;
                {
                    run_screen(
                        &crate::menu::inspection_area_loop::SCREEN,
                        &Entry::new(menus),
                    )
                }?;
                if yes() {
                    break 'all;
                }
                inc(OUT);
            }
            current += 1;
            if current == get(PLAYER_COUNT) {
                break;
            }
        }
        if !passes::goes_on(get(OUT), get(PLAYER_COUNT)) {
            set(GOES_ON, 0);
        }
    }
    set(DUE, 0);
    Ok(())
}
