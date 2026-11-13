//! For cars 0 and 1 the input words from the replay's tape: frame
//! `step_frame` (16 bytes; the car's 8) has the first word and the packed rest, taken apart
//! (`drive::unpack_inputs`) into `input_words` ...

use crate::errors::BadInput;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::on_tape;

pub fn play_inputs(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    let replay = &race.replay;
    let car = shared.work.get().current_car.get();
    if !on_tape(car) {
        return Ok(());
    }
    let entry = replay
        .staging()
        .entry(replay.replay_frames().step_frame.get(), car)?;
    let (ebx, second) = (entry.input.get(), entry.packed.get());
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(second);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        shared.physics_tables.get().input_words[k].set(*v);
    }
    Ok(())
}
