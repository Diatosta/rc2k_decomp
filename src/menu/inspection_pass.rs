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
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The figures, verdict and faults are their Rust versions;
//! the sleeps, the screen, the fades and the page through `part`, in the game their Rust
//! versions (the sleep `system::sleep`'s). Ungated (the page waits for the player).
//! Refuses a player count of 0 or past 4, a player missing, a taking-part player's car
//! without its table, before anything. Shadow mode runs the original alone (it logs and
//! shows). Self-test: a copy whose screen, sleep, fade and
//! page calls go to stand-ins, against the Rust version, players taking part by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Carried, run_screen_carried};
use crate::menu::standings::*;

const GOES_ON_RVA: usize = rva(0x75592C); // dword_75592C
const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const FAILED_RVA: usize = rva(0x755978); // dword_755978
const ANSWER_RVA: usize = rva(0x75524C); // g_yesNoAnswer

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn inspection_pass(c: &Carried) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::passes` (who is looked at, the pauses, `goes_on`).
    use rc2k_core::menu::passes;
    let players = counted_players()?;
    for &p in &players {
        if passes::takes_part(get(p + 0xD0)) {
            crate::menu::driver_stats::car_table(p)?;
        }
    }
    let mut g = *c;
    unsafe {
        game_write(GOES_ON_RVA, 1u32);
        for (n, &p) in players.iter().enumerate() {
            g.measure_x = n as u32;
            game_write(CURRENT_RVA, g.measure_x);
            if !passes::takes_part(get(p + 0xD0)) {
                continue;
            }
            crate::menu::driver_stats::driver_stats()?;
            crate::menu::inspection_verdict::inspection_verdict()?;
            crate::menu::inspection_faults::inspection_faults()?;
            if !passes::is_out(get(p + 0xD0)) {
                continue;
            }
            game_write(FAILED_RVA, game_read::<u32>(FAILED_RVA).wrapping_add(1));
            crate::menu::inspection_screen::inspection_screen()?;
            g.arg = passes::SCREEN_PAUSE;
            crate::system::sleep::sleep(g.arg);
            crate::menu::fade_in::fade_in()?;
            run_screen_carried(&crate::menu::inspection_area_loop::SCREEN, &mut g)?;
            if game_read::<u32>(ANSWER_RVA) == 1 {
                return Ok(());
            }
            crate::menu::fades::fade(true)?;
            g.arg = passes::INSPECTION_PAUSE;
            crate::system::sleep::sleep(g.arg);
        }
        if !passes::goes_on(game_read(FAILED_RVA), game_read(PLAYER_COUNT_RVA)) {
            game_write(GOES_ON_RVA, 0u32);
        }
    }
    Ok(())
}
