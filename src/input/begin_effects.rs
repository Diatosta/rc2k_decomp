//! `input::begin_effects` (0x40F29D): the race's force feedback begun: if any sound object
//! is in use (`g_raceSoundObjectCount`, a race is on) both records' effects asked to start
//! (`input::start_effects`); the clock (`system::tick_count`) kept as the effects' last
//! step (`g_effectsLastTime`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `input/effects.rs`. One caller, `input::device_command` (command 3, the race's
//! start). Its calls through `part`. Shadow mode runs the original alone. Self-test: a
//! copy whose two variables are the test's and whose calls go to the noting stand-ins
//! (the clock the test's), against the Rust version, with a count of 0 and not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::input::effects::{LAST_TIME, Part, START, TICK, compare, dispatch_effects};
use crate::sound::turn_parts::{COUNT, TEST_TICK};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::begin_effects", rva(0x40F29D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the variables at `count` and `last` (addresses).
pub fn begin_effects_with(r: &mut Regs, count: u32, last: u32, part: Part) -> Result<(), Refused> {
    let mut g = *r;
    unsafe {
        if (count as *const u32).read_volatile() != 0 {
            part(START, &mut g)?;
        }
        part(TICK, &mut g)?;
        (last as *mut u32).write_volatile(g.eax);
    }
    Ok(())
}

/// The Rust version.
pub fn begin_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    begin_effects_with(r, a(COUNT), a(LAST_TIME), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effects(&HOOK, r, call_original, begin_effects);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = [0u32; 2]; // the count, the last time
    let (count, last) = (vars.as_mut_ptr() as u32, vars.as_mut_ptr() as u32 + 4);
    // pusha; cmp g_raceSoundObjectCount, 0
    let entry = [0x60, 0x83, 0x3D, 0x7C, 0x4F, 0x51, 0x00, 0x00];
    let copy = match copy_with(
        0x40F29D,
        0x1B,
        &entry,
        &[0xA, 0xF],
        &[START, TICK],
        &[],
        &[(COUNT, 4, count), (LAST_TIME, 4, last)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(count, 8usize)];
    for n in [0u32, 1, 3] {
        TEST_TICK.store(0x1234_5000 + n, Ordering::SeqCst);
        compare(
            &mut t,
            &format!("count {n}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe { (count as *mut [u32; 2]).write_volatile([n, 0xAAAA_AAAA]) },
            &all,
            &|r, part| begin_effects_with(r, count, last, part),
            false,
        );
    }
    let _ = &mut vars;
    t.finish()
}
