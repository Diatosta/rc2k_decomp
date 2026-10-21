//! `menu::ready_players` (0x471F7E): each counted player (0 .. `dword_755984`, the
//! original's `do .. while`) taking part again: +0xD0 1, +0x88 0, +0x94 0.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: `saves::load_savegame`, the single rally's and the time trial's multi-player
//! events. Refuses (before anything) a player count of 0 or past 4 or a player missing
//! (the original runs on past the table or reads through 0). Self-test: the game's players
//! with test values, for each count, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::player::player_regions;
use crate::menu::standings::{PLAYER_COUNT_RVA, counted_players, player, put, put_all};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::ready_players", rva(0x471F7E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn ready_players() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::event::PLAYER_READY`.
    for p in counted_players()? {
        put_all(p, &rc2k_core::menu::event::PLAYER_READY);
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    ready_players()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &player_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: Vec<Region> = player_regions();
    if regions.len() != 4 {
        return t.skip("a player missing from g_players");
    }
    let mut vars = regions.clone();
    vars.push(at(PLAYER_COUNT_RVA));
    let saved = capture(&vars);
    for count in 1..=4u32 {
        check_state(
            &mut t,
            &format!("{count} players"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(PLAYER_COUNT_RVA, count);
                for n in 0..4 {
                    for o in [0x88u32, 0x94, 0xD0] {
                        put(player(n) + o, 0x5EED_0000 + n * 0x100 + o);
                    }
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
