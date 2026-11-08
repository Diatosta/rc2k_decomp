//! `drive::record_inputs` (0x4601A8): for cars 0 and 1 the input words onto the replay's
//! tape: ebx and the packed rest (`drive::pack_inputs`) into frame `dword_6EF7DC` (the
//! car's 8 of its 16 bytes), and the words given back as the tape keeps them
//! (`drive::unpack_inputs`).
//!
//! Caller: `drive::record_inputs_if`.

use crate::errors::BadInput;
use crate::pointers::Store;
use crate::race::owner::Race;
use crate::race::replay_work_store::REPLAY_WORK;
use rc2k_core::drive::replay_frames::{on_tape, tape_offset};

/// The Rust version.
// Converted: `rc2k_core::drive::replay_frames::{on_tape, tape_offset}`.
pub fn record_inputs(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    let replay = &race.replay;
    let car = shared.work.current_car.get();
    if !on_tape(car) {
        return Ok(());
    }
    let (eax, ebx, ecx, edx) = (
        shared.physics_tables.input_words[0].get(),
        shared.physics_tables.input_words[1].get(),
        shared.physics_tables.input_words[2].get(),
        shared.physics_tables.input_words[3].get(),
    );
    let packed = crate::drive::pack_inputs::pack_inputs(eax, ecx, edx);
    let esi = tape_offset(replay.replay_frames.tenth.get(), car);
    let mut words = [0u8; 8];
    words[..4].copy_from_slice(&ebx.to_le_bytes());
    words[4..].copy_from_slice(&packed.to_le_bytes());
    REPLAY_WORK.replay_tape.store(esi as usize, &words)?;
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(packed);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        shared.physics_tables.input_words[k].set(*v);
    }
    Ok(())
}
