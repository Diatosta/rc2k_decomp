//! `menu::sort_standings_b_by_figure` (0x4A467A): standings table B's first 0xB4 entries sorted going up by their +0x34 (a reading: the stage's figure), as
//! `menu::standings::sort_table_b`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: nine of the game's. Refuses a null table, before anything. Self-test: the original and
//! the Rust version on the game's tables, keys by lot; put back.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::sort_standings_b_by_figure",
    rva(0x4A467A),
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
pub unsafe fn sort_standings_b_by_figure(entry: &Regs) -> Result<Regs, Refused> {
    sort_table_b(0x34, false)?;
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { sort_standings_b_by_figure(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &sort_regions_b(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = sort_regions_b();
    if regions[0].0 == 0 {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.push(at(CURRENT_RVA));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { sort_standings_b_by_figure(&Regs::SENTINEL) }.is_err();
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
                let first: u32 = unsafe { game_read(TABLE_B_RVA) };
                for n in 0..0xB4u32 {
                    put(first + B_ENTRY * n + 0x34, (n * 0x9E37 + case * 0x51) % 97);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
