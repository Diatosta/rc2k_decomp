//! `sound::play_car_stream` (0x469AA1): the car shown's sound stream started.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The car (as `menu::load_car_background` finds it, 0x21 counting as 0x17):
//! `dword_A8C188` = 1, `dword_A8C184` = `off_75A31E[car]`, `dword_A8C180` = 0, and
//! `sound::start_stream` with eax = the first dword there and ecx = `dword_755698`.
//! Five call sites, next to `load_car_background`'s.
//! Refuses what the record's lookup refuses.

use crate::errors::Refused;
use crate::menu::load_car_background::shown_car;
use crate::menu::owner::menus;

// ---- game data ----

// dword_755990

// ---- Rust version ----

/// `start` is `sound::start_stream` (the file, the volume).
pub fn play_car_stream(start: &mut dyn FnMut(u32, u32)) -> Result<(), Refused> {
    let menus = menus();
    // Converted: `rc2k_core::sound::set_up::car_stream`.
    let car = shown_car(menus)?;
    let failed = std::cell::Cell::new(None);
    let s = rc2k_core::sound::set_up::car_stream(
        // `MENU_LISTS.car_streams` (off_75A31E) has 24 entries; the original reads any
        // car's, past them what the image has after (`read_on` goes on past them too).
        crate::session::read_on_in(
            &menus.tables.lists,
            &menus.tables.lists.car_streams,
            car as usize,
            "a car's stream",
        )?,
        |list| {
            crate::menu::texts_store::speech_entry(list, 0).unwrap_or_else(|e| {
                failed.set(Some(e));
                0
            })
        },
        menus.screens.option_work.speech_level.get(),
    );
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    crate::sound::owner::sound().race.car_stream.on.set(s.on);
    crate::sound::owner::sound()
        .race
        .car_stream
        .stream
        .set(s.list);
    crate::sound::owner::sound()
        .race
        .car_stream
        .place
        .set(s.place);
    start(s.first, s.volume);
    Ok(())
}
