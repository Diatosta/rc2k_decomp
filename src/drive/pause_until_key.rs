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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::race_parts::{
    input_update, race_frame, race_keys, stop_race_sounds, take_key, wake_car_sounds,
};
use crate::errors::Refused;
use crate::physics::flow::{a, g, p};
use rc2k_core::drive::race_ticks;

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
/// The Rust version. The registers it leaves are read by no caller (liveness: nothing
/// live after `race_loop`'s call).
pub unsafe fn pause_until_key() -> Result<(), Refused> {
    p(0x6235EC, 0x19);
    for _ in 0..5 {
        take_key();
    }
    stop_race_sounds()?;
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
        take_key();
        race_frame()?;
        if g(0x518374) == 0 {
            hold = true;
            continue;
        }
        hold = false;
        input_update();
        if g(0x6235F0) != 0 {
            p(0x6235F0, g(0x6235F0) - 1);
            continue;
        }
        race_keys()?;
        if any_ending_key() != 0 {
            break;
        }
    }
    for c in 0..2 {
        p(B + 0xFE0 * c + 0x728, 0);
        p(B + 0xFE0 * c + 0x71C, 0);
        p(B + 0xFE0 * c + 0x724, 0);
    }
    p(0x6235E8, 0);
    wake_car_sounds()?;
    take_key();
    p(B + 0x468, 0);
    Ok(())
}
