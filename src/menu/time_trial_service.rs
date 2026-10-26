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
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! `menu::service_pass` without its due test, its money and its keys. Callers: the time trial's
//! events. The calls through `event_flow::Call`. Refuses a player count of 0 or past 4 or a player
//! missing before anything, a current player past the four when it is read.

use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::*;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::screen_loop::run_screen_carried;
use crate::menu::state_store::MENU_STATE;
use crate::session::Dword;

const SERVICE_TIME: &Dword = &EVENT_STATE.service_minutes; // dword_75593C
const GOES_ON: &Dword = &EVENT_STATE.event_goes_on; // dword_75592C
const OUT: &Dword = &EVENT_STATE.event_out; // dword_755978
const DUE: &Dword = &EVENT_STATE.service_due; // dword_755938

/// The Rust version. `f` the registers the caller carries; the original keeps every
/// register of it (`pusha` .. `popa`).
pub unsafe fn time_trial_service(f: &Flow) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::passes` (the minutes, the pauses, who is out,
    // `goes_on`).
    use rc2k_core::menu::passes;
    crate::menu::standings::counted_players()?;
    let mut g = f.c;
    set(SERVICE_TIME, passes::TIME_TRIAL_MINUTES);
    set(GOES_ON, 1);
    g.measure_x = 0;
    g.palette = 0;
    'all: {
        loop {
            set(CURRENT, g.measure_x);
            unsafe { crate::menu::inspection_screen::inspection_screen() }?;
            g.arg = passes::SCREEN_PAUSE;
            crate::system::sleep::sleep(g.arg);
            crate::menu::fade_in::fade_in()?;
            g.arg = passes::FADE_PAUSE;
            crate::system::sleep::sleep(g.arg);
            MENU_STATE.driver_stats.repair_time.set(0);
            unsafe { crate::menu::driver_stats::driver_stats() }?;
            MENU_STATE.menus.service_flag.set(0);
            unsafe { run_screen_carried(&crate::service::service_area_loop::SCREEN, &mut g) }?;
            if yes() {
                break 'all;
            }
            unsafe { crate::service::apply_repairs::apply_repairs_in_game() }?;
            crate::menu::set_player_entry_kind::set_player_entry_kind()?;
            let current = get(CURRENT);
            if current >= 4 || player(current) == 0 {
                return Err(BadInput("a current player past the four or missing").into());
            }
            let record = player(current);
            if passes::is_out(dword(record + 0xD0)?) {
                crate::menu::inspection_faults::inspection_faults()?;
                unsafe { run_screen_carried(&crate::menu::inspection_area_loop::SCREEN, &mut g) }?;
                if yes() {
                    break 'all;
                }
                inc(OUT);
            }
            g.measure_x = g.measure_x.wrapping_add(1);
            if g.measure_x == get(PLAYER_COUNT) {
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
