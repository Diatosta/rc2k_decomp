//! `physics::queue_pace_note` (0x4418B3): the pace notes group called at checkpoint edx
//! (see `race/pace_notes.rs`): with groups (`dword_615E14` not 0), the first group whose
//! checkpoint (`dword_619E18`) is edx; its text (`byte_61BE18` from `dword_61AE18[group]`)
//! code by code up to a NUL: each below 0x54 cued (`physics::cue_listed_car_sound_last`,
//!) and queued as a car event (`physics::queue_listed_event`,
//! ebx the code); one of 0x54 or more ends it with "!!! invalid pace data", "data", the
//! code, "ecx", the group, "pointer", its place logged (level 1, each in `pusha` /
//! `popa`).
//!
//! Takes: the checkpoint
//!
//! Caller: `physics::pace_notes_step`. Its calls by place (this
//! file's `pace_part`): 0 `logfile::log_write_l1`, 1 `logfile::log_int_l1`, 2
//! `logfile::log_int_l1_nl`, 3 `physics::cue_listed_car_sound_last`, 4
//! `physics::queue_listed_event`, 5 this function, 6 `physics::expire_car_events`. Refuses
//! (before anything) more than 0x400 groups (the scan reads past `dword_619E18`) and a text
//! place past the 0x4000 bytes (read past the text; checked at each code).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::physics::pace_calls::{Code, code, find_group, text_place};

/// "!!! invalid pace data", "data", "ecx", "pointer".
const WORDS: [&std::ffi::CStr; 4] = [c"!!! invalid pace data", c"data", c"ecx", c"pointer"];

/// The Rust version: the pace notes of checkpoint `checkpoint` (edx), each cued
/// (`physics::cue_listed_car_sound_last`) and queued (`physics::queue_listed_event`). The
/// registers it leaves are read by none of its callers.
// Converted: `rc2k_core::physics::pace_calls::{find_group, code, text_place}`.
pub fn queue_pace_note(
    race: &Race,
    checkpoint: u32,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let physics_store = &race.physics;
    use crate::logfile::log_int::log_int;
    let groups = physics_store.pace.group_count.get();
    let found = find_group(groups, checkpoint, |n| {
        physics_store.pace.group_checkpoints[n as usize].get()
    })?;
    if groups == 0 {
        return Ok(());
    }
    let Some(group) = found else {
        return Ok(());
    };
    let mut at = physics_store.pace.group_texts[group as usize].get();
    loop {
        let byte = physics_store.pace.text.get(text_place(at)? as usize) as u32;
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
                    race, byte, sound,
                )?;
                { crate::physics::queue_listed_event::queue_listed_event(race, byte) }?;
                at = at.wrapping_add(1);
            }
        }
    }
}
