//! The action of a slot on the "load game" menu: reads the saved game, puts its state back into
//! the game and goes on with the event from there.
//!
//! In order: `menu::ready_players`; `read_savegame_slot` on the highlighted slot (page x 10 +
//! item - 1; if that fails: menu sound 8, `menu::go_back`, and that is all); the seven dwords,
//! the four cars and the 180 records go back where `gather_savegame` took them from (between
//! the dwords and the cars `saves::keep_player_settings` runs if `use_savegame_keys` is 0, and
//! `menu::drop_missing_controllers`); `menu::next_record_kind` says where the event stands (4
//! or 5: the next section; 0 or 1: the next stage; 3: nothing more happens here; any other
//! value is a damaged file: the player is told and the menu is back); unless it was 3, the event itself runs
//! (`menu::run_a8_championship` if the event mode is 0x13, else `menu::run_championship`) and
//! `menu::go_back` follows. The players' own records and names are set aside first and put back
//! at the end (`menu::own_settings`).
//!
//! A file that is not a saved game (a corrupt one, a short one) is not a reason to end the
//! game: the player is told (`error_dialog`) and is back at the menu.
//!
//! Refuses, before anything is done: a slot past 99, a missing player or a car without its name
//! pointer; later, what its callees refuse.

use crate::errors::{BadInput, Refused};
use crate::menu::go_back;
use crate::menu::loop_store::LoopFlags;
use crate::menu::pages_store::MenuPages;
use crate::owners::Owners;
use crate::saves::gather_savegame::{CAR_LEN, RECORDS_LEN, cars, head};
use crate::saves::read_savegame_slot::read_savegame_slot_in;
use crate::saves::write_savegame_slot::SLOTS;
use rc2k_core::saves::savegame::{A8_MODE, CORRUPT, Resume, resume};

/// Where the saved game starts in the file: what `gather_savegame` wrote.
const PAYLOAD_OFFSET: usize = 0x30;

/// The menu sound of a slot that cannot be read.
const UNREADABLE_SOUND: u32 = 8;

/// The slot the highlighted menu item stands for.
fn highlighted_slot(pages: &MenuPages, loop_flags: &LoopFlags) -> u32 {
    rc2k_core::saves::savegame::slot(pages.read().save_page, loop_flags.read().item)
}

/// The players' own records and names come back when the loaded championship is over or left
/// (`menu::own_settings`).
pub fn load_savegame(o: Owners<'static>) -> Result<(), Refused> {
    crate::menu::own_settings::around_championship(o.menus, || load_and_run(o))
}

fn load_and_run(o: Owners<'static>) -> Result<(), Refused> {
    let (menus, saves) = (o.menus, o.saves);
    let slot = highlighted_slot(&menus.tables.pages, &menus.screens.loop_flags);
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    cars(&menus.play)?;

    crate::menu::ready_players::ready_players(menus)?;
    menus.screens.event.event_goes_on.set(1u32);
    menus.play.champ_view.reset_times.set(0u32);
    menus.screens.event.event_out.set(0u32);
    if !read_savegame_slot_in(saves, slot)? {
        crate::sound::play_menu_sound::play_menu_sound(
            o.sound,
            o.audio,
            o.menus,
            &o.system.session.picture_clock,
            UNREADABLE_SOUND,
        );
        go_back::go_back(menus);
        return Ok(());
    }

    let restored = restore(o);
    match restored {
        Ok(Resume::NextSection) => {
            menus
                .screens
                .event
                .section
                .set(menus.screens.event.section.get().wrapping_add(1));
            menus.screens.event.stage_in_section.set(0u32);
        }
        Ok(Resume::NextStage) => menus
            .screens
            .event
            .stage_in_section
            .set(menus.screens.event.stage_in_section.get().wrapping_add(1)),
        Ok(Resume::Over) => {
            go_back::go_back(menus);
            return Ok(());
        }
        // A file that is not a saved game's: the player is told and is back at the menu; the
        // game ends only for what is not the file's doing.
        Ok(Resume::Corrupt) => return damaged(menus, slot, "the saved game is corrupt"),
        Err(Refused::BadInput(why)) => return damaged(menus, slot, why.0),
        Err(Refused::Fatal(error)) if error.code == CORRUPT => {
            return damaged(menus, slot, error.message);
        }
        Err(other) => return Err(other),
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

/// The saved game in the slot's file put back into the game, and where the event stands. A
/// file that does not hold what a saved game does is refused here, before the event runs.
fn restore(o: Owners<'static>) -> Result<Resume, Refused> {
    let (menus, saves) = (o.menus, o.saves);
    let cars = cars(&menus.play)?;
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
    crate::menu::drivers_store::records_from_bytes(&menus.cars.drivers.drivers, &records)?;
    menus.screens.flow.write().start_section = cars[0].get().saved_start_section;
    menus.screens.flow.write().start_section_a8 = cars[0].get().saved_start_section_a8;

    Ok(resume(crate::menu::next_record_kind::next_record_kind(
        menus, o.system,
    )?))
}

/// A saved game that cannot be loaded: logged, the player told, back at the menu.
fn damaged(menus: &crate::menu::owner::Menus, slot: u32, why: &str) -> Result<(), Refused> {
    crate::program_log::log(&format!(
        "saves::load_savegame: slot {slot} is damaged: {why}"
    ));
    crate::error_dialog::notice(&crate::error_dialog::text_with(
        rc2k_core::strings::own::id::DAMAGED_SAVEGAME,
        &(slot + 1).to_string(),
    ));
    go_back::go_back(menus);
    Ok(())
}
