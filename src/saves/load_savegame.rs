//! `saves::load_savegame` (0x46E6E1): the action of a slot on the "load game" menu: reads the
//! saved game, puts its state back into the game and goes on with the event from there.
//!
//! In:  nothing
//! Out: every register preserved
//! In order:
//! - `menu::ready_players`; `dword_75592C` = 1, `dword_755250` = 0, `dword_755978` = 0;
//! - `read_savegame_slot` on slot `dword_75523C * 10 + item - 1`. If that fails: menu
//!   sound 8, `menu::go_back`, and that is all;
//! - the seven dwords, the four cars and the 180 records go back where `gather_savegame`
//!   took them from; `dword_75516C` is left pointing at the saved cars; between the dwords
//!   and the cars it calls `saves::keep_player_settings` (only if `dword_755144` is 0) and
//!   `menu::drop_missing_controllers`. Each car's 0x20 bytes at +0x124 are copied to where the car's pointer at
//!   +0x18 pointed before the car was overwritten; car 0's dwords at +0x14 and +0x274 go to
//!   `dword_7554C0` and `dword_7554C4`;
//! - `menu::next_record_kind` says where the event stands: 4 or 5, the next section starts (section + 1,
//!   stage index 0); 0 or 1, the next stage (index + 1); 3, nothing more happens here; any
//!   other value is fatal error 0x80702821 "Corrupt SaveGame";
//! - unless it was 3: `dword_755970` = 1, `menu::link_player_names`, and then the event
//!   itself runs: `menu::run_a8_championship` if `dword_7556D0` is 0x13, else
//!   `menu::run_championship`. These only come back when the event is over or left;
//! - `menu::go_back`.
//!
//! Ten menu tables point at this function. Refuses, before anything is done: a slot past 99, a
//! missing player or a car without its pointer at +0x18; later, what its Rust callees refuse (the
//! event loops when their callees refuse).

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::go_back;
use crate::menu::owner::{Menus, menus};
use crate::pointers::GamePtr;
use crate::saves::gather_savegame::{CAR_LEN, RECORDS_LEN, cars, head};
use crate::saves::owner::saves;
use crate::saves::read_savegame_slot::read_savegame_slot;
use crate::saves::write_savegame_slot::SLOTS;
use rc2k_core::saves::savegame::{A8_MODE, CORRUPT, Resume, resume};

// ---- game data ----

const PAYLOAD_OFFSET: usize = 0x30; // 0x51EBE4: what `gather_savegame` wrote

/// The menu sound of a slot that cannot be read (`sub_40F059`'s eax).
const UNREADABLE_SOUND: u32 = 8;

/// The slot the highlighted menu item stands for.
fn highlighted_slot(menus: &Menus) -> u32 {
    // Converted: `rc2k_core::saves::savegame::slot`.
    rc2k_core::saves::savegame::slot(
        menus.tables.pages.save_page.get(),
        menus.screens.loop_flags.item.get(),
    )
}

// ---- Rust version ----

/// The Rust version (see the header).
pub fn load_savegame() -> Result<(), Refused> {
    let saves = saves();
    let menus = menus();
    let slot = highlighted_slot(menus);
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let cars = cars(&menus.play)?;

    crate::menu::ready_players::ready_players()?;
    menus.screens.event.event_goes_on.set(1u32);
    menus.tables.pages.reset_times.set(0u32);
    menus.screens.event.event_out.set(0u32);
    if !read_savegame_slot(slot)? {
        crate::sound::play_menu_sound::play_menu_sound_in_game(UNREADABLE_SOUND);
        go_back::go_back();
        return Ok(());
    }

    let mut cursor = GamePtr::to(&saves.save_games.file).offset(PAYLOAD_OFFSET as u32);
    for at in head(menus) {
        at.set(cursor.read::<u32>("the savegame's head")?);
        cursor = cursor.offset(4);
    }
    if saves.savegame_preview.use_savegame_keys.get() == 0 {
        crate::saves::keep_player_settings::keep_player_settings(cursor)?;
    }
    crate::menu::drop_missing_controllers::drop_missing_controllers()?;

    for car in cars {
        let name = car.get().name_number;
        let saved: [u8; CAR_LEN] = cursor.read("a saved car")?;
        car.set(rc2k_core::menu::player::Player::from_file_bytes(saved));
        // The file's own pointer is not to be trusted: the game's stays (a save's holds the
        // same place, unless the file was damaged).
        car.update(|p| p.name_number = name);
        crate::menu::player_names::set_name_bytes(
            crate::menu::player_names::name_of(&car.get())?,
            &car.get().name_copy,
        );
        cursor = cursor.offset(CAR_LEN as u32);
    }
    cursor.copy_to(
        GamePtr::to(&menus.cars.drivers),
        RECORDS_LEN,
        "the saved drivers",
    )?;
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

    match resume(crate::menu::next_record_kind::next_record_kind()?) {
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
            go_back::go_back();
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
    crate::menu::link_player_names::link_player_names()?;
    if menus.play.champ_view.event_mode.get() == A8_MODE {
        crate::menu::run_a8_championship::run_a8_championship()?;
    } else {
        crate::menu::run_championship::run_championship()?;
    }
    go_back::go_back();
    Ok(())
}
