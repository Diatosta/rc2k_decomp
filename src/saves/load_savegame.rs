//! `saves::load_savegame` (0x46E6E1): the action of a slot on the "load game" menu: reads the
//! saved game, puts its state back into the game and goes on with the event from there.
//!
//! In:  nothing
//! Out: every register preserved
//! In order:
//!   - `sub_471F7E`; `dword_75592C` = 1, `dword_755250` = 0, `dword_755978` = 0;
//!   - `read_savegame_slot` on slot `dword_75523C * 10 + item - 1`. If that fails:
//!     `sub_40F059` with eax = 8, `menu::go_back`, and that is all;
//!   - the seven dwords, the four cars and the 180 records go back where `gather_savegame`
//!     took them from; `dword_75516C` is left pointing at the saved cars; between the dwords
//!     and the cars it calls `maybeCopySomeData` (only if `dword_755144` is 0) and
//!     `sub_4748B6`. Each car's 0x20 bytes at +0x124 are copied to where the car's pointer at
//!     +0x18 pointed before the car was overwritten; car 0's dwords at +0x14 and +0x274 go to
//!     `dword_7554C0` and `dword_7554C4`;
//!   - `sub_470D08` says where the event stands: 4 or 5, the next section starts (section + 1,
//!     stage index 0); 0 or 1, the next stage (index + 1); 3, nothing more happens here; any
//!     other value is fatal error 0x80702821 "Corrupt SaveGame";
//!   - unless it was 3: `dword_755970` = 1, `sub_47059E`, and then the event itself runs:
//!     `sub_4AA4D7` if `dword_7556D0` is 0x13, else `sub_4A0E60`. These only come back when
//!     the event is over or left;
//!   - `menu::go_back`.
//!
//! `sub_471F7E` (`menu::ready_players`), `maybeCopySomeData` (`saves::keep_player_settings`,
//! esi the saved cars) and the two event loops (`menu::run_championship`,
//! `menu::run_a8_championship`) are their Rust versions; the other four (`sub_4748B6`,
//! `sub_470D08`, `sub_47059E`, `sub_40F059`) are called with `call_game` (three of them through
//! their hooks), with the registers the original has at each call. Ten menu tables point at
//! this function. Refuses, before anything is done: a slot past 99, a missing player or a car
//! without its pointer at +0x18; later, what its Rust callees refuse (the event loops when
//! their callees refuse; the original then runs from the start). Hook removed 2026-10-22 (typed
//! tables): the menus and draw lists run its `entry` (`menu::table_calls`); the registers above
//! are the original's, for reference.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::go_back;
use crate::menu::name_lists::SAVEGAME_PAGE_RVA;
use crate::saves::gather_savegame::{
    CAR_LEN, CAR0_A_RVA, CAR0_B_RVA, HEAD_RVAS, NAME_LEN, NAME_OFFSET, NAME_PTR_OFFSET,
    RECORDS_LEN, RECORDS_RVA, cars,
};
use crate::saves::read_savegame_slot::read_savegame_slot;
use crate::saves::write_savegame_slot::{FILE_RVA, SLOTS};
use crate::service::state::menu_item;
use rc2k_core::saves::savegame::{A8_MODE, CORRUPT, Resume, resume};
use std::ptr::copy_nonoverlapping;

// ---- game data ----

const LOADING_RVA: usize = rva(0x75592C); // dword_75592C: set to 1
const CLEARED_A_RVA: usize = rva(0x755250); // dword_755250: set to 0
const CLEARED_B_RVA: usize = rva(0x755978); // dword_755978: set to 0
const SAVED_CARS_PTR_RVA: usize = rva(0x75516C); // dword_75516C: the saved cars in the buffer
const SKIP_COPY_RVA: usize = rva(0x755144); // dword_755144: 0 = `maybeCopySomeData` is called
const CONTINUING_RVA: usize = rva(0x755970); // dword_755970: set to 1 before the event runs
const INDEX_RVA: usize = rva(0x755958); // index: the event's stage in its section
const SECTION_RVA: usize = rva(0x75595C); // section
const MODE_RVA: usize = rva(0x7556D0); // dword_7556D0: 0x13 picks the other event loop
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

/// Calls one of them with the registers given, and leaves the registers it returns.
pub type Game<'a> = &'a mut dyn FnMut(Callee, &mut Regs) -> Result<(), Refused>;

