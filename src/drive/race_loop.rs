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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::race_parts::{
    input_update, players_all_out, race_frame, race_keys, restart_clock, tyre_change,
};
use crate::errors::Refused;
use crate::physics::flow::{a, g, p};
use crate::session::FLAGS;

const B: usize = 0x70F3EC;
const NEXT: usize = 0xFE0;

fn view_bit(view: usize) -> bool {
    let bits = unsafe { ((g(view) + 0x468) as *const u32).read_unaligned() };
    bits & 0x40 != 0
}

/// The quit answer `answer` put into the cars (0x444EB5 ..).
fn put_answer(answer: u32) {
    if g(0x52A910) != 0 {
        let block = g(0x71B3B8 + 4 * g(0x52A920) as usize);
        unsafe { ((block + 0xB0) as *mut u32).write_unaligned(answer) };
        return;
    }
    p(B + 0xB0, answer);
    p(B + NEXT + 0xB0, answer);
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

/// The Rust version. The registers it leaves are read by no caller (liveness: ecx, edi,
/// edx and esi after `drive::play_race`'s call have no read point).
pub unsafe fn race_loop() -> Result<(), Refused> {
    for c in 0..8 {
        p(B + NEXT * c + 0xFAC, 10);
    }
    FLAGS.paused.set(0);
    p(0x6235E8, 0);
    restart_clock();
    p(0x5CC3B4, g(0x613830));
    p(0x5CC3B0, g(0x613830));
    for k in 0..0x20 {
        p(0x5CC1B0 + 4 * k, u32::MAX);
    }
    p(0x5C2538, u32::MAX);
    loop {
        restart_clock();
        p(0x613840, g(0x613830));
        // 0x444DE9: a frame.
        let paused = loop {
            if FLAGS.window_active.get() == 0 {
                FLAGS.paused.set(1);
            }
            if FLAGS.paused.get() != 0 {
                break true;
            }
            input_update();
            if g(0x71C3E0) != 0 {
                if g(0x529854) != 0 {
                    crate::framework::log("drive::race_loop: the credits race ends: rolled off");
                    return Ok(());
                }
                let (or, pressed) = unsafe { crate::drive::any_key_down::any_key_down() };
                if or != 0 {
                    // Which keys (`g_keysPressed`, by scan code), and the window's own state
                    // of them (`g_keysDown`), for the credits race that ends at once after
                    // the intro (the owner, 2026-10-24).
                    let keys = |at: u32| {
                        (0..0x80u32)
                            .filter(|n| unsafe { ((at + n) as *const u8).read() } != 0)
                            .map(|n| format!("{n:#x}"))
                            .collect::<Vec<_>>()
                            .join(" ")
                    };
                    crate::framework::log(&format!(
                        "drive::race_loop: the credits race ends: keys pressed [{}], down [{}], \
                         frame {}",
                        keys(pressed),
                        keys(a(crate::input::store::DeviceSlot::at(0x4F6ADC))),
                        g(0x613830)
                    ));
                    return Ok(());
                }
            }
            if unsafe { (a(crate::input::store::DeviceSlot::at(0x4F6CDD)) as *const u8).read() }
                != 0
            {
                let answer = unsafe { crate::drive::quit_question::quit_question() }?;
                if answer == 0 {
                    break false;
                }
                put_answer(answer);
                return Ok(());
            }
            if g(0x6235EC) == 0 {
                if view_bit(0x63DBB0) {
                    break true;
                }
                if g(0x63DB98) != 1 && view_bit(0x63DBB4) {
                    break true;
                }
            }
            if FLAGS.paused.get() != 0 {
                break true;
            }
            race_keys()?;
            unsafe { crate::drive::race_step::race_step() }?;
            tyre_change()?;
            if players_all_out()? {
                return Ok(());
            }
            race_frame()?;
        };
        if !paused {
            continue;
        }
        // 0x444F0F: paused.
        if g(0x52A910) != 0 {
            let block = g(0x71B3B8 + 4 * g(0x52A920) as usize);
            unsafe { ((block + 0x468) as *mut u32).write_unaligned(0) };
        } else {
            unsafe { crate::drive::pause_until_key::pause_until_key() }?;
        }
        FLAGS.paused.set(0);
    }
}
