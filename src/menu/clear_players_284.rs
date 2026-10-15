//! `menu::clear_players_284` (0x4A9679): every player's `field_284` set to -1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The four players of `g_players`. Called at the start of each kind of event
//! (`sub_482382` and seven more). Self-test: the game's players, theirs and the test's
//! values, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::player::{PLAYERS, Player, player_at, player_regions};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::clear_players_284", rva(0x4A9679), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn clear_players_284() -> Result<(), Refused> {
    for n in 0..PLAYERS {
        let p = player_at(n);
        if p == 0 {
            return Err(crate::errors::BadInput("a player missing from g_players").into());
        }
    }
    for n in 0..PLAYERS {
        unsafe { (*(player_at(n) as *mut Player)).field_284 = -1 };
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    clear_players_284()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &player_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = player_regions();
    if regions.len() != PLAYERS as usize {
        return t.skip("a player missing from g_players");
    }
    let saved = capture(&regions);
    for value in [None, Some(0), Some(7), Some(-1)] {
        check_state(
            &mut t,
            &format!("field_284 {value:?}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&regions, &saved);
                if let Some(v) = value {
                    for n in 0..PLAYERS {
                        unsafe { (*(player_at(n) as *mut Player)).field_284 = v + n as i32 };
                    }
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
