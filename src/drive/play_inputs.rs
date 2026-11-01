//! `drive::play_inputs` (0x460162): for cars 0 and 1 the input words from the replay's
//! tape: frame `dword_6EF7E4` (16 bytes; the car's 8) has ebx and the packed word, taken
//! apart (`drive::unpack_inputs`) into `dword_6138C4` .. (eax, ebx, ecx, edx).
//!
//! In:  nothing
//! Out: eax .. edx the words, esi the tape's offset (cars 0, 1); the rest kept; no flags
//!      read
//!
//! Callers: `drive::play_inputs_if`, `sub_460835`.

use crate::drive::replay_frames_store::REPLAY_FRAMES;
use crate::errors::BadInput;
use crate::physics::tables_store::PHYSICS;
use crate::pointers::GamePtr;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{on_tape, tape_offset}`.
pub fn play_inputs() -> Result<(), BadInput> {
    let car = RACE_WORK.current_car.get();
    if !on_tape(car) {
        return Ok(());
    }
    let esi = tape_offset(REPLAY_FRAMES.step_frame.get(), car);
    let at = |o: u32| GamePtr::to(&REPLAY_WORK.replay_tape).offset(esi).offset(o);
    let [ebx, second]: [u32; 2] = at(0).read("the replay tape")?;
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(second);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        PHYSICS.input_words[k].set(*v);
    }
    Ok(())
}
