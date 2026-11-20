//! The race paused until a key. Five keys taken, the race's sounds stopped (`race::stop_race_sounds`),
//! `scene_switch` 1, the two first blocks' notice 0xCB (`notice_id`, `notice_time_b` 0, `notice_time` -1); the
//! wait `pause_wait` 30, held at 10 or more while the window is not active; each turn a key
//! taken and the race's frame drawn (`drive::race_frame`, stream A's: through the game); with the
//! window active, `input::update` and, the wait over, `race::race_keys` and any key down
//! but Ctrl, Alt, Tab, F1 .. F10 and F11 ends it. Then the notices cleared, `scene_switch`
//! 0 and the sounds woken (`race::wake_car_sounds`), a key taken, the first block's `input_keys`
//! 0.
//!
//! Refuses what its calls refuse.

use crate::drive::race_parts::{
    input_update, race_frame, race_keys, stop_race_sounds, take_key, wake_car_sounds,
};
use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::drive::race_ticks;

/// Whether a key ends the pause: any of 1 .. 0x7F but 0x1D, 0x38, 0x0F, 0x3B .. 0x44, 0x57.
fn any_ending_key(input: &crate::input::owner::Input) -> u32 {
    (1..0x80usize)
        .filter(|&k| race_ticks::ends_pause(k))
        .fold(0u32, |o, k| o | input.devices.keys_pressed.get(k) as u32)
}

pub fn pause_until_key(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (input, system) = (o.input, o.system);
    let shared = &race.shared;
    let pad_at_start = input.pad_marker();
    for _ in 0..5 {
        take_key(input);
    }
    stop_race_sounds(o)?;
    shared.vars.get().scene_switch.set(1);
    for c in 0..2 {
        race.car_blocks().cars[c as usize].notice_id.set(0xCB);
        race.car_blocks().cars[c as usize].notice_time_b.set(0);
        race.car_blocks().cars[c as usize].notice_time.set(u32::MAX);
    }
    shared.vars.get().pause_wait.set(0x1E);
    // Entered from the wait held, or from a turn.
    let mut hold = true;
    loop {
        if hold {
            let wait = shared.vars.get().pause_wait.get();
            if race_ticks::held_wait(wait) != wait {
                shared
                    .vars
                    .get()
                    .pause_wait
                    .set(race_ticks::held_wait(wait));
            }
        }
        take_key(input);
        race_frame(race, o)?;
        if system.session.flags.window_active.get() == 0 {
            hold = true;
            continue;
        }
        hold = false;
        input_update(o);
        if shared.vars.get().pause_wait.get() != 0 {
            shared
                .vars
                .get()
                .pause_wait
                .set(shared.vars.get().pause_wait.get() - 1);
            continue;
        }
        race_keys(race, o)?;
        if any_ending_key(input) != 0 || input.pad_pressed_since(pad_at_start) {
            break;
        }
    }
    for c in 0..2 {
        race.car_blocks().cars[c as usize].notice_shown.set(0);
        race.car_blocks().cars[c as usize].notice_id.set(0);
        race.car_blocks().cars[c as usize].notice_last.set(0);
    }
    shared.vars.get().scene_switch.set(0);
    wake_car_sounds(race, o.sound, o.audio, &o.system.session.picture_clock)?;
    take_key(input);
    race.car_blocks().cars[0].input_keys.set(0);
    Ok(())
}
