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
use crate::menu::owner::{Menus, Play};
use crate::pointers::GamePtr;
use crate::session::Dword;

// ---- game data ----

/// The seven variables at the start, in the order they are saved.
pub fn head(menus: &Menus) -> [&Dword; 7] {
    [
        &menus.screens.event.player_count,
        &menus.screens.event.stages_done,
        &menus.screens.event.index, // the event's stage in its section
        &menus.screens.event.section,
        &menus.screens.event.event_kind,
        &menus.play.champ_view.event_mode,
        &menus.screens.state.menus.class_mode,
    ]
}
pub use rc2k_core::saves::savegame::{CAR_LEN, CARS, RECORDS_LEN};

/// The four players, if every one has its pointer at +0x18.
pub fn cars(
    play: &Play,
) -> Result<&[crate::session::Shared<crate::menu::player::Player>; CARS], BadInput> {
    if play
        .players
        .players
        .iter()
        .any(|p| p.get().name_player().is_none())
    {
        return Err(BadInput(
            "a car has no pointer at +0x18: the original reads through null",
        ));
    }
    Ok(&play.players.players)
}

// ---- Rust version ----

pub fn gather_savegame(menus: &Menus, buffer: GamePtr) -> Result<(), BadInput> {
    use rc2k_core::saves::savegame::{RECORDS_AT, car_at};
    let cars = cars(&menus.play)?;
    for (n, at) in head(menus).into_iter().enumerate() {
        // The savegame is a file's bytes.
        buffer
            .offset(4 * n as u32)
            .write(at.get(), "the savegame's head")?;
    }
    cars[0].update(|p| {
        p.saved_start_section = menus.screens.flow.start_section.get();
        p.saved_start_section_a8 = menus.screens.flow.start_section_a8.get();
    });
    for (n, car) in cars.iter().enumerate() {
        let name =
            crate::menu::player_names::name_bytes(crate::menu::player_names::name_of(&car.get())?);
        car.update(|p| p.name_copy = name);
        buffer.offset(car_at(n) as u32).write_bytes(
            &car.get()
                .to_file_bytes(crate::menu::player_names::name_address),
            "a player's record",
        )?;
    }
    GamePtr::to(&menus.cars.drivers).copy_to(
        buffer.offset(RECORDS_AT as u32),
        RECORDS_LEN,
        "the drivers' records",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::owner::menus;
    use crate::saves::owner::saves;
    use rc2k_core::menu::player::Player;
    use rc2k_core::saves::savegame::{LEN, car_at};

    /// The saved game's cars are the bytes the original wrote: each record as it is in
    /// memory, with the address of its name's buffer at +0x18.
    #[test]
    fn the_saved_cars_carry_the_names_addresses() {
        let play = &menus().play;
        let _guard = crate::store::test_lock();
        let buffer = GamePtr::to(&saves().save_games.file);
        let file_before = buffer.read_bytes(LEN, "the file").unwrap();
        let players_before = play
            .players
            .players
            .each_ref()
            .map(crate::session::Shared::get);
        let original_names = std::array::from_fn::<_, 4, _>(crate::menu::player_names::name_bytes);
        // Distinct names and a distinct byte in every dword of every record.
        for (n, car) in play.players.players.iter().enumerate() {
            let mut bytes = car.get().to_le_bytes();
            for (k, b) in bytes.iter_mut().enumerate().skip(0x28) {
                *b = (k + 7 * n) as u8;
            }
            let mut record = Player::from_le_bytes(bytes);
            record.set_name_player(n);
            car.set(record);
            crate::menu::player_names::set_name_bytes(n, &[b'a' + n as u8; 0x20]);
        }
        let names_before = std::array::from_fn::<_, 4, _>(crate::menu::player_names::name_bytes);
        gather_savegame(menus(), buffer).unwrap();
        let file = buffer.read_bytes(LEN, "the file").unwrap();
        for (n, car) in play.players.players.iter().enumerate() {
            // What the original wrote: the record, its name address in place of the number.
            let mut expected = car.get();
            expected.name_number = crate::menu::player_names::name_address(n);
            let at = car_at(n);
            assert_eq!(
                &file[at..at + 0x28c],
                &expected.to_le_bytes()[..],
                "car {n}"
            );
            let address = u32::from_le_bytes(file[at + 0x18..at + 0x1c].try_into().unwrap());
            assert_eq!(address, crate::menu::player_names::name_address(n));
            assert_eq!(&file[at + 0x124..at + 0x144], &names_before[n][..]);
        }
        buffer.write_bytes(&file_before, "the file").unwrap();
        for (car, before) in play.players.players.iter().zip(players_before) {
            car.set(before);
        }
        for (n, name) in original_names.iter().enumerate() {
            crate::menu::player_names::set_name_bytes(n, name);
        }
    }
}
