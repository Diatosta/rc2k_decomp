//! `menu::add_player_times` (0x4A25AD): each counted player's (`dword_755984`) stage time
//! (+0x88) added to the total (+0x8C) when it took part (+0xD0 1), else both 3600000.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a player count of 0 or past 4 or a missing player,
//! before anything. Self-test: the original and the Rust version on the game's players,
//! the flags by lot; put back.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::add_player_times", rva(0x4A25AD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EXTRA: [usize; 0] = [];
const CASES: u32 = 3;

fn test_entry() -> Regs {
    Regs::SENTINEL
}

fn refusal_expected() -> bool {
    false
}

fn prepare(case: u32) {
    let _ = case;
    for n in 0..4u32 {
        if player(n) != 0 {
            put(player(n) + 0xD0, u32::from((n + case).is_multiple_of(2)));
            put(player(n) + 0x88, 12_345 * (n + 1));
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn add_player_times(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::event::added_time`.
    for p in counted_players()? {
        let entered = get(p + 0xD0);
        // The times are read only for a player that took part, as the original.
        let (time, total) = if entered == 1 {
            (get(p + 0x88), get(p + 0x8C))
        } else {
            (0, 0)
        };
        put_all(p, &rc2k_core::menu::event::added_time(entered, time, total));
    }
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    (0..4u32)
        .map(|n| (player(n), 0x400))
        .filter(|&(p, _)| p != 0)
        .collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { add_player_times(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = test_entry();
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { add_player_times(&before) }.is_err();
    restore(&vars, &saved);
    if refused && !refusal_expected() {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(),
            &|| {
                restore(&vars, &saved);
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
