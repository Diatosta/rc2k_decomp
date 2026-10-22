//! `menu::rank_best_score` (0x4A8799): the last best score of the current player's group (`dword_75598C`'s record +0x90) moved up its ten, as `menu::standings::rank_in_group`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A869E`. Refuses a current player past 3 or missing, a null table or a group past 4, before anything. Self-test: the original and the Rust version on the game's tables, scores by lot; put
//! back.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::rank_best_score", rva(0x4A8799), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn rank_best_score(entry: &Regs) -> Result<Regs, Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    rank_in_group(get(player(n) + 0x90))?;
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { rank_best_score(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &rank_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = rank_regions();
    if regions[0].0 == 0 {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.push(at(CURRENT_RVA));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { rank_best_score(&Regs::SENTINEL) }.is_err();
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
                let table: u32 = unsafe { game_read(BEST_SCORES_RVA) };
                for n in 0..50u32 {
                    put(table + 0x3C * n + 0x38, (n * 0x9E37 + case * 0x51) % 13);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
