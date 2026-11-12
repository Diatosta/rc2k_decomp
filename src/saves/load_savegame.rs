//! `saves::load_savegame` (0x46E6E1): the action of a slot on the "load game" menu: reads the
//! saved game, puts its state back into the game and goes on with the event from there.
//!
//! In order:
//! - `menu::ready_players`; the event is marked as going on (`event_goes_on` = 1), the
//!   times are not reset (`reset_times` = 0) and the player is not out (`event_out` = 0);
//! - `read_savegame_slot` on the highlighted slot (page x 10 + item - 1). If that fails:
//!   menu sound 8, `menu::go_back`, and that is all;
//! - the seven dwords, the four cars and the 180 records go back where `gather_savegame`
//!   took them from; the saved cars are left in the file's buffer (`saved_cars`); between
//!   the dwords and the cars it calls `saves::keep_player_settings` (only if
//!   `use_savegame_keys` is 0) and `menu::drop_missing_controllers`. Each car's 0x20 bytes
//!   at +0x124 are copied to where the car's name pointer at +0x18 pointed before the car
//!   was overwritten; car 0's two start sections go back to `flow.start_section` and
//!   `start_section_a8`;
//! - `menu::next_record_kind` says where the event stands: 4 or 5, the next section starts
//!   (section + 1, stage index 0); 0 or 1, the next stage (index + 1); 3, nothing more
//!   happens here; any other value is fatal error 0x80702821 "Corrupt SaveGame";
//! - unless it was 3: `continuing_save` = 1, `menu::link_player_names`, and then the event
//!   itself runs: `menu::run_a8_championship` if the event mode is 0x13, else
//!   `menu::run_championship`. These only come back when the event is over or left;
//! - `menu::go_back`.
//!
//! Not the original's: the players' own records and names are set aside first and put
//! back at the end (`menu::own_settings`).
//!
//! Ten menu tables point at this function. Refuses, before anything is done: a slot past 99,
//! a missing player or a car without its name pointer; later, what its Rust callees refuse
//! (the event loops when their callees refuse).

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::go_back;
use crate::menu::owner::Menus;
use crate::owners::Owners;
use crate::pointers::Store;
use crate::saves::gather_savegame::{CAR_LEN, RECORDS_LEN, cars, head};
use crate::saves::read_savegame_slot::read_savegame_slot_in;
use crate::saves::write_savegame_slot::SLOTS;
use rc2k_core::saves::savegame::{A8_MODE, CORRUPT, Resume, resume};

// ---- game data ----

/// Where the saved game starts in the file: what `gather_savegame` wrote.
const PAYLOAD_OFFSET: usize = 0x30;

/// The menu sound of a slot that cannot be read.
const UNREADABLE_SOUND: u32 = 8;

/// The slot the highlighted menu item stands for.
fn highlighted_slot(menus: &Menus) -> u32 {
    // Converted: `rc2k_core::saves::savegame::slot`.
    rc2k_core::saves::savegame::slot(
        menus.tables.pages.save_page.get(),
        menus.screens.loop_flags.item.get(),
    )
}

/// The players' own records and names come back when
/// the loaded championship is over or left (`menu::own_settings`; not the original's).
pub fn load_savegame(o: Owners<'_>) -> Result<(), Refused> {
    crate::menu::own_settings::around_championship(o.menus, || load_and_run(o))
}

fn load_and_run(o: Owners<'_>) -> Result<(), Refused> {
    let (menus, saves) = (o.menus, o.saves);
    let slot = highlighted_slot(menus);
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let cars = cars(&menus.play)?;

    crate::menu::ready_players::ready_players(menus)?;
    menus.screens.event.event_goes_on.set(1u32);
    menus.tables.pages.reset_times.set(0u32);
    menus.screens.event.event_out.set(0u32);
    if !read_savegame_slot_in(saves, slot)? {
        crate::sound::play_menu_sound::play_menu_sound_in_game(UNREADABLE_SOUND);
        go_back::go_back(menus);
        return Ok(());
    }

    let file = &saves.save_games.file;
    let mut cursor = PAYLOAD_OFFSET;
    for at in head(menus) {
        let word = file.read_at(cursor, 4, "the savegame's head")?;
        at.set(u32::from_le_bytes([word[0], word[1], word[2], word[3]]));
        cursor += 4;
    }
    if saves.savegame_preview.use_savegame_keys.get() == 0 {
        crate::saves::keep_player_settings::keep_player_settings(menus, file, cursor)?;
    }
    crate::menu::drop_missing_controllers::drop_missing_controllers(menus, o.input)?;

    for car in cars {
        let name = car.get().name_number;
        let saved: [u8; CAR_LEN] = file
            .read_at(cursor, CAR_LEN, "a saved car")?
            .try_into()
            .map_err(|_| BadInput("a saved car of another length"))?;
        car.set(rc2k_core::menu::player::Player::from_file_bytes(saved));
        // The file's own pointer is not to be trusted: the game's stays (a save's holds the
        // same place, unless the file was damaged).
        car.update(|p| p.name_number = name);
        crate::menu::player_names::set_name_bytes(
            menus,
            crate::menu::player_names::name_of(&car.get())?,
            &car.get().name_copy,
        );
        cursor += CAR_LEN;
    }
    let records = file.read_at(cursor, RECORDS_LEN, "the saved drivers")?;
    menus.cars.drivers.store(0, &records)?;
    menus
        .screens
        .flow
        .start_section
        .set(cars[0].get().saved_start_section);
    menus
        .screens
        .flow
        .start_section_a8
        .set(cars[0].get().saved_start_section_a8);

    match resume(crate::menu::next_record_kind::next_record_kind(menus)?) {
        Resume::NextSection => {
            menus
                .screens
                .event
                .section
                .set(menus.screens.event.section.get().wrapping_add(1));
            menus.screens.event.index.set(0u32);
        }
        Resume::NextStage => menus
            .screens
            .event
            .index
            .set(menus.screens.event.index.get().wrapping_add(1)),
        Resume::Over => {
            go_back::go_back(menus);
            return Ok(());
        }
        Resume::Corrupt => {
            return Err(FatalError {
                code: CORRUPT,
                message: "Corrupt SaveGame",
            }
            .into());
        }
    }
    menus.screens.event.continuing_save.set(1u32);
    crate::menu::link_player_names::link_player_names(menus)?;
    if menus.play.champ_view.event_mode.get() == A8_MODE {
        crate::menu::run_a8_championship::run_a8_championship_in(o)?;
    } else {
        crate::menu::run_championship::run_championship_in(o)?;
    }
    go_back::go_back(menus);
    Ok(())
}
