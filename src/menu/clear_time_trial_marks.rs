//! `menu::clear_time_trial_marks` (0x48C85C): the +0x40 of each of the chosen stage's ten
//! best times set to 0 (the mark of a new entry, set by `menu::enter_time_trial_best`).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Callers: the time trial's events. Refuses (before anything) what
//! `event_flow::stage_times` refuses. Self-test: the game's tables for each stage choice
//! there, the marks filled first, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::event_flow::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::clear_time_trial_marks",
    rva(0x48C85C),
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

pub fn clear_time_trial_marks() -> Result<(), Refused> {
    let times = stage_times()?;
    for k in 0..10 {
        put(times + k * TIME_ENTRY + 0x40, 0);
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    clear_time_trial_marks()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = stage_regions().unwrap_or_default();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let stage = [STAGE_KIND, STAGE_SUB, STAGE_NUMBER].map(|v| at(rva(v)));
    let saved_stage = capture(&stage);
    let choices = stage_choices();
    if choices.is_empty() {
        return t.skip("no stage tables");
    }
    for c in choices {
        choose_stage(c);
        let Some(regions) = stage_regions() else {
            continue;
        };
        let saved = capture(&regions);
        check_state(
            &mut t,
            &format!("stage {c:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| {
                restore(&regions, &saved);
                choose_stage(c);
                let times = regions[1].0;
                for k in 0..10 {
                    put(times + k * TIME_ENTRY + 0x40, 0x5EED_0000 + k);
                }
            },
            &regions,
            &rust,
        );
        restore(&regions, &saved);
    }
    restore(&stage, &saved_stage);
    t.finish()
}
