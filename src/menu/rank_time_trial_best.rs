//! `menu::rank_time_trial_best` (0x48CC51): the tenth of the chosen stage's best times
//! moved up to its place: from the tenth to the second, each entry whose +0x14 (the time)
//! is below (unsigned) the one before swaps with it, field by field: +0xC, +0x14, +4, +8,
//! +0x1C, +0x10, +0x18, +0x40, +0x50, +0x54, +0x58, the dwords +0x5C .. +0x98, and the 31
//! name bytes at +0x20 (through `g_keptPlayerName`, which keeps the last swap's upper
//! name). The pass does not stop at the first entry in order.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `menu::enter_time_trial_best`. Refuses (before anything) what
//! `event_flow::stage_times` refuses. Self-test: the game's tables for each stage choice
//! there with test times (in order, reversed, equal, by lot), put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::event_flow::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::rank_time_trial_best",
    rva(0x48CC51),
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

pub const KEPT_NAME: usize = 0x755884; // g_keptPlayerName
const NAME_LEN: u32 = 0x1F;
const SWAPPED: [u32; 11] = [0xC, 0x14, 4, 8, 0x1C, 0x10, 0x18, 0x40, 0x50, 0x54, 0x58];

pub fn rank_time_trial_best() -> Result<(), Refused> {
    let times = stage_times()?;
    let kept = crate::framework::game_addr(rva(KEPT_NAME)) as u32;
    let mut lower = times + 9 * TIME_ENTRY;
    for _ in 0..9 {
        let upper = lower - TIME_ENTRY;
        if dword(lower + 0x14) < dword(upper + 0x14) {
            for o in SWAPPED.into_iter().chain((0x5C..=0x98).step_by(4)) {
                let (a, b) = (dword(upper + o), dword(lower + o));
                put(lower + o, a);
                put(upper + o, b);
            }
            unsafe {
                let byte = |p: u32| *(p as *const u8);
                for k in 0..NAME_LEN {
                    *((kept + k) as *mut u8) = byte(upper + 0x20 + k);
                }
                for k in 0..NAME_LEN {
                    *((upper + 0x20 + k) as *mut u8) = byte(lower + 0x20 + k);
                }
                for k in 0..NAME_LEN {
                    *((lower + 0x20 + k) as *mut u8) = byte(kept + k);
                }
            }
        }
        lower = upper;
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    rank_time_trial_best()?;
    Ok(Left::nothing())
}

fn regions() -> Vec<Region> {
    let mut regions = stage_regions().unwrap_or_default();
    regions.push((
        crate::framework::game_addr(rva(KEPT_NAME)) as u32,
        NAME_LEN as usize,
    ));
    regions
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
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
        let regions = regions();
        if regions.len() != 3 {
            continue;
        }
        let saved = capture(&regions);
        let times = regions[1].0;
        for case in 0..5u32 {
            check_state(
                &mut t,
                &format!("stage {c:x?}, times case {case}"),
                call_original,
                Regs::SENTINEL,
                &|| {
                    restore(&regions, &saved);
                    choose_stage(c);
                    for k in 0..10u32 {
                        let e = times + k * TIME_ENTRY;
                        for o in (0..TIME_ENTRY).step_by(4) {
                            put(e + o, 0x4100_0000 + k * 0x1000 + o + case);
                        }
                        let time = match case {
                            0 => 1000 + k,
                            1 => 1000 - k,
                            2 => 500,
                            3 => {
                                if k == 9 {
                                    0
                                } else {
                                    1000 + k
                                }
                            }
                            _ => k.wrapping_mul(0x9E37_79B9) >> 20,
                        };
                        put(e + 0x14, time);
                    }
                },
                &regions,
                &rust,
            );
        }
        restore(&regions, &saved);
    }
    restore(&stage, &saved_stage);
    t.finish()
}
