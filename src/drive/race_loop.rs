//! `drive::race_loop` (0x444D46): the race played until it ends. The eight blocks' +0xFAC
//! 10, `dword_518398` (paused) and `dword_6235E8` 0; the clock (`scene::restart_clock`)
//! into `dword_5CC3B4`, `5CC3B0`; 0x20 dwords at `unk_5CC1B0` and `dword_5C2538` -1. Then,
//! with the tenth's base `dword_613840` the clock each time round: a window not active
//! pauses (`dword_518398` 1); unpaused, `input::update`; in the attract mode
//! (`dword_71C3E0`) a key (`drive::any_key_down`) or `dword_529854` ends it; key 1 (Esc)
//! asks to quit (`drive::quit_question`: no goes on, else the answer into the cars' +0xB0
//! (a replay's car's only), with two cars a finished car's 2) and ends it; the views' bit
//! 0x40 (+0x468, `g_viewBlocks`; the second with two views) while not holding
//! (`dword_6235EC` 0) pauses; else `race::race_keys`, `drive::race_step`,
//! `race::tyre_change`, all out (`race::players_all_out`) ends it, and the race's frame
//! (`sub_4541BF`, stream A's: through the game). Paused: in a replay its car's +0x468 0,
//! else `drive::pause_until_key`; unpaused again.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read
//!
//! Caller: `drive::play_race`. Ungated: the race's step loop.
//! Refuses what its calls refuse. Shadow mode runs the original alone.
//! Self-test: skipped (it plays a race).

use crate::drive::race_parts::{
    input_update, players_all_out, race_frame, race_keys, restart_clock, tyre_change,
};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::physics::flow::{a, g, p};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::race_loop", rva(0x444D46), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const B: usize = 0x70F3EC;
const NEXT: usize = 0xFE0;
const PAUSED: usize = 0x518398;

fn view_bit(view: usize) -> bool {
    let bits = unsafe { ((g(view) + 0x468) as *const u32).read_unaligned() };
    bits & 0x40 != 0
}

/// The quit answer put into the cars (0x444EB5 ..).
fn answer(r: &Regs) {
    if g(0x52A910) != 0 {
        let block = g(0x71B3B8 + 4 * g(0x52A920) as usize);
        unsafe { ((block + 0xB0) as *mut u32).write_unaligned(r.eax) };
        return;
    }
    p(B + 0xB0, r.eax);
    p(B + NEXT + 0xB0, r.eax);
    if g(0x63DB94) != 2 {
        return;
    }
    if g(B + 0xA8) != 0 {
        p(B + 0xB0, 2);
    }
    if g(B + NEXT + 0xA8) != 0 {
        p(B + NEXT + 0xB0, 2);
    }
}

/// The Rust version.
pub unsafe fn race_loop(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    for c in 0..8 {
        p(B + NEXT * c + 0xFAC, 10);
    }
    p(PAUSED, 0);
    p(0x6235E8, 0);
    restart_clock(&mut r);
    r.eax = g(0x613830);
    p(0x5CC3B4, r.eax);
    p(0x5CC3B0, r.eax);
    for k in 0..0x20 {
        p(0x5CC1B0 + 4 * k, u32::MAX);
    }
    r.eax = u32::MAX;
    r.ecx = 0;
    r.edi = a(0x5CC1B0) + 0x80;
    p(0x5C2538, u32::MAX);
    loop {
        restart_clock(&mut r);
        r.eax = g(0x613830);
        p(0x613840, r.eax);
        // 0x444DE9: a frame.
        let paused = loop {
            if g(0x518374) == 0 {
                p(PAUSED, 1);
            }
            if g(PAUSED) != 0 {
                break true;
            }
            input_update(&mut r);
            if g(0x71C3E0) != 0 {
                if g(0x529854) != 0 {
                    return Ok(left_of(&r));
                }
                let (or, esi) = unsafe { crate::drive::any_key_down::any_key_down() };
                r.eax = or;
                r.ecx = 0x80;
                r.esi = esi;
                if or != 0 {
                    return Ok(left_of(&r));
                }
            }
            if unsafe { (a(0x4F6CDD) as *const u8).read() } != 0 {
                unsafe { crate::drive::quit_question::quit_question(&r) }?.apply(&mut r);
                if r.eax == 0 {
                    break false;
                }
                answer(&r);
                return Ok(left_of(&r));
            }
            if g(0x6235EC) == 0 {
                r.ebx = g(0x63DBB0);
                if view_bit(0x63DBB0) {
                    break true;
                }
                if g(0x63DB98) != 1 {
                    r.ebx = g(0x63DBB4);
                    if view_bit(0x63DBB4) {
                        break true;
                    }
                }
            }
            if g(PAUSED) != 0 {
                break true;
            }
            race_keys(&mut r)?;
            unsafe { crate::drive::race_step::race_step(&r) }?.apply(&mut r);
            tyre_change(&mut r)?;
            if players_all_out(&mut r)? {
                return Ok(left_of(&r));
            }
            race_frame(&mut r);
        };
        if !paused {
            continue;
        }
        // 0x444F0F: paused.
        if g(0x52A910) != 0 {
            let block = g(0x71B3B8 + 4 * g(0x52A920) as usize);
            unsafe { ((block + 0x468) as *mut u32).write_unaligned(0) };
        } else {
            unsafe { crate::drive::pause_until_key::pause_until_key(&r) }?.apply(&mut r);
        }
        p(PAUSED, 0);
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { race_loop(r) } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    let Some(t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    t.skip("it plays a race")
}
