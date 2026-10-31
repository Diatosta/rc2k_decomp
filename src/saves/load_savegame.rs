//! `saves::load_savegame` (0x46E6E1): the action of a slot on the "load game" menu: reads the
//! saved game, puts its state back into the game and goes on with the event from there.
//!
//! In:  nothing
//! Out: every register preserved
//! In order:
//! - `sub_471F7E`; `dword_75592C` = 1, `dword_755250` = 0, `dword_755978` = 0;
//! - `read_savegame_slot` on slot `dword_75523C * 10 + item - 1`. If that fails:
//!   `sub_40F059` with eax = 8, `menu::go_back`, and that is all;
//! - the seven dwords, the four cars and the 180 records go back where `gather_savegame`
//!   took them from; `dword_75516C` is left pointing at the saved cars; between the dwords
//!   and the cars it calls `maybeCopySomeData` (only if `dword_755144` is 0) and
//!   `sub_4748B6`. Each car's 0x20 bytes at +0x124 are copied to where the car's pointer at
//!   +0x18 pointed before the car was overwritten; car 0's dwords at +0x14 and +0x274 go to
//!   `dword_7554C0` and `dword_7554C4`;
//! - `sub_470D08` says where the event stands: 4 or 5, the next section starts (section + 1,
//!   stage index 0); 0 or 1, the next stage (index + 1); 3, nothing more happens here; any
//!   other value is fatal error 0x80702821 "Corrupt SaveGame";
//! - unless it was 3: `dword_755970` = 1, `sub_47059E`, and then the event itself runs:
//!   `sub_4AA4D7` if `dword_7556D0` is 0x13, else `sub_4A0E60`. These only come back when
//!   the event is over or left;
//! - `menu::go_back`.
//!
//! Ten menu tables point at this function. Refuses, before anything is done: a slot past 99, a
//! missing player or a car without its pointer at +0x18; later, what its Rust callees refuse (the
//! event loops when their callees refuse). The menus and
//! draw lists run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::go_back;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_loop::Carried;
use crate::pointers::GamePtr;
use crate::saves::gather_savegame::{
    CAR_LEN, NAME_LEN, NAME_OFFSET, NAME_PTR_OFFSET, RECORDS_LEN, cars, head,
};
use crate::saves::preview_store::SAVEGAME_PREVIEW;
use crate::saves::read_savegame_slot::read_savegame_slot;
use crate::saves::write_savegame_slot::{SAVE_GAMES, SLOTS};
use crate::service::state::menu_item;
use rc2k_core::saves::savegame::{A8_MODE, CORRUPT, Resume, resume};

// ---- game data ----

const PAYLOAD_OFFSET: usize = 0x30; // 0x51EBE4: what `gather_savegame` wrote

/// The unported functions it calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Callee {
    Prepare = 0x471F7E,        // sub_471F7E
    CopySomeData = 0x4719F5,   // maybeCopySomeData
    AfterHead = 0x4748B6,      // sub_4748B6
    EventState = 0x470D08,     // sub_470D08: eax = where the event stands
    BeforeEvent = 0x47059E,    // sub_47059E
    RunEventMode13 = 0x4AA4D7, // sub_4AA4D7
    RunEvent = 0x4A0E60,       // sub_4A0E60
    Refuse = 0x40F059,         // sub_40F059, with eax = 8 when the slot cannot be read
}

/// Calls one of them with the carried values given (the original's eax .. esi), and
/// leaves what it returns in them.
pub type Game<'a> = &'a mut dyn FnMut(Callee, &mut Carried) -> Result<(), Refused>;

fn in_game(callee: Callee, regs: &mut Carried) -> Result<(), Refused> {
    match callee {
        Callee::Prepare => crate::menu::ready_players::ready_players()?,
        Callee::CopySomeData => {
            crate::saves::keep_player_settings::keep_player_settings(regs.cars)?;
        }
        // (The championship drivers keep every register of their caller.)
        Callee::RunEvent => crate::menu::run_championship::run_championship(regs)?,
        Callee::RunEventMode13 => crate::menu::run_a8_championship::run_a8_championship(regs)?,
        Callee::AfterHead => crate::menu::drop_missing_controllers::drop_missing_controllers()?,
        Callee::EventState => regs.arg = crate::menu::next_record_kind::next_record_kind()?,
        Callee::BeforeEvent => crate::menu::link_player_names::link_player_names()?,
        Callee::Refuse => crate::sound::play_menu_sound::play_menu_sound_in_game(regs.arg),
    }
    Ok(())
}

