//! `drive::play_inputs` (0x460162): for cars 0 and 1 the input words from the replay's tape: frame
//! `step_frame` (16 bytes; the car's 8) has the first word and the packed rest, taken apart
//! (`drive::unpack_inputs`) into `input_words` ...
//!
//! Callers: `drive::play_inputs_if`, `race::step_replay_cars`.

use crate::errors::BadInput;
use crate::pointers::Store;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};

// Converted: `rc2k_core::drive::replay_frames::{on_tape, tape_offset}`.
pub fn play_inputs(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    let replay = &race.replay;
    let car = shared.work.current_car.get();
    if !on_tape(car) {
        return Ok(());
    }
    let esi = tape_offset(replay.replay_frames.step_frame.get(), car);
    let mut words = [0u8; 8];
    race.replay_work()
        .replay_tape
        .load(esi as usize, &mut words)?;
    let ebx = u32::from_le_bytes([words[0], words[1], words[2], words[3]]);
    let second = u32::from_le_bytes([words[4], words[5], words[6], words[7]]);
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(second);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        shared.physics_tables.input_words[k].set(*v);
    }
    Ok(())
}
