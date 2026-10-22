//! `menu::init_standings_b` (0x4A4644): the 200 entries of standings table B emptied (their
//! first five dwords -1).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A44A0`. Refuses a null table, before anything. Self-test: the original and
//! the Rust version on the game's table; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::init_standings_b", rva(0x4A4644), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn init_standings_b(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    if first == 0 {
        return Err(BadInput("no standings table").into());
    }
    // Converted: `rc2k_core::menu::standings::EMPTY_B`.
    use rc2k_core::menu::standings::{EMPTY_B, ROOM};
    for n in 0..ROOM {
        for (k, v) in EMPTY_B.into_iter().enumerate() {
            put(first + B_ENTRY * n + 4 * k as u32, v);
        }
    }
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![(unsafe { game_read(TABLE_B_RVA) }, (200 * B_ENTRY) as usize)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { init_standings_b(r) }?))
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
    let refused = unsafe { init_standings_b(&before) }.is_err();
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
