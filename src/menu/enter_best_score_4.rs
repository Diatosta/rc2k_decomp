//! `menu::enter_best_score_4` (0x4A871C): as `menu::enter_best_score` with the score +0xCC into group 4.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A91BF`. Refuses as `enter_best_score`. Self-test: the original and the Rust version on the game's
//! tables, scores by lot; put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::enter_best_score_4", rva(0x4A871C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn enter_best_score_4(entry: &Regs) -> Result<Regs, Refused> {
    enter_best(Some(4), 0xCC)?;
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let mut v = rank_regions();
    v.extend(
        (0..4u32)
            .map(|n| (player(n), 0x400))
            .filter(|&(p, _)| p != 0),
    );
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { enter_best_score_4(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.push(at(rva(0x75598C)));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { enter_best_score_4(&Regs::SENTINEL) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..3u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&vars, &saved);
                let n: u32 = unsafe { game_read(rva(0x75598C)) };
                if n < 4 && player(n) != 0 {
                    put(player(n) + 0xCC, 0x20 + case * 7);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
