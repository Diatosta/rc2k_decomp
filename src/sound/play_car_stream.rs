//! `sound::play_car_stream` (0x469AA1): the car shown's sound stream started.
//!
//! The car (as `menu::load_car_background` finds it, 0x21 counting as 0x17):
//! `CarStream.on` = 1, `CarStream.stream` = `MenuLists.car_streams[0][car]`, `CarStream.place` = 0, and
//! `sound::start_stream` with the name (the first dword there) and the volume `OptionWork.speech_level`.
//! Five call sites, next to `load_car_background`'s.
//! Refuses what the record's lookup refuses.

use crate::errors::Refused;
use crate::menu::load_car_background::shown_car;
use crate::menu::owner::Menus;

// ---- game data ----

// Event.shown_car

/// `start` is `sound::start_stream` (the file, the volume).
pub fn play_car_stream(
    start: &mut dyn FnMut(u32, u32),
    menus: &Menus,
    sound: &crate::sound::owner::Sound,
    names: &crate::audio::names_store::SampleNames,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::set_up::car_stream`.
    let car = shown_car(menus)?;
    let failed = std::cell::Cell::new(None);
    let s = rc2k_core::sound::set_up::car_stream(
        // `MENU_LISTS.car_streams` (MenuLists.car_streams[0]) has 24 entries; the original reads any
        // car's, past them what the image has after (`read_on` goes on past them too).
        crate::session::read_on_in(
            &menus.tables.lists,
            &menus.tables.lists.car_streams,
            car as usize,
            "a car's stream",
        )?,
        |list| {
            crate::menu::texts_store::speech_entry(names, list, 0).unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
        },
        menus.screens.option_work.speech_level.get(),
    );
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    sound.race.car_stream.get().on.set(s.on);
    sound.race.car_stream.get().stream.set(s.list);
    sound.race.car_stream.get().place.set(s.place);
    start(s.first, s.volume);
    Ok(())
}
