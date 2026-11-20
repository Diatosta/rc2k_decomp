//! Copies the state a saved game holds into a buffer.
//!
//! In this order:
//! - seven dwords (`head`): the event's player count, stages done, stage index and
//!   section, event kind, event mode, and the class mode;
//! - the four players, 0x28C bytes each;
//! - 180 records of 0x94 bytes from the drivers' table.
//!
//! Before the cars are copied it also changes them: car 0 gets the two start sections
//! (`flow.start_section` and `start_section_a8`, into `saved_start_section` and
//! `saved_start_section_a8` of its record), and every car gets the 0x20 bytes of its name's
//! buffer (the pointer in `name_number`) copied to +0x124 of its saved record, so that the
//! saved copy carries them.
//! Refuses, before anything is written: a missing player or a car without that pointer.

use crate::errors::BadInput;
use crate::memory::cells::Store;
use crate::menu::owner::{Menus, Play};

/// The seven variables at the start, in the order they are saved.
#[derive(Clone, Copy)]
pub enum Head {
    PlayerCount,
    StagesDone,
    StageInSection,
    Section,
    EventKind,
    EventMode,
    ClassMode,
}

/// One of the head's variables in the menus' state.
#[derive(Clone, Copy)]
pub struct HeadWord<'a> {
    menus: &'a Menus,
    which: Head,
}

impl HeadWord<'_> {
    pub fn get(&self) -> u32 {
        let m = self.menus;
        match self.which {
            Head::PlayerCount => m.screens.event.read().player_count,
            Head::StagesDone => m.screens.event.read().stages_done,
            Head::StageInSection => m.screens.event.read().stage_in_section,
            Head::Section => m.screens.event.read().section,
            Head::EventKind => m.screens.event.read().event_kind,
            Head::EventMode => m.play.champ_view.read().event_mode,
            Head::ClassMode => m.screens.state.read().menus.class_mode,
        }
    }

    pub fn set(&self, value: u32) {
        let m = self.menus;
        match self.which {
            Head::PlayerCount => m.screens.event.write().player_count = value,
            Head::StagesDone => m.screens.event.write().stages_done = value,
            Head::StageInSection => m.screens.event.write().stage_in_section = value,
            Head::Section => m.screens.event.write().section = value,
            Head::EventKind => m.screens.event.write().event_kind = value,
            Head::EventMode => m.play.champ_view.write().event_mode = value,
            Head::ClassMode => m.screens.state.write().menus.class_mode = value,
        }
    }
}

/// The seven variables at the start, in the order they are saved.
pub fn head(menus: &Menus) -> [HeadWord<'_>; 7] {
    [
        Head::PlayerCount,
        Head::StagesDone,
        Head::StageInSection,
        Head::Section,
        Head::EventKind,
        Head::EventMode,
        Head::ClassMode,
    ]
    .map(|which| HeadWord { menus, which })
}

pub use rc2k_core::saves::savegame::{CAR_LEN, CARS, RECORDS_LEN};

/// The four players, if every one has its name pointer in `name_number`.
pub fn cars(
    play: &Play,
) -> Result<&[crate::memory::session::Shared<crate::menu::player::Player>; CARS], BadInput> {
    if play
        .slots
        .records
        .iter()
        .any(|p| p.get().name_player().is_none())
    {
        return Err(BadInput(
            "a car has no pointer at +0x18: the original reads through null",
        ));
    }
    Ok(&play.slots.records)
}

