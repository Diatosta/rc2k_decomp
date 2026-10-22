//! `menu::clear_record_entries` (0x47202F): the current record's event (`dword_7556D8`,
//! 0x618 an event, by the record's +0x3C) has its ten entries' (0x9C apart) +0x40 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A3BE0`. `records::find_record`'s Rust version. Refuses a record past the
//! table or no events table, before anything. Self-test: the original and the Rust version
//! on the game's tables; put back.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::clear_record_entries",
    rva(0x47202F),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

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
pub unsafe fn clear_record_entries(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::bests::clear_marks`.
    let event = current_event()?;
    rc2k_core::menu::bests::clear_marks(&mut crate::menu::rank_record_entry::event_list(event));
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    match current_event() {
        Ok(e) => vec![(e, 0x9C * 10)],
        Err(_) => vec![(0, 0)],
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_record_entries(r) }?))
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
    let refused = unsafe { clear_record_entries(&before) }.is_err();
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
