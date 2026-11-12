//! `sound::step_car_stream` (0x46F7AD): the car's sound stream goes on to its next part
//! when one has ended.
//!
//! Only with `ChampView.event_mode` at 2 and `CarStream.on` at 1, and while the stream is not
//! playing (`sound::stream_playing`: `AudioState.sound_stream` is not 1; one other caller): `CarStream.place` + 1
//! and the part at that place of the list `CarStream.stream`; -1 ends it
//! (`CarStream.on` = 0), else `sound::start_stream` with it and `OptionWork.speech_level`. Once a
//! frame on four screens (the car specification, the slideshow, two stage screens).

use crate::errors::BadInput;
use crate::menu::owner::Menus;

// ---- game data ----

// `sound::stream_playing`

/// `playing` stands for `sound::stream_playing`, `start` for `sound::start_stream` (the part, the
/// volume).
pub fn step_car_stream(
    playing: &mut dyn FnMut() -> bool,
    start: &mut dyn FnMut(u32, u32),
    menus: &Menus,
    sound: &crate::sound::owner::Sound,
) -> Result<(), BadInput> {
    // Converted: `rc2k_core::sound::race::stream_step`.
    use rc2k_core::sound::race::{StreamStep, stream_step};
    let list: u32 = sound.race.car_stream.get().stream.get();
    let failed = std::cell::Cell::new(None);
    let step = stream_step(
        menus.play.champ_view.event_mode.get(),
        sound.race.car_stream.get().on.get(),
        playing,
        sound.race.car_stream.get().place.get(),
        |place| {
            crate::menu::texts_store::speech_entry(list, place).unwrap_or_else(|e| {
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
            sound.race.car_stream.get().place.set(place);
            sound.race.car_stream.get().on.set(0);
            return Ok(());
        }
        StreamStep::Start { place, part } => (place, part),
    };
    sound.race.car_stream.get().place.set(place);
    start(part, menus.screens.option_work.speech_level.get());
    Ok(())
}

/// With the game's stream flag and `sound::start_stream`.
pub fn step_car_stream_in_game(
    menus: &Menus,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
    system: &crate::system::owner::System,
) -> Result<(), BadInput> {
    step_car_stream(
        &mut || audio.state.sound_stream.get().playing == 1,
        &mut |name, volume| {
            crate::sound::start_stream::start_stream(audio, system, name, volume);
        },
        menus,
        sound,
    )
}