/// The saved game's bytes put into `file` from byte `base` on.
pub fn gather_savegame(menus: &Menus, file: &dyn Store, base: usize) -> Result<(), BadInput> {
    use rc2k_core::saves::savegame::{RECORDS_AT, car_at};
    let cars = cars(&menus.play)?;
    for (n, at) in head(menus).into_iter().enumerate() {
        // The savegame is a file's bytes.
        file.store(base + 4 * n, &at.get().to_le_bytes())?;
    }
    cars[0].update(|p| {
        p.saved_start_section = menus.screens.flow.read().start_section;
        p.saved_start_section_a8 = menus.screens.flow.read().start_section_a8;
    });
    for (n, car) in cars.iter().enumerate() {
        let name = crate::menu::player_names::name_bytes(
            menus,
            crate::menu::player_names::name_of(&car.get())?,
        );
        car.update(|p| p.name_copy = name);
        file.store(base + car_at(n), &car.get().to_file_bytes())?;
    }
    let records = crate::menu::drivers_store::records_to_bytes(&menus.cars.drivers.drivers);
    debug_assert_eq!(records.len(), RECORDS_LEN);
    file.store(base + RECORDS_AT, &records)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc2k_core::menu::player::Player;
    use rc2k_core::saves::name_id::NameId;
    use rc2k_core::saves::savegame::{LEN, car_at};

    /// The saved game's cars are each record as it is in memory, with the address of its
    /// name's buffer in `name_number`.
    #[test]
    fn the_saved_cars_carry_the_names_addresses() {
        let play = &crate::test_support::own::menus().play;
        let _guard = crate::test_support::test_lock();
        let buffer = &crate::test_support::own::saves().save_games.file;
        let file_before = buffer.read_at(0, LEN, "the file").unwrap();
        let players_before = play
            .slots
            .records
            .each_ref()
            .map(crate::memory::session::Shared::get);
        let original_names = std::array::from_fn::<_, 4, _>(|i| {
            crate::menu::player_names::name_bytes(crate::test_support::own::menus(), i)
        });
        // Distinct names and a distinct byte in every dword of every record.
        for (n, car) in play.slots.records.iter().enumerate() {
            let mut bytes = car.get().to_le_bytes();
            for (k, b) in bytes.iter_mut().enumerate().skip(0x28) {
                *b = (k + 7 * n) as u8;
            }
            let mut record = Player::from_le_bytes(bytes);
            record.set_name_player(n);
            car.set(record);
            crate::menu::player_names::set_name_bytes(
                crate::test_support::own::menus(),
                n,
                &[b'a' + n as u8; 0x20],
            );
        }
        let names_before = std::array::from_fn::<_, 4, _>(|i| {
            crate::menu::player_names::name_bytes(crate::test_support::own::menus(), i)
        });
        gather_savegame(crate::test_support::own::menus(), buffer, 0).unwrap();
        let file = buffer.read_at(0, LEN, "the file").unwrap();
        for (n, car) in play.slots.records.iter().enumerate() {
            // The record, its name address in place of the number.
            let mut expected = car.get();
            expected.name_number = NameId::of_player(n).to_file();
            let at = car_at(n);
            assert_eq!(
                &file[at..at + 0x28c],
                &expected.to_le_bytes()[..],
                "car {n}"
            );
            let address = u32::from_le_bytes(file[at + 0x18..at + 0x1c].try_into().unwrap());
            assert_eq!(address, NameId::of_player(n).to_file());
            assert_eq!(&file[at + 0x124..at + 0x144], &names_before[n][..]);
        }
        buffer.write_at(0, &file_before, "the file").unwrap();
        for (car, before) in play.slots.records.iter().zip(players_before) {
            car.set(before);
        }
        for (n, name) in original_names.iter().enumerate() {
            crate::menu::player_names::set_name_bytes(crate::test_support::own::menus(), n, name);
        }
    }

    /// A saved game's bytes read back into the menus' state (the way a load does it: the head
    /// dwords, each car with the game's name pointer kept, the driver records) and gathered
    /// again are the same bytes; the first gathering of a varied state is frozen by checksum.
    #[test]
    fn a_savegame_loaded_and_saved_again_is_the_same_bytes() {
        use rc2k_core::saves::savegame::RECORDS_AT;
        let _guard = crate::test_support::test_lock();
        let menus = crate::test_support::own::menus();
        let buffer = &crate::test_support::own::saves().save_games.file;
        let file_before = buffer.read_at(0, LEN, "the file").unwrap();
        let heads_before: Vec<u32> = head(menus).iter().map(super::HeadWord::get).collect();
        let players_before = menus
            .play
            .slots
            .records
            .each_ref()
            .map(crate::memory::session::Shared::get);
        let names_before: [_; 4] =
            std::array::from_fn(|i| crate::menu::player_names::name_bytes(menus, i));
        let records_before =
            crate::menu::drivers_store::records_to_bytes(&menus.cars.drivers.drivers);
        let start_before = [
            menus.screens.flow.read().start_section,
            menus.screens.flow.read().start_section_a8,
        ];
        // A varied state: head values, cars, names, driver records.
        for (n, d) in head(menus).into_iter().enumerate() {
            d.set(0x11 * (n as u32 + 1));
        }
        for (n, car) in menus.play.slots.records.iter().enumerate() {
            let mut bytes = car.get().to_le_bytes();
            for (k, b) in bytes.iter_mut().enumerate().skip(0x28) {
                *b = (k * 5 + 3 * n) as u8;
            }
            let mut record = Player::from_le_bytes(bytes);
            record.set_name_player(n);
            car.set(record);
            crate::menu::player_names::set_name_bytes(menus, n, &[b'A' + n as u8; 0x20]);
        }
        let varied_records: Vec<u8> = (0..records_before.len())
            .map(|k| (k.wrapping_mul(2_654_435_761) >> 9) as u8)
            .collect();
        crate::menu::drivers_store::records_from_bytes(
            &menus.cars.drivers.drivers,
            &varied_records,
        )
        .unwrap();
        let wrote = records_to_vec(menus);

        gather_savegame(menus, buffer, 0).unwrap();
        let first = buffer.read_at(0, LEN, "the file").unwrap();
        let sum = crate::test_support::hash(&first[..LEN]);

        // The load half.
        let mut cursor = 0;
        for d in head(menus) {
            d.set(u32::from_le_bytes(
                first[cursor..cursor + 4].try_into().unwrap(),
            ));
            cursor += 4;
        }
        for (n, car) in menus.play.slots.records.iter().enumerate() {
            let name = car.get().name_number;
            let at = car_at(n);
            let saved: [u8; CAR_LEN] = first[at..at + CAR_LEN].try_into().unwrap();
            car.set(Player::from_file_bytes(saved));
            car.update(|p| p.name_number = name);
            crate::menu::player_names::set_name_bytes(
                menus,
                crate::menu::player_names::name_of(&car.get()).unwrap(),
                &car.get().name_copy,
            );
        }
        crate::menu::drivers_store::records_from_bytes(
            &menus.cars.drivers.drivers,
            &first[RECORDS_AT..RECORDS_AT + RECORDS_LEN],
        )
        .unwrap();
        gather_savegame(menus, buffer, 0).unwrap();
        let second = buffer.read_at(0, LEN, "the file").unwrap();
        assert_eq!(records_to_vec(menus), wrote);

        buffer.write_at(0, &file_before, "the file").unwrap();
        for (d, v) in head(menus).into_iter().zip(heads_before) {
            d.set(v);
        }
        for (car, before) in menus.play.slots.records.iter().zip(players_before) {
            car.set(before);
        }
        for (n, name) in names_before.iter().enumerate() {
            crate::menu::player_names::set_name_bytes(menus, n, name);
        }
        crate::menu::drivers_store::records_from_bytes(
            &menus.cars.drivers.drivers,
            &records_before,
        )
        .unwrap();
        menus.screens.flow.write().start_section = start_before[0];
        menus.screens.flow.write().start_section_a8 = start_before[1];

        assert!(second == first, "the saved game changed through a load");
        assert_eq!((first.len(), sum), (29_276, 13_361_182_919_990_968_310));
    }

    fn records_to_vec(menus: &Menus) -> Vec<u8> {
        crate::menu::drivers_store::records_to_bytes(&menus.cars.drivers.drivers)
    }
}
