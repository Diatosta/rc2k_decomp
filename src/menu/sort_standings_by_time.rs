//! `menu::sort_standings_by_time` (0x47B2C4): standings table A's first 0xB4 entries sorted by their +0x14 (a reading: the stage time), as `menu::standings::sort_table_a`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_473126`, `sub_4A1EE6`. Refuses a null table, before anything. Self-test: the original and the Rust version on the game's tables, keys by lot; put
//! back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::sort_standings_by_time",
    rva(0x47B2C4),
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

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn sort_standings_by_time(entry: &Regs) -> Result<Regs, Refused> {
    sort_table_a(0x14)?;
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { sort_standings_by_time(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &sort_regions(), &rust);
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
    let regions = sort_regions();
    if regions[0].0 == 0 {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.push(at(CURRENT_RVA));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { sort_standings_by_time(&Regs::SENTINEL) }.is_err();
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
                let first: u32 = unsafe { game_read(TABLE_A_RVA) };
                for n in 0..0xB4u32 {
                    put(first + A_ENTRY * n + 0x14, (n * 0x9E37 + case * 0x51) % 97);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
