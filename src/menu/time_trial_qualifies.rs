//! `menu::time_trial_qualifies` (0x48C7CE): whether the current player's time makes the
//! chosen stage's best times: `dword_755918` 0; `dword_755914` the player's +0x88 (the
//! time); when that is not above (signed) the tenth entry's +0x14, `dword_755918` and
//! `dword_755920` 1.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: the time trial's events, after the results. Refuses (before anything) a
//! current player past the four or missing, a stage kind past the table or a null pointer
//! to the stage's records or times (`event_flow::stage_times`). Self-test: the game's
//! tables for each stage choice there, the player's time below, equal to and above the
//! tenth.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::event_flow::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::time_trial_qualifies",
    rva(0x48C7CE),
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

pub const QUALIFIES: usize = 0x755918; // dword_755918
pub const TIME: usize = 0x755914; // dword_755914
pub const ANY_QUALIFIED: usize = 0x755920; // dword_755920
/// The tenth entry of the best times.
pub const TENTH: u32 = 9 * TIME_ENTRY;

pub fn time_trial_qualifies() -> Result<(), Refused> {
    let tenth = stage_times()? + TENTH;
    let p = current_player()?;
    set(QUALIFIES, 0);
    let time = dword(p + 0x88);
    set(TIME, time);
    // Converted: `rc2k_core::menu::bests::makes_time`.
    if rc2k_core::menu::bests::makes_time(time, dword(tenth + 0x14)) {
        set(QUALIFIES, 1);
        set(ANY_QUALIFIED, 1);
    }
    Ok(())
}

fn regions() -> Vec<Region> {
    [QUALIFIES, TIME, ANY_QUALIFIED]
        .iter()
        .map(|&v| at(rva(v)))
        .collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    time_trial_qualifies()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(p) = current_player() else {
        return t.skip("no current player");
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend([STAGE_KIND, STAGE_SUB, STAGE_NUMBER].map(|v| at(rva(v))));
    vars.push((p + 0x88, 4));
    let saved = capture(&vars);
    let choices = stage_choices();
    if choices.is_empty() {
        return t.skip("no stage tables");
    }
    for c in choices {
        choose_stage(c);
        let Ok(times) = stage_times() else { continue };
        let tenth = dword(times + TENTH + 0x14);
        for time in [
            tenth.wrapping_sub(1),
            tenth,
            tenth.wrapping_add(1),
            0x8000_0000,
        ] {
            check_state(
                &mut t,
                &format!("stage {c:x?}, tenth {tenth:#x}, time {time:#x}"),
                call_original,
                Regs::SENTINEL,
                &|| {
                    choose_stage(c);
                    put(p + 0x88, time);
                    for v in [QUALIFIES, TIME, ANY_QUALIFIED] {
                        set(v, 0xAAAA_AAAA);
                    }
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&vars, &saved);
    t.finish()
}
