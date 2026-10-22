//! `drive::pause_until_key` (0x445376): the race paused until a key. The hold count
//! `dword_6235EC` 0x19, five keys taken, the race's sounds stopped (`race::stop_race_sounds`),
//! `dword_6235E8` 1, the two first blocks' notice 0xCB (+0x71C, +0x730 0, +0x72C -1); the
//! wait `dword_6235F0` 30, held at 10 or more while the window is not active; each turn a key
//! taken and the race's frame drawn (`sub_4541BF`, stream A's: through the game); with the
//! window active, `input::update` and, the wait over, `race::race_keys` and any key down
//! but Ctrl, Alt, Tab, F1 .. F10 and F11 ends it. Then the notices cleared, `dword_6235E8`
//! 0 and the sounds woken (`race::wake_car_sounds`), a key taken, the first block's +0x468
//! 0.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read
//!
//! Caller: `drive::race_loop`. Ungated: it waits for the player.
//! Refuses what its calls refuse. Shadow mode runs the original alone.
//! Self-test: skipped (it waits for a key).

use crate::drive::race_parts::{
    input_update, race_frame, race_keys, stop_race_sounds, take_key, wake_car_sounds,
};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use rc2k_core::drive::race_ticks;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::pause_until_key", rva(0x445376), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const B: usize = 0x70F3EC;
const KEYS: usize = 0x4F6CDC; // g_keysPressed

/// Whether a key ends the pause: any of 1 .. 0x7F but 0x1D, 0x38, 0x0F, 0x3B .. 0x44, 0x57.
fn any_ending_key() -> u32 {
    (1..0x80usize)
        .filter(|&k| race_ticks::ends_pause(k))
        .fold(0u32, |o, k| {
            o | unsafe { (a(KEYS + k) as *const u8).read() } as u32
        })
}

/// The Rust version.
// Converted: `rc2k_core::drive::race_ticks::{ends_pause, held_wait}`.
pub unsafe fn pause_until_key(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    p(0x6235EC, 0x19);
    for _ in 0..5 {
        take_key(&mut r);
    }
    stop_race_sounds(&mut r)?;
    p(0x6235E8, 1);
    for c in 0..2 {
        p(B + 0xFE0 * c + 0x71C, 0xCB);
        p(B + 0xFE0 * c + 0x730, 0);
        p(B + 0xFE0 * c + 0x72C, u32::MAX);
    }
    p(0x6235F0, 0x1E);
    // From 0x4453EE (the wait held) or 0x445401 (a turn).
    let mut hold = true;
    loop {
        if hold {
            let wait = g(0x6235F0);
            if race_ticks::held_wait(wait) != wait {
                p(0x6235F0, race_ticks::held_wait(wait));
            }
        }
        take_key(&mut r);
        race_frame(&mut r);
        if g(0x518374) == 0 {
            hold = true;
            continue;
        }
        hold = false;
        input_update(&mut r);
        if g(0x6235F0) != 0 {
            p(0x6235F0, g(0x6235F0) - 1);
            continue;
        }
        race_keys(&mut r)?;
        r.eax = any_ending_key();
        r.esi = a(KEYS);
        r.ecx = 0x80;
        if r.eax != 0 {
            break;
        }
    }
    for c in 0..2 {
        p(B + 0xFE0 * c + 0x728, 0);
        p(B + 0xFE0 * c + 0x71C, 0);
        p(B + 0xFE0 * c + 0x724, 0);
    }
    p(0x6235E8, 0);
    wake_car_sounds(&r)?;
    take_key(&mut r);
    p(B + 0x468, 0);
    Ok(left_of(&r))
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { pause_until_key(r) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { pause_until_key(r) }?;
    left.apply(r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("it waits for a key")
}
