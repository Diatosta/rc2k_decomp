//! For the cars on the tape (0 and 1; 2 and 3 with the extra tape) the input words onto the replay's tape: the
//! first word and the packed rest (`drive::pack_inputs`) into the frame `tenth` (the car's 8 of its
//! 16 bytes), and the words given back as the tape keeps them (`drive::unpack_inputs`).

use crate::errors::BadInput;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::on_tape;

pub fn record_inputs(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    let replay = &race.replay;
    let car = shared.work.get().current_car.get();
    let extended = race.viewer_buffers().extra_tape.is_set();
    if !on_tape(car, extended) {
        return Ok(());
    }
    let (eax, ebx, ecx, edx) = (
        shared.physics_tables.get().input_words[0].get(),
        shared.physics_tables.get().input_words[1].get(),
        shared.physics_tables.get().input_words[2].get(),
        shared.physics_tables.get().input_words[3].get(),
    );
    let packed = crate::drive::pack_inputs::pack_inputs(eax, ecx, edx);
    let entry = replay
        .staging()
        .entry(replay.replay_frames().tenth.get(), car, extended)?;
    entry.input.set(ebx);
    entry.packed.set(packed);
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(packed);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        shared.physics_tables.get().input_words[k].set(*v);
    }
    Ok(())
}
