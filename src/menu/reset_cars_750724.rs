//! `menu::reset_cars_750724` (0x46FC52): four entries of `off_750724`'s table (0x5C apart
//! from +0x118) given +0x14 0, +0x48 -0x3E4 and +0x1C -0x57 (a reading: the cars on the
//! front end's carousel at their start).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! They are the in-game options menu's items 3 .. 6 (`menu/in_game_options_items.rs`):
//! this hides all four, as `menu::hide_in_game_options` hides the last ones.
//! Callers: `sub_487E2C`, the event drivers. Self-test: the original and the Rust version
//! on the game's table; put back.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::in_game_options_items::set;
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use rc2k_core::menu::items::{HIDDEN, IN_GAME_OPTIONS};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::reset_cars_750724", rva(0x46FC52), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TABLE_RVA: usize = rva(0x750724); // off_750724
const EXTRA: [usize; 0] = [];
const CASES: u32 = 2;

fn test_entry() -> Regs {
    Regs::SENTINEL
}

fn refusal_expected() -> bool {
    false
}

fn prepare(case: u32) {
    let _ = case;
    for n in 0..4u32 {
        put(
            game_addr(TABLE_RVA) as u32 + 0x118 + 0x5C * n + 0x14,
            case + n,
        );
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_cars_750724(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::items::HIDDEN` (the four items `hide_in_game_options`
    // hides, all of them).
    for n in IN_GAME_OPTIONS {
        unsafe { set(n, HIDDEN) };
    }
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![(game_addr(TABLE_RVA) as u32 + 0x118, 0x5C * 4)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_cars_750724(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
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
    let before = test_entry();
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { reset_cars_750724(&before) }.is_err();
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
