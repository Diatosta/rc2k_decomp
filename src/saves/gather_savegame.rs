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
use crate::pointers::GamePtr;
use crate::session::Dword;

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
        *car = GamePtr::new(PLAYER_TABLE.pointers[0].addr())
            .offset(4 * n as u32)
            .read::<u32>("the players' table")?;
        if *car == 0 {
            return Err(BadInput(
                "a player is missing: the original reads through null",
            ));
        }
        if GamePtr::new(*car + NAME_PTR_OFFSET).read::<u32>("a player's record")? == 0 {
            return Err(BadInput(
                "a car has no pointer at +0x18: the original reads through null",
            ));
        }
    }
    Ok(cars)
}

// ---- Rust version ----

pub fn gather_savegame(buffer: GamePtr) -> Result<(), BadInput> {
    use rc2k_core::saves::savegame::{CAR0_A, CAR0_B, RECORDS_AT, car_at};
    let cars = cars()?;
    for (n, at) in head().into_iter().enumerate() {
        buffer
            .offset(4 * n as u32)
            .write(at.get(), "the savegame's head")?;
    }
    GamePtr::new(cars[0] + CAR0_A).write(SCREEN_FLOW.start_section.get(), "a player's record")?;
    GamePtr::new(cars[0] + CAR0_B)
        .write(SCREEN_FLOW.start_section_a8.get(), "a player's record")?;
    for (n, car) in cars.into_iter().enumerate() {
        let name = GamePtr::new(car + NAME_PTR_OFFSET).read::<u32>("a player's record")?;
        GamePtr::new(name).copy_to(GamePtr::new(car + NAME_OFFSET), NAME_LEN, "a player's name")?;
        GamePtr::new(car).copy_to(
            buffer.offset((car_at(n)) as u32),
            CAR_LEN,
            "a player's record",
        )?;
    }
    GamePtr::new(crate::menu::drivers_store::drivers_addr()).copy_to(
        buffer.offset(RECORDS_AT as u32),
        RECORDS_LEN,
        "the drivers' records",
    )?;
    Ok(())
}
