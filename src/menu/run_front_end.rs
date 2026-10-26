//! `menu::run_front_end` (0x47486C): the game between its start and its end: the films
//! and the main menu, with everything that is reached from it.
//!
//! In:  nothing
//! Out: eax = 0; everything else preserved
//! `dword_755684` = `dword_4F82D8`; `menu::drop_missing_controllers`; `gfx::build_fonts`;
//! the films at the start (`sub_4161CD` with eax = 0, not ported); the log line "BEV:
//! Starting GB_Front"; `menu::main_menu_loop`; "BEV: Finished GB_Front";
//! `menu::store_player_names`; `gfx::free_fonts`; `saves::list_car_numbers`.
//! Callers: `system::main_loop` and 0x443260.
//! Each real call writes a line to hook.log saying which version runs. Refuses what its parts
//! refuse; such a refusal comes with the work half done.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, log, rva};

// ---- game data ----

const FROM_RVA: usize = rva(0x4F82D8); // dword_4F82D8
const TO_RVA: usize = rva(0x755684); // dword_755684
const STARTING_RVA: usize = rva(0x4DE67B); // "BEV: Starting GB_Front"
const FINISHED_RVA: usize = rva(0x4DE692); // "BEV: Finished GB_Front"

// ---- Rust version ----

/// What the function calls, in the order of the test's stand-ins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    DropMissingControllers,
    BuildFonts,
    /// The films, with eax = 0.
    Films,
    /// A line of the game's log: the text's address.
    Log(u32),
    MainMenuLoop,
    StorePlayerNames,
    FreeFonts,
    ListCarNumbers,
}

pub type Outside<'a> = &'a mut dyn FnMut(Call) -> Result<(), Refused>;

pub unsafe fn run_front_end(outside: Outside) -> Result<(), Refused> {
    unsafe {
        game_write(TO_RVA, game_read::<u32>(FROM_RVA));
        outside(Call::DropMissingControllers)?;
        outside(Call::BuildFonts)?;
        outside(Call::Films)?;
        outside(Call::Log(game_addr(STARTING_RVA) as u32))?;
        outside(Call::MainMenuLoop)?;
        outside(Call::Log(game_addr(FINISHED_RVA) as u32))?;
        outside(Call::StorePlayerNames)?;
        outside(Call::FreeFonts)?;
        outside(Call::ListCarNumbers)
    }
}

/// The calls done by the Rust versions; the films and the menu's loop through the game
/// (the loop's own hook decides which version of it runs).
fn in_game(call: Call) -> Result<(), Refused> {
    unsafe {
        match call {
            Call::DropMissingControllers => {
                crate::menu::drop_missing_controllers::drop_missing_controllers()?
            }
            Call::BuildFonts => crate::calls::call(rva(0x415563), &mut Regs::default()),
            Call::Films => crate::calls::call(rva(0x4161CD), &mut Regs::default()),
            Call::Log(text) => crate::menu::text_entry::log_l1(text as *const u8, true),
            Call::MainMenuLoop => crate::calls::call(rva(0x4748D9), &mut Regs::default()),
            Call::StorePlayerNames => {
                crate::menu::store_player_names::store_player_names(&mut |line| {
                    crate::menu::text_entry::log_l1(line, true)
                })?
            }
            Call::FreeFonts => crate::gfx::free_fonts::free_fonts(
                game_addr(crate::gfx::glyph_tables::TABLES_LIST_RVA)
                    as *mut crate::memory::MemoryList,
                game_addr(crate::gfx::build_fonts::PALETTES_RVA) as *mut u32,
            )?,
            Call::ListCarNumbers => crate::saves::list_car_numbers::list_car_numbers()?,
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    log("menu::run_front_end: the Rust version runs");
    match unsafe { run_front_end(&mut in_game) } {
        Ok(()) => {
            r.eax = 0;
            log("menu::run_front_end: the Rust version has come back");
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
