//! `saves::gather_savegame` (0x4AB427): copies the state a saved game holds into a buffer.
//!
//! In:  edi = where it goes (0x725C bytes)
//! Out: every register preserved
//! In this order:
//! - seven dwords: `dword_755984`, `dword_755950`, the event's stage index and section
//!   (0x755958, 0x75595C), `dword_755988`, `dword_7556D0`, `dword_75AB40`;
//! - the four players `off_75647C[0 .. 4]`, 0x28C bytes each;
//! - 180 records of 0x94 bytes from `unk_73B674`.
//!
//! Before the cars are copied it also changes them: car 0 gets `dword_7554C0` at +0x14 and
//! `dword_7554C4` at +0x274, and every car gets the 0x20 bytes its pointer at +0x18 points to
//! copied to +0x124 (a name, by its size; a reading), so that the saved copy carries them.
//! What the seven dwords and the records are has not been worked out.
//! Refuses, before anything is written: a missing player or a car without that pointer.

use crate::errors::BadInput;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::state_store::MENU_STATE;
use crate::session::Dword;
use std::ptr::copy_nonoverlapping;

// ---- game data ----

/// The seven variables at the start, in the order they are saved.
pub fn head() -> [&'static Dword; 7] {
    [
        &EVENT_STATE.player_count,
        &EVENT_STATE.stages_done,
        &EVENT_STATE.index, // the event's stage in its section
        &EVENT_STATE.section,
        &EVENT_STATE.event_kind,
        &CHAMP_VIEW.event_mode,
        &MENU_STATE.menus.class_mode,
    ]
}
pub use rc2k_core::saves::savegame::{CAR_LEN, CARS, NAME_LEN, RECORDS_LEN};
pub use rc2k_core::saves::savegame::{NAME_AT as NAME_OFFSET, NAME_POINTER as NAME_PTR_OFFSET};

/// The four players, if every one is there and has its pointer at +0x18.
pub fn cars() -> Result<[u32; CARS], BadInput> {
    let mut cars = [0u32; CARS];
    for (n, car) in cars.iter_mut().enumerate() {
        *car = unsafe { ((PLAYER_TABLE.pointers[0].addr() as *const u32).add(n)).read_unaligned() };
        if *car == 0 {
            return Err(BadInput(
                "a player is missing: the original reads through null",
            ));
        }
        if unsafe { ((*car + NAME_PTR_OFFSET) as *const u32).read_unaligned() } == 0 {
            return Err(BadInput(
                "a car has no pointer at +0x18: the original reads through null",
            ));
        }
    }
    Ok(cars)
}

// ---- Rust version ----

pub unsafe fn gather_savegame(buffer: *mut u8) -> Result<(), BadInput> {
    use rc2k_core::saves::savegame::{CAR0_A, CAR0_B, RECORDS_AT, car_at};
    // Converted: `rc2k_core::saves::savegame` (the state's layout).
    let cars = cars()?;
    unsafe {
        for (n, at) in head().into_iter().enumerate() {
            (buffer.add(4 * n) as *mut u32).write_unaligned(at.get());
        }
        ((cars[0] + CAR0_A) as *mut u32).write_unaligned(SCREEN_FLOW.start_section.get());
        ((cars[0] + CAR0_B) as *mut u32).write_unaligned(SCREEN_FLOW.start_section_a8.get());
        for (n, car) in cars.into_iter().enumerate() {
            let name = ((car + NAME_PTR_OFFSET) as *const u32).read_unaligned();
            std::ptr::copy(name as *const u8, (car + NAME_OFFSET) as *mut u8, NAME_LEN);
            copy_nonoverlapping(car as *const u8, buffer.add(car_at(n)), CAR_LEN);
        }
        copy_nonoverlapping(
            crate::menu::drivers_store::drivers_addr() as *const u8,
            buffer.add(RECORDS_AT),
            RECORDS_LEN,
        );
    }
    Ok(())
}
