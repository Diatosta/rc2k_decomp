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
//! `menu::service_pass` without its due test, its money and its keys. Callers: the time
//! trial's events. The calls through `event_flow::Call` (in the game `game_call`).
//! Ungated (the pages wait for the player). Refuses a player count of 0 or past 4 or a
//! player missing before anything, a current player past the four when it is read.
//! Shadow mode runs the original alone. Self-test: `event_flow::test_flow`, scripts of
//! the two pages' answers and the players' +0xD0.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::menu::event_flow::*;

const SERVICE_TIME: usize = 0x75593C; // dword_75593C
const GOES_ON: usize = 0x75592C; // dword_75592C
const CLEARED: [usize; 2] = [0x75AC8C, 0x75AB38]; // dword_75AC8C, dword_75AB38
const OUT: usize = 0x755978; // dword_755978
const DUE: usize = 0x755938; // dword_755938

pub unsafe fn time_trial_service(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::passes` (the minutes, the pauses, who is out,
    // `goes_on`).
    use rc2k_core::menu::passes;
    crate::menu::standings::counted_players()?;
    let mut g = *entry;
    set(SERVICE_TIME, passes::TIME_TRIAL_MINUTES);
    set(GOES_ON, 1);
    g.ecx = 0;
    g.ebx = 0;
    'all: {
        loop {
            set(CURRENT, g.ecx);
            call(INSPECTION_SCREEN, &mut g)?;
            g.eax = passes::SCREEN_PAUSE;
            call(SLEEP, &mut g)?;
            call(FADE_IN, &mut g)?;
            g.eax = passes::FADE_PAUSE;
            call(SLEEP, &mut g)?;
            set(CLEARED[0], 0);
            call(DRIVER_STATS, &mut g)?;
            set(CLEARED[1], 0);
            call(SERVICE_AREA_LOOP, &mut g)?;
            if yes() {
                break 'all;
            }
            call(APPLY_REPAIRS, &mut g)?;
            call(SET_PLAYER_ENTRY_KIND, &mut g)?;
            let current = get(CURRENT);
            if current >= 4 || player(current) == 0 {
                return Err(BadInput("a current player past the four or missing").into());
            }
            g.edi = player(current);
            if passes::is_out(dword(g.edi + 0xD0)) {
                call(INSPECTION_FAULTS, &mut g)?;
                call(INSPECTION_AREA_LOOP, &mut g)?;
                if yes() {
                    break 'all;
                }
                inc(OUT);
            }
            g.ecx = g.ecx.wrapping_add(1);
            if g.ecx == get(PLAYER_COUNT) {
                break;
            }
        }
        if !passes::goes_on(get(OUT), get(PLAYER_COUNT)) {
            set(GOES_ON, 0);
        }
    }
    set(DUE, 0);
    Ok(*entry)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { time_trial_service(r, &mut game_call) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
