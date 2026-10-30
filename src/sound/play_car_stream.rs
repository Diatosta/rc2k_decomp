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
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::load_car_background::shown_car;
use crate::menu::option_work_store::OPTION_WORK;
use crate::pointers::GamePtr;

// ---- game data ----

// dword_755990

// ---- Rust version ----

/// `start` is `sound::start_stream` (the file, the volume).
pub fn play_car_stream(start: &mut dyn FnMut(u32, u32)) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::set_up::car_stream`.
    let car = shown_car()?;
    let failed = std::cell::Cell::new(None);
    let s = rc2k_core::sound::set_up::car_stream(
        // `MENU_LISTS.car_streams` (off_75A31E) has 24 entries; the original reads any
        // car's, past them what the image has after (`read_on` goes on past them too).
        crate::menu::texts_store::text_at(crate::session::read_on(
            &MENU_LISTS.car_streams,
            car as usize,
            "a car's stream",
        )?)?
        .addr(),
        |list| {
            GamePtr::new(list)
                .read::<u32>("a car's stream list")
                .unwrap_or_else(|e| {
                    failed.set(Some(e));
                    0
                })
        },
        OPTION_WORK.speech_level.get(),
    );
    if let Some(e) = failed.take() {
        return Err(e.into());
    }
    crate::sound::store::CAR_STREAM.on.set(s.on);
    crate::sound::store::CAR_STREAM.stream.set(s.list);
    crate::sound::store::CAR_STREAM.place.set(s.place);
    start(s.first, s.volume);
    Ok(())
}
