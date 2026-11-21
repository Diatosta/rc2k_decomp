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
    // A replay of the own format also has a tape for cars 4 .. 7 (`race::replay_v2_store`).
    let tail = race.viewer_buffers().v2.is_set() && (4..8).contains(&car);
    if !on_tape(car, extended) && !tail {
        return Ok(());
    }
    let (eax, ebx, ecx, edx) = (
        shared.run().input_words[0],
        shared.run().input_words[1],
        shared.run().input_words[2],
        shared.run().input_words[3],
    );
    let packed = crate::drive::pack_inputs::pack_inputs(eax, ecx, edx);
    if tail {
        let tenth = replay.replay_frames().tenth as usize;
        let mut v2 = replay.v2().lock();
        let entry = v2
            .tail
            .tape
            .get_mut(tenth)
            .ok_or(BadInput("a tape entry past the frame's tenths"))?;
        entry[car as usize - 4] = [ebx, packed];
    } else {
        let tenth = replay.replay_frames().tenth;
        replay.staging_mut().set_entry(
            tenth,
            car,
            extended,
            crate::race::replay_staging_store::TapeEntry { input: ebx, packed },
        )?;
    }
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(packed);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        shared.run_mut().input_words[k] = *v;
    }
    Ok(())
}
