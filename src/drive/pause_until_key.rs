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
//! Caller: `drive::race_loop`. Refuses what its calls refuse.

use crate::drive::race_parts::{
    input_update, race_frame, race_keys, stop_race_sounds, take_key, wake_car_sounds,
};
use crate::errors::Refused;
use crate::input::owner::input;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::race;
use crate::system::owner::system;
use rc2k_core::drive::race_ticks;

/// Whether a key ends the pause: any of 1 .. 0x7F but 0x1D, 0x38, 0x0F, 0x3B .. 0x44, 0x57.
fn any_ending_key() -> u32 {
    (1..0x80usize)
        .filter(|&k| race_ticks::ends_pause(k))
        .fold(0u32, |o, k| o | input().devices.keys_pressed.get(k) as u32)
}

/// The Rust version.
// Converted: `rc2k_core::drive::race_ticks::{ends_pause, held_wait}`.
/// The Rust version. The registers it leaves are read by no caller (liveness: nothing
/// live after `race_loop`'s call).
pub fn pause_until_key() -> Result<(), Refused> {
    let race = race();
    race.shared.vars.hold_count.set(0x19);
    for _ in 0..5 {
        take_key();
    }
    stop_race_sounds()?;
    race.shared.vars.scene_switch.set(1);
    for c in 0..2 {
        CAR_BLOCKS.cars[c as usize].notice_id.set_bits(0xCB);
        CAR_BLOCKS.cars[c as usize].notice_time_b.set_bits(0);
        CAR_BLOCKS.cars[c as usize].notice_time.set_bits(u32::MAX);
    }
    race.shared.vars.pause_wait.set(0x1E);
    // From 0x4453EE (the wait held) or 0x445401 (a turn).
    let mut hold = true;
    loop {
        if hold {
            let wait = race.shared.vars.pause_wait.get();
            if race_ticks::held_wait(wait) != wait {
                race.shared.vars.pause_wait.set(race_ticks::held_wait(wait));
            }
        }
        take_key();
        race_frame()?;
        if system().session.flags.window_active.get() == 0 {
            hold = true;
            continue;
        }
        hold = false;
        input_update();
        if race.shared.vars.pause_wait.get() != 0 {
            race.shared
                .vars
                .pause_wait
                .set(race.shared.vars.pause_wait.get() - 1);
            continue;
        }
        race_keys()?;
        if any_ending_key() != 0 {
            break;
        }
    }
    for c in 0..2 {
        CAR_BLOCKS.cars[c as usize].notice_shown.set_bits(0);
        CAR_BLOCKS.cars[c as usize].notice_id.set_bits(0);
        CAR_BLOCKS.cars[c as usize].notice_last.set_bits(0);
    }
    race.shared.vars.scene_switch.set(0);
    wake_car_sounds()?;
    take_key();
    CAR_BLOCKS.cars[0].input_keys.set_bits(0);
    Ok(())
}
