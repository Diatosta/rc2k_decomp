//! `menu::run_front_end` (0x47486C): the game between its start and its end: the films
//! and the main menu, with everything that is reached from it.
//!
//! In:  nothing
//! Out: eax = 0; everything else preserved
//! `dword_755684` = `dword_4F82D8`; `menu::drop_missing_controllers`; `gfx::build_fonts`;
//! the films at the start (`sub_4161CD` with eax = 0, not ported); the log line "BEV:
//! Starting GB_Front"; `menu::main_menu_loop`; "BEV: Finished GB_Front";
//! `menu::store_player_names`; `gfx::free_fonts`; `saves::list_car_numbers`.
//! Callers: `system::main_loop` and 0x443260. Refuses what its parts refuse; such a refusal comes
//! with the work half done. Rust callers run its `entry` (`routes.rs`); the registers above are the
//! original's, for reference.

use crate::audio::store::AUDIO;
use crate::errors::Refused;
use crate::framework::{log, rva};
use crate::menu::screen_loop::{Carried, run_screen};
use crate::store::{game_addr, game_write};

// ---- game data ----

const TO_RVA: usize = rva(0x755684); // dword_755684
const STARTING_RVA: usize = rva(0x4DE67B); // "BEV: Starting GB_Front"
const FINISHED_RVA: usize = rva(0x4DE692); // "BEV: Finished GB_Front"

// ---- Rust version ----

/// The Rust version (the original keeps ebp, ecx, edi and edx; it leaves eax 0 and esi
/// as the main menu does, which its callers do not read).
pub fn run_front_end() -> Result<(), Refused> {
    log("menu::run_front_end: the Rust version runs");
    unsafe {
        game_write(TO_RVA, AUDIO.master_volume.get());
        crate::menu::drop_missing_controllers::drop_missing_controllers()?;
        crate::gfx::build_fonts::build_fonts()?;
        // The films, with eax 0 (the game's start).
        crate::menu::play_films::play_films(false)?;
        crate::menu::text_entry::log_l1(game_addr(STARTING_RVA) as *const u8, true);
        run_screen(&crate::menu::main_menu_loop::SCREEN, &Carried::default())?;
        crate::menu::text_entry::log_l1(game_addr(FINISHED_RVA) as *const u8, true);
        crate::menu::store_player_names::store_player_names(&mut |line| {
            crate::menu::text_entry::log_l1(line, true)
        })?;
        crate::gfx::free_fonts::free_fonts(
            game_addr(crate::gfx::glyph_tables::TABLES_LIST_RVA) as *mut crate::memory::MemoryList,
            game_addr(crate::gfx::build_fonts::PALETTES_RVA) as *mut u32,
        )?;
        crate::saves::list_car_numbers::list_car_numbers()?;
    }
    log("menu::run_front_end: the Rust version has come back");
    Ok(())
}
