//! `menu::service_pass` (0x4A15E3): the service area between stages. `dword_75ACC0` 0,
//! `dword_75593C` up by 900, `dword_75592C` 1; for each counted player (`dword_75598C` 0
//! .. `dword_755984`, the original's `do .. while`; ebx adds up their +0xD0) taking part
//! (+0xD0 1) when the service is due (`dword_755938` 1): the inspection's screen
//! (`menu::inspection_screen`), a second's sleep, the fade in, 250 ms' sleep,
//! `dword_75AC8C` 0, the figures (`menu::driver_stats`), `dword_75AB38` 0, the drivers'
//! condition reset (`menu::reset_driver_condition`), the repair flags cleared
//! (`service::clear_repair_flags`), six keys taken (`input::take_key`), the service page
//! (`service::service_area_loop`; a yes to leaving ends it all), the repairs
//! (`service::apply_repairs`), the entry kind (`menu::set_player_entry_kind`); a player
//! now out (+0xD0 2) gets the faults (`menu::inspection_faults`) and the inspection page
//! (`menu::inspection_area_loop`; a yes ends it all) and counts in `dword_755978`; its
//! +0x288 then `dword_75ACC0` x 100. When every player is out, `dword_75592C` 0. At the
//! end `dword_755938` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a player count of 0 or past 4, a player missing, a
//! taking-part player's car without its table, before anything.

use crate::errors::Refused;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_loop::{Entry, run_screen};
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;
// service::service_area_loop

/// The Rust version (see the header).
pub fn service_pass() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::passes` (who is serviced, the minutes, the money, the
    // pauses, `goes_on`).
    use rc2k_core::menu::passes;
    let players = counted_players()?;
    let due = { EVENT_STATE.service_due.get() } == 1;
    for &p in &players {
        if due && passes::takes_part(p.get().entry) {
            crate::menu::driver_stats::car_table(p)?;
        }
    }
    MENU_STATE.driver_stats.money.set(0u32);
    EVENT_STATE
        .service_minutes
        .set(passes::more_minutes(EVENT_STATE.service_minutes.get()));
    EVENT_STATE.event_goes_on.set(1u32);
    'all: {
        for (n, &p) in players.iter().enumerate() {
            EVENT_STATE.current_player.set(n as u32);
            let entry = p.get().entry;
            if !passes::serviced(entry, || EVENT_STATE.service_due.get()) {
                continue;
            }
            crate::menu::inspection_screen::inspection_screen()?;
            crate::system::sleep::sleep(passes::SCREEN_PAUSE);
            crate::menu::fade_in::fade_in()?;
            crate::system::sleep::sleep(passes::FADE_PAUSE);
            MENU_STATE.driver_stats.repair_time.set(0);
            crate::menu::driver_stats::driver_stats()?;
            MENU_STATE.menus.service_flag.set(0);
            crate::menu::reset_driver_condition::reset_driver_condition()?;
            crate::service::clear_repair_flags::clear_repair_flags()?;
            for _ in 0..passes::DROPPED_KEYS {
                crate::input::take_key::take_key();
            }
            run_screen(
                &crate::service::service_area_loop::SCREEN,
                &Entry::default(),
            )?;
            if MENU_PAGES.yes_no_answer.get() == 1 {
                break 'all;
            }
            crate::service::apply_repairs::apply_repairs_in_game()?;
            crate::menu::set_player_entry_kind::set_player_entry_kind()?;
            if passes::is_out(p.get().entry) {
                crate::menu::inspection_faults::inspection_faults()?;
                run_screen(
                    &crate::menu::inspection_area_loop::SCREEN,
                    &Entry::default(),
                )?;
                if MENU_PAGES.yes_no_answer.get() == 1 {
                    break 'all;
                }
                EVENT_STATE
                    .event_out
                    .set(EVENT_STATE.event_out.get().wrapping_add(1));
            }
            let spent = passes::spent(MENU_STATE.driver_stats.money.get());
            p.update(|q| q.store(&[(0x288, spent)]));
        }
        if !passes::goes_on(EVENT_STATE.event_out.get(), PLAYER_COUNT_RVA.get()) {
            EVENT_STATE.event_goes_on.set(0u32);
        }
    }
    EVENT_STATE.service_due.set(0u32);
    Ok(())
}
