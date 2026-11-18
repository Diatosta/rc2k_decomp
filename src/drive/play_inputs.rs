//! For the cars on the tape (0 and 1; 2 and 3 with the extra tape) the input words from the replay's tape: frame
//! `step_frame` (16 bytes; the car's 8) has the first word and the packed rest, taken apart
//! (`drive::unpack_inputs`) into `input_words` ...

use crate::errors::BadInput;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::on_tape;

pub fn play_inputs(race: &Race) -> Result<(), BadInput> {
    let shared = &race.shared;
    let replay = &race.replay;
    let car = shared.work.get().current_car.get();
    let extended = race.viewer_buffers().extra_tape.is_set();
    let tail = race.viewer_buffers().v2.is_set() && (4..8).contains(&car);
    if !on_tape(car, extended) && !tail {
        return Ok(());
    }
    let (ebx, second) = if tail {
        let step = replay.replay_frames().step_frame.get() as usize;
        let v2 = replay.v2().lock();
        let [keys, packed] = v2
            .tail
            .tape
            .get(step)
            .ok_or(BadInput("a tape entry past the frame's tenths"))?[car as usize - 4];
        (keys, packed)
    } else {
        let entry =
            replay
                .staging()
                .entry(replay.replay_frames().step_frame.get(), car, extended)?;
        (entry.input.get(), entry.packed.get())
    };
    let [eax, ecx, edx] = crate::drive::unpack_inputs::unpack_inputs(second);
    for (k, v) in [eax, ebx, ecx, edx].iter().enumerate() {
        shared.physics_tables.get().input_words[k].set(*v);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::own::race;

    /// Inputs recorded for each of four cars come back from the block of a replay with the extra
    /// tape; without it cars 2 and 3 are not on the tape and nothing is read or written.
    #[test]
    fn four_cars_inputs_go_through_the_block() {
        let _g = crate::test_support::test_lock();
        let race = race();
        let words = &race.shared.physics_tables.get().input_words;
        let (tenth, car_cell) = (
            &race.replay.replay_frames().tenth,
            &race.shared.work.get().current_car,
        );
        let kept_block = race.viewer_buffers().block_bytes();
        let kept_flag = race.viewer_buffers().extra_tape.is_set();
        let kept: Vec<u32> = words
            .iter()
            .map(crate::memory::session::PlainDword::get)
            .collect();
        let kept_car = car_cell.get();
        let kept_tenth = tenth.get();
        race.viewer_buffers().extra_tape.set(true);
        race.replay.staging().clear_extra_tape();
        tenth.set(37);
        let value = |car: u32, k: u32| (car + 1) * 0x101 + k * 0x20;
        for car in 0..4u32 {
            car_cell.set(car);
            // Throttle, keys, steering and brake in the ranges the packing keeps.
            for (k, w) in words.iter().enumerate() {
                w.set(if k == 1 {
                    value(car, 1)
                } else {
                    value(car, k as u32) & 0xFF
                });
            }
            crate::drive::record_inputs::record_inputs(race).unwrap();
        }
        // The frame goes to the block and back, as a recording is written and a replay read.
        crate::race::replay_staging_store::store_in_block(race);
        for e in race.replay.staging().extra_tape.iter().flatten() {
            e.input.set(0);
            e.packed.set(0);
        }
        crate::race::replay_staging_store::load_from_block(race);
        for car in 0..4u32 {
            car_cell.set(car);
            race.replay.replay_frames().step_frame.set(37);
            for w in words.iter() {
                w.set(0);
            }
            play_inputs(race).unwrap();
            assert_eq!(words[1].get(), value(car, 1), "car {car}");
        }
        // Without the extra tape cars 2 and 3 are not on the tape.
        race.viewer_buffers().extra_tape.set(false);
        car_cell.set(2);
        words[1].set(77);
        play_inputs(race).unwrap();
        assert_eq!(words[1].get(), 77);
        race.viewer_buffers().extra_tape.set(kept_flag);
        race.replay.staging().clear_extra_tape();
        car_cell.set(kept_car);
        tenth.set(kept_tenth);
        for (w, v) in words.iter().zip(kept) {
            w.set(v);
        }
        race.viewer_buffers()
            .put_in_block(0, &kept_block, "test")
            .ok();
    }
}
