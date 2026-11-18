//! The current car's input words: 0, then, for a car with a device, its readings
//! (`drive::device_axes`). In a network game the car's block gives them, or, for the car
//! recorded, takes them.
//!
//! Refuses (before anything) a current car past 7, and what `device_axes` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

/// Set while a paused machine's race goes on for the others (`drive::race_loop::
/// step_while_paused`): the own car's words are then full brake and the handbrake, so that it
/// stays where it was paused.
pub static HELD: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The words of a held car: no throttle, the handbrake's key bit, no steering, full brake.
const HELD_WORDS: [u32; 4] = [0, 4, 0, 0x100];

pub fn car_input_words(
    race: &Race,
    input: &crate::input::owner::Input,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    let race_owner = race;
    let frame = &race.frame;
    use rc2k_core::drive::inputs::{NO_DEVICE, ReplayWords, check_car, device_words, replay_words};
    let car = race_owner.shared.work.get().current_car.get();
    check_car(car)?;
    let mut device = frame.frame_work.get().listed_car_sounds[car as usize].get();
    // The third and fourth players of a larger split screen have no sound object or view
    // to list them by: their devices are their own numbers.
    if device == NO_DEVICE
        && !net.race_slot.is_on()
        && (2..race_owner.shared.view.get().players.get()).contains(&car)
    {
        device = car;
    }
    let readings = if device != NO_DEVICE {
        Some(crate::drive::device_axes::device_axes(device, input)?)
    } else {
        None
    };
    let words = device_words(readings);
    for (k, v) in words.iter().enumerate() {
        race_owner.shared.physics_tables.get().input_words[k].set(*v);
    }
    if HELD.load(std::sync::atomic::Ordering::Relaxed)
        && net.race_slot.get().is_some_and(|n| n.player == car)
    {
        for (word, v) in race_owner
            .shared
            .physics_tables
            .get()
            .input_words
            .iter()
            .zip(HELD_WORDS)
        {
            word.set(v);
        }
    }
    if let Some(net_race) = net.race_slot.get() {
        let block = race_owner
            .shared
            .race_car_or(car as usize, "a car's block")?;
        // +0xFD0 .. +0xFDC: the kept throttle, key bits, steering and brake words.
        let kept = [
            &block.kept_throttle_word,
            &block.kept_keys_word,
            &block.kept_steering_word,
            &block.kept_brake_word,
        ];
        for (k, at) in kept.into_iter().enumerate() {
            match replay_words(car, net_race.player) {
                ReplayWords::FromBlock => {
                    race_owner.shared.physics_tables.get().input_words[k].set(at.get());
                }
                ReplayWords::IntoBlock => {
                    at.set(race_owner.shared.physics_tables.get().input_words[k].get());
                }
            }
        }
    }
    Ok(())
}
