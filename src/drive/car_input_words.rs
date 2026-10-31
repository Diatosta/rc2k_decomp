//! `drive::car_input_words` (0x43A27E): the current car's input words (`dword_6138C4` ..
//! `6138D0`): 0, then, for a car with a device (`dword_641680[g_currentCar]` not -1), its
//! readings (`drive::device_axes`). With a replay on (`dword_52A910`): the car's block
//! (`g_carBlocks[g_currentCar]`) +0xFD0 .. gives them, or, for the car recorded
//! (`dword_52A920`), takes them.
//!
//! In:  nothing
//! Out: eax .. edx the readings (eax the car with a replay on), esi the device or the
//!      block; the rest kept; no flags read
//!
//! Callers: `drive::car_controls` (`sub_43A1CA`), `sub_460835`, `sub_460B2F`. Refuses
//! (before anything) a current car past 7, and what `device_axes` refuses.

use crate::errors::Refused;
use crate::physics::tables_store::PHYSICS;
use crate::pointers::GamePtr;
use crate::race::frame_store::FRAME_WORK;
use crate::race::work_store::RACE_WORK;

/// The Rust version.
// Converted: `rc2k_core::drive::inputs::{check_car, device_words, replay_words}`.
pub fn car_input_words() -> Result<(), Refused> {
    use rc2k_core::drive::inputs::{NO_DEVICE, ReplayWords, check_car, device_words, replay_words};
    let car = RACE_WORK.current_car.get();
    check_car(car)?;
    let device = FRAME_WORK.listed_car_sounds[car as usize].get();
    let readings = if device != NO_DEVICE {
        Some(crate::drive::device_axes::device_axes(device)?)
    } else {
        None
    };
    let words = device_words(readings);
    for (k, v) in words.iter().enumerate() {
        PHYSICS.input_words[k].set(*v);
    }
    if crate::net::store::NET.session.get() != 0 {
        let block = RACE_WORK.car_block(car as usize).wrapping_add(0xFD0);
        for k in 0..4u32 {
            let at = GamePtr::new(block.wrapping_add(4 * k));
            match replay_words(car, crate::net::store::NET.player.get()) {
                ReplayWords::FromBlock => {
                    PHYSICS.input_words[k as usize].set(at.read::<u32>("a car's block")?);
                }
                ReplayWords::IntoBlock => {
                    at.write(PHYSICS.input_words[k as usize].get(), "a car's block")?;
                }
            }
        }
    }
    Ok(())
}