/// The slot the highlighted menu item stands for.
fn highlighted_slot() -> u32 {
    // Converted: `rc2k_core::saves::savegame::slot`.
    rc2k_core::saves::savegame::slot(MENU_PAGES.save_page.get(), menu_item())
}

// ---- Rust version ----

/// `entry` holds the carried values at entry (the original's eax .. esi, no carry): the
/// callees get them as the original's registers stand at each call (edi and ebp, which it
/// also writes, are read by none of them).
pub fn load_savegame(entry: &Carried, game: Game) -> Result<(), Refused> {
    let slot = highlighted_slot();
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let cars = cars()?;
    let mut g = *entry;

    game(Callee::Prepare, &mut g)?;
    EVENT_STATE.event_goes_on.set(1u32);
    MENU_PAGES.reset_times.set(0u32);
    EVENT_STATE.event_out.set(0u32);
    g.palette = slot;
    g.arg = 0;
    if !read_savegame_slot(slot)? {
        g.arg = 8;
        game(Callee::Refuse, &mut g)?;
        go_back::go_back();
        return Ok(());
    }

    let payload = SAVE_GAMES.file.addr() + PAYLOAD_OFFSET as u32;
    let mut cursor = GamePtr::new(payload);
    for at in head() {
        g.arg = cursor.read::<u32>("the savegame's head")?;
        at.set(g.arg);
        cursor = cursor.offset(4);
    }
    SAVEGAME_PREVIEW.saved_cars.set(cursor.addr());
    g.cars = cursor.addr();
    if SAVEGAME_PREVIEW.use_savegame_keys.get() == 0 {
        game(Callee::CopySomeData, &mut g)?;
    }
    game(Callee::AfterHead, &mut g)?;

    for car in cars {
        let name = GamePtr::new(car + NAME_PTR_OFFSET).read::<u32>("a player's record")?;
        cursor.copy_to(GamePtr::new(car), CAR_LEN, "a saved car")?;
        GamePtr::new(car + NAME_OFFSET).copy_to(GamePtr::new(name), NAME_LEN, "a player's name")?;
        cursor = cursor.offset(CAR_LEN as u32);
        g.palette = car + NAME_OFFSET; // ebx (edi the car, ebp the name: unread)
    }
    cursor.copy_to(
        GamePtr::new(crate::menu::drivers_store::drivers_addr()),
        RECORDS_LEN,
        "the saved drivers",
    )?;
    g.cars = cursor.offset(RECORDS_LEN as u32).addr();
    g.clock = 0xB4;
    SCREEN_FLOW
        .start_section
        .set(GamePtr::new(cars[0] + 0x14).read::<u32>("a player's record")?);
    g.arg = GamePtr::new(cars[0] + 0x274).read::<u32>("a player's record")?;
    SCREEN_FLOW.start_section_a8.set(g.arg);

    game(Callee::EventState, &mut g)?;
    match resume(g.arg) {
        Resume::NextSection => {
            EVENT_STATE
                .section
                .set(EVENT_STATE.section.get().wrapping_add(1));
            EVENT_STATE.index.set(0u32);
        }
        Resume::NextStage => EVENT_STATE
            .index
            .set(EVENT_STATE.index.get().wrapping_add(1)),
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
    EVENT_STATE.continuing_save.set(1u32);
    game(Callee::BeforeEvent, &mut g)?;
    if CHAMP_VIEW.event_mode.get() == A8_MODE {
        game(Callee::RunEventMode13, &mut g)?;
    } else {
        game(Callee::RunEvent, &mut g)?;
    }
    go_back::go_back();
    Ok(())
}

/// The Rust version where the typed tables run it (a refusal is the
/// caller's).
pub fn entry(c: &Carried) -> Result<(), crate::errors::Refused> {
    load_savegame(c, &mut in_game)
}
