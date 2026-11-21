//! `sound::play_car_stream`: the car shown's sound stream started.
//!
//! The car (as `menu::load_car_background` finds it, 0x21 counting as 0x17): `CarStream.on` is 1,
//! `CarStream.stream` is `MenuLists.car_streams[0][car]`, `CarStream.place` is 0, and
//! `sound::start_stream` is called with the name and the volume `MenuOptions.menu_voices_volume`. Five
//! call sites, next to `load_car_background`'s. Refuses what the record's lookup refuses.

use crate::errors::{BadInput, Refused};
use crate::menu::load_car_background::shown_car;
use crate::menu::owner::Menus;
use crate::sound::store::CarStream;

// ---- game data ----

/// The stream dword of car `car`. `car_streams` holds cars 0 .. 0x17; any car's number may index
/// it, so the cars past them read on into the next field, `stage_speeches`. The numbers come from
/// the 34 car descriptions, which nothing writes during play: 0 .. 0x20 (the Clio's 0x21 counts as
/// 0x17), so cars 0x18 .. 0x20 read `stage_speeches[0 .. 8]`; anything further is refused.
fn car_stream(lists: &crate::menu::lists_store::MenuLists, car: u32) -> Result<u32, BadInput> {
    let car = car as usize;
    let tables = lists.read();
    let streams = &tables.car_streams;
    match streams.get(car) {
        Some(stream) => Ok(*stream),
        None => tables
            .stage_speeches
            .get(car - streams.len())
            .copied()
            .ok_or(BadInput("a car's stream past the stage speeches")),
    }
}

/// `start` is `sound::start_stream` (the file, the volume).
pub fn play_car_stream(
    start: &mut dyn FnMut(u32, u32),
    menus: &Menus,
    sound: &crate::sound::owner::Sound,
    names: &crate::audio::names_store::SampleNames,
) -> Result<(), Refused> {
    // The logic is `rc2k_core::sound::set_up::car_stream`.
    let car = shown_car(menus)?;
    let failed = std::cell::Cell::new(None);
    let s = rc2k_core::sound::set_up::car_stream(
        car_stream(&menus.lists, car)?,
        |list| {
            crate::menu::texts_store::speech_entry(names, list, 0).unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
        },
        menus.options.snapshot().menu_voices_volume,
    );
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    *sound.race.car_stream.get().borrow_mut() = CarStream {
        on: s.on,
        stream: s.list,
        place: s.place,
    };
    start(s.first, s.volume);
    Ok(())
}