fn in_game(callee: Callee, regs: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match callee {
            Callee::Prepare => crate::menu::ready_players::ready_players()?,
            Callee::CopySomeData => {
                crate::saves::keep_player_settings::keep_player_settings(regs.esi)?
            }
            // (The championship drivers keep every register of their caller.)
            Callee::RunEvent => crate::menu::run_championship::run_championship(
                &crate::menu::screen_loop::carried(regs),
            )?,
            Callee::RunEventMode13 => crate::menu::run_a8_championship::run_a8_championship(
                &crate::menu::screen_loop::carried(regs),
            )?,
            _ => crate::calls::call(rva(callee as usize), regs),
        }
    }
    Ok(())
}

/// The slot the highlighted menu item stands for.
fn highlighted_slot() -> u32 {
    // Converted: `rc2k_core::saves::savegame::slot`.
    rc2k_core::saves::savegame::slot(unsafe { game_read::<u32>(SAVEGAME_PAGE_RVA) }, menu_item())
}

// ---- Rust version ----

/// `entry` holds the registers at entry: the unported callees get the registers the
/// original would have at each call.
pub unsafe fn load_savegame(entry: &Regs, game: Game) -> Result<(), Refused> {
    let slot = highlighted_slot();
    if slot >= SLOTS {
        return Err(BadInput("slot past the name list").into());
    }
    let cars = cars()?;
    let mut g = *entry;

    unsafe {
        game(Callee::Prepare, &mut g)?;
        game_write(LOADING_RVA, 1u32);
        game_write(CLEARED_A_RVA, 0u32);
        game_write(CLEARED_B_RVA, 0u32);
        g.ebx = slot;
        g.eax = 0;
        if !read_savegame_slot(slot)? {
            g.eax = 8;
            game(Callee::Refuse, &mut g)?;
            go_back::go_back();
            return Ok(());
        }

        let payload = game_addr(FILE_RVA) + PAYLOAD_OFFSET;
        let mut cursor = payload as *const u8;
        for at in HEAD_RVAS {
            g.eax = (cursor as *const u32).read_unaligned();
            game_write(at, g.eax);
            cursor = cursor.add(4);
        }
        game_write(SAVED_CARS_PTR_RVA, cursor as u32);
        g.esi = cursor as u32;
        if game_read::<u32>(SKIP_COPY_RVA) == 0 {
            game(Callee::CopySomeData, &mut g)?;
        }
        game(Callee::AfterHead, &mut g)?;

        for car in cars {
            // The pointer as it is before the car is overwritten: the saved one may be stale.
            let name = ((car + NAME_PTR_OFFSET) as *const u32).read_unaligned();
            copy_nonoverlapping(cursor, car as *mut u8, CAR_LEN);
            std::ptr::copy((car + NAME_OFFSET) as *const u8, name as *mut u8, NAME_LEN);
            cursor = cursor.add(CAR_LEN);
            (g.edi, g.ebp, g.ebx) = (car, name, car + NAME_OFFSET);
        }
        copy_nonoverlapping(cursor, game_addr(RECORDS_RVA) as *mut u8, RECORDS_LEN);
        g.esi = cursor.add(RECORDS_LEN) as u32;
        g.edx = 0xB4;
        g.edi = cars[0];
        game_write(
            CAR0_A_RVA,
            ((cars[0] + 0x14) as *const u32).read_unaligned(),
        );
        g.eax = ((cars[0] + 0x274) as *const u32).read_unaligned();
        game_write(CAR0_B_RVA, g.eax);

        // Converted: `rc2k_core::saves::savegame` (`resume`, `A8_MODE`).
        game(Callee::EventState, &mut g)?;
        match resume(g.eax) {
            Resume::NextSection => {
                game_write(SECTION_RVA, game_read::<u32>(SECTION_RVA).wrapping_add(1));
                game_write(INDEX_RVA, 0u32);
            }
            Resume::NextStage => game_write(INDEX_RVA, game_read::<u32>(INDEX_RVA).wrapping_add(1)),
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
        game_write(CONTINUING_RVA, 1u32);
        game(Callee::BeforeEvent, &mut g)?;
        if game_read::<u32>(MODE_RVA) == A8_MODE {
            game(Callee::RunEventMode13, &mut g)?;
        } else {
            game(Callee::RunEvent, &mut g)?;
        }
        go_back::go_back();
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { load_savegame(&entry, &mut in_game) }?;
    Ok(())
}
