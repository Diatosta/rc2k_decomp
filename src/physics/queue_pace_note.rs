//! Calls the pace notes group of a checkpoint (see `race/pace_notes.rs`): with groups, the first
//! group whose checkpoint is the given one; its text is read code by code up to a NUL: each code
//! below 0x54 is cued (`physics::cue_listed_car_sound_last`) and queued as a car event
//! (`physics::queue_listed_event`); a code of 0x54 or more ends it with the log lines "!!! invalid
//! pace data", "data", the code, "ecx", the group, "pointer", its place (level 1).
//!
//! Refuses (before anything) more than 0x400 groups (the scan reads past the group checkpoints) and
//! a text place past the 0x4000 bytes (checked at each code).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::tables_store::PhysicsTables;
use crate::race::frame_store::FrameWork;
use crate::race::owner_physics::Physics;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::pace_calls::{Code, code, find_group, text_place};

/// The log words: "!!! invalid pace data", "data", "ecx", "pointer".
const WORDS: [&std::ffi::CStr; 4] = [c"!!! invalid pace data", c"data", c"ecx", c"pointer"];

/// The pace notes of checkpoint `checkpoint`, each cued (`physics::cue_listed_car_sound_last`) and
/// queued (`physics::queue_listed_event`).
// Core: `rc2k_core::physics::pace_calls::{find_group, code, text_place}`.
#[allow(
    clippy::too_many_arguments,
    reason = "the race's tables the note reads, and the owners"
)]
pub fn queue_pace_note(
    physics: &Physics,
    frame_work: &FrameWork,
    work: &RaceWork,
    physics_tables: &PhysicsTables,
    car: &Car,
    checkpoint: u32,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let physics_store = physics;
    use crate::logfile::log_int::log_int;
    let groups = physics_store.pace().group_count.get();
    let found = find_group(groups, checkpoint, |n| {
        physics_store.pace().group_checkpoints[n as usize].get()
    })?;
    if groups == 0 {
        return Ok(());
    }
    let Some(group) = found else {
        return Ok(());
    };
    let mut at = physics_store.pace().group_texts[group as usize].get();
    loop {
        let byte = physics_store.pace().text.get(text_place(at)? as usize) as u32;
        match code(byte as u8) {
            Code::End => return Ok(()),
            Code::Bad => {
                let word = |n: usize| &WORDS[n];
                crate::logfile::log_write::log_nul(word(0), false, 1);
                crate::logfile::log_write::log_nul(word(1), false, 1);
                log_int(byte as i32, false, 1);
                crate::logfile::log_write::log_nul(word(2), false, 1);
                log_int(group as i32, true, 1);
                crate::logfile::log_write::log_nul(word(3), false, 1);
                log_int(at as i32, true, 1);

                return Ok(());
            }
            Code::Call => {
                crate::physics::cue_listed_car_sound_last::cue_listed_car_sound_last(
                    frame_work, work, byte, sound, audio,
                )?;
                {
                    crate::physics::queue_listed_event::queue_listed_event(
                        car,
                        physics_tables,
                        byte,
                    )
                }?;
                at = at.wrapping_add(1);
            }
        }
    }
}
