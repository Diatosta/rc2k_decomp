//! `drive::record_inputs` (0x4601A8): for cars 0 and 1 the input words onto the replay's
//! tape: ebx and the packed rest (`drive::pack_inputs`) into frame `dword_6EF7DC` (the
//! car's 8 of its 16 bytes), and the words given back as the tape keeps them
//! (`drive::unpack_inputs`).
//!
//! In:  nothing
//! Out: eax .. edx the words, esi the tape's offset (cars 0, 1); the rest kept; no flags
//!      read
//!
//! Caller: `drive::record_inputs_if`.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::race::owner_replay::Replay;
use crate::race::replay_work_store::REPLAY_WORK;
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{on_tape, tape_offset}`.
pub fn record_inputs(replay: &Replay) -> Result<(), BadInput> {
    let car = crate::race::owner::race().shared.work.current_car.get();
    if !on_tape(car) {
        return Ok(());
    }
    let (eax, ebx, ecx, edx) = (
        crate::race::owner::race().shared.physics_tables.input_words[0].get(),
        crate::race::owner::race().shared.physics_tables.input_words[1].get(),
        crate::race::owner::race().shared.physics_tables.input_words[2].get(),
        crate::race::owner::race().shared.physics_tables.input_words[3].get(),
    );
    let packed = crate::drive::pack_inputs::pack_inputs(eax, ecx, edx);
    let esi = tape_offset(replay.replay_frames.tenth.get(), car);
    GamePtr::to(&REPLAY_WORK.replay_tape)
        .offset(esi)
        .write([ebx, packed], "the replay tape")?;
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(packed);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        crate::race::owner::race().shared.physics_tables.input_words[k].set(*v);
    }
    Ok(())
}
