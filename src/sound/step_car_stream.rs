//! `sound::step_car_stream` (0x46F7AD): the car's sound stream goes on to its next part
//! when one has ended.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Only with `dword_7556D0` at 2 and `dword_A8C188` at 1, and while the stream is not
//! playing (`sub_40C4D5`: `dword_506960` is not 1; one other caller): `dword_A8C180` + 1
//! and the part at that place of the list `dword_A8C184`; -1 ends it
//! (`dword_A8C188` = 0), else `sound::start_stream` with it and `dword_755698`. Once a
//! frame on four screens (the car specification, the slideshow, two stage screens).

use crate::audio::store::AUDIO;
use crate::errors::BadInput;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::option_work_store::OPTION_WORK;
use crate::pointers::GamePtr;

// ---- game data ----

// sub_40C4D5

// ---- Rust version ----

/// `playing` stands for `sub_40C4D5`, `start` for `sound::start_stream` (the part, the
/// volume).
pub fn step_car_stream(
    playing: &mut dyn FnMut() -> bool,
    start: &mut dyn FnMut(u32, u32),
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::sound::race::stream_step`.
    use rc2k_core::sound::race::{StreamStep, stream_step};
    let list: u32 = crate::sound::store::CAR_STREAM.stream.get();
    let failed = std::cell::Cell::new(None);
    let step = stream_step(
        CHAMP_VIEW.event_mode.get(),
        crate::sound::store::CAR_STREAM.on.get(),
        playing,
        crate::sound::store::CAR_STREAM.place.get(),
        |place| {
            GamePtr::new(list.wrapping_add(place.wrapping_mul(4)))
                .read::<u32>("a car's stream list")
                .unwrap_or_else(|e| {
                    failed.set(Some(e));
                    0
                })
        },
    );
    if let Some(e) = failed.take() {
        return Err(e);
    }
    let (place, part) = match step {
        StreamStep::Nothing => return Ok(()),
        StreamStep::Ended { place } => {
            crate::sound::store::CAR_STREAM.place.set(place);
            crate::sound::store::CAR_STREAM.on.set(0);
            return Ok(());
        }
        StreamStep::Start { place, part } => (place, part),
    };
    crate::sound::store::CAR_STREAM.place.set(place);
    start(part, OPTION_WORK.speech_level.get());
    Ok(())
}

/// With the game's stream flag and `sound::start_stream`.
pub fn step_car_stream_in_game() -> Result<(), BadInput> {
    step_car_stream(
        &mut || AUDIO.sound_stream.get().playing == 1,
        &mut crate::sound::start_stream::start_stream,
    )
}
