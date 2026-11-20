//! `sound::step_car_stream`: the car's sound stream goes on to its next part when one has ended.
//!
//! Only with `ChampView.event_mode` at 2 and `CarStream.on` at 1, and while the stream is not
//! playing (`sound::stream_playing`: `AudioShared.sound_stream` is not 1): `CarStream.place` + 1
//! and the part at that place of the list `CarStream.stream`; -1 ends it (`CarStream.on` = 0), else
//! `sound::start_stream` with it and `MenuOptions.menu_voices_volume`. Once a frame on four screens (the
//! car specification, the slideshow, two stage screens).

use crate::errors::BadInput;

// ---- game data ----

// `sound::stream_playing`

/// With the game's stream flag and `sound::start_stream`.
pub fn step_car_stream(o: crate::owners::Owners<'static>) -> Result<(), BadInput> {
    let (menus, sound, audio) = (o.menus, o.sound, o.audio);
    let (champ, options, texts) = (&menus.play.champ_view, &menus.options, &menus.texts);
    let names = &audio.sample_names;
    let playing = || audio.state.sound_stream.get().playing == 1;
    // The logic is `rc2k_core::sound::race::stream_step`.
    use rc2k_core::sound::race::{StreamStep, stream_step};
    let list: u32 = sound.race.car_stream.get().borrow().stream;
    let failed = std::cell::Cell::new(None);
    let (on, place_now) = {
        let stream = sound.race.car_stream.get().borrow();
        (stream.on, stream.place)
    };
    let step = stream_step(champ.read().event_mode, on, playing, place_now, |place| {
        crate::menu::texts_store::speech_entry(texts, names, list, place).unwrap_or_else(|e| {
            failed.set(Some(e));
            0
        })
    });
    if let Some(e) = failed.take() {
        return Err(e);
    }
    let (place, part) = match step {
        StreamStep::Nothing => return Ok(()),
        StreamStep::Ended { place } => {
            let mut stream = sound.race.car_stream.get().borrow_mut();
            stream.place = place;
            stream.on = 0;
            return Ok(());
        }
        StreamStep::Start { place, part } => (place, part),
    };
    sound.race.car_stream.get().borrow_mut().place = place;
    crate::sound::start_stream::start_stream(o, part, options.snapshot().menu_voices_volume);
    Ok(())
}
