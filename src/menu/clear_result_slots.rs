//! `menu::clear_result_slots` (0x470DDA): the four players' fourteen result slots
//! (+0x98 .. +0xCC) and every driver's (+0x5C .. +0x90) emptied: -1, but 0 in the seventh
//! and the last (a reading: two groups of six places and a count).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a missing player or a drivers' table without its
//! end entry, before anything. Self-test: the original and the Rust version on the game's
//! tables; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::clear_result_slots", rva(0x470DDA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EXTRA: [usize; 0] = [];
const CASES: u32 = 2;

fn prepare(case: u32) {
    let _ = case;
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_result_slots(entry: &Regs) -> Result<Regs, Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    let mut drivers = driver_table()?;
    // Converted: `rc2k_core::menu::event::result_slots`.
    use rc2k_core::menu::event::result_slots;
    for n in 0..4 {
        put_all(player(n), &result_slots(0x98));
    }
    drivers.store_all(&result_slots(0x5C));
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    let mut v: Vec<Region> = (0..4u32).map(|n| (player(n), 0x400)).collect();
    let first = game_addr(DRIVERS_RVA) as u32;
    match entries(first, DRIVER) {
        Ok(all) => v.push((first, all.len() * DRIVER as usize)),
        Err(_) => v.push((0, 0)),
    }
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_result_slots(r) }?))
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
    let before = Regs::SENTINEL;
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { clear_result_slots(&before) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            before,
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
