//! `scene::draw_carousel` (0x490649): the entries of the main menu's carousel that
//! can be seen.
//!
//! In:  ecx, passed on for a carousel of one entry
//! Out: ebx = the last entry drawn, ecx = its place in the row (for one entry: ebx = 0,
//!      edx = 0, ecx as it came); nothing changed but ebx for a carousel of none
//!
//! With one entry (`g_carouselCount` = 1): entry 0, the place whatever ecx holds.
//! With none or fewer: nothing. Else the entry before the front one
//! (`g_carouselFront`, going round) at place -1, the front one at 0, the next at 1,
//! and while the carousel turns (`g_carouselAngle` not 0) the one after at 2; each
//! through `scene::draw_carousel_entry`.
//! One caller, `scene::draw_main_menu_carousel`.
//! Shadow mode runs the original alone (each entry is compared in its own hook).
//! Self-test: a redirected copy whose calls note the entry and its place.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::scene::draw_carousel_entry::draw_carousel_entry;
use crate::scene::draw_menu_object::draw_menu_object_in_game;

// ---- game data ----

const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const FRONT_RVA: usize = rva(0x75AA7C); // g_carouselFront
const ANGLE_RVA: usize = rva(0x75AA80); // g_carouselAngle

// ---- Rust version ----

/// `entry`: what stands for `scene::draw_carousel_entry`, given the entry and its row
/// place. `lone_row`: ecx at the original's entry, the row place a lone entry is drawn at
/// (the original sets ebx and edx for it, not ecx).
// Converted: `rc2k_core::scene::carousel::drawn`
pub unsafe fn draw_carousel(
    lone_row: u32,
    entry: &mut dyn FnMut(u32, u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    use rc2k_core::scene::carousel::{Drawn, drawn};
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        match drawn(
            count,
            || game_read(FRONT_RVA),
            || game_read::<u32>(ANGLE_RVA) != 0,
        ) {
            Drawn::Nothing => Ok(()),
            Drawn::Lone => entry(0, lone_row),
            Drawn::Row(entries) => {
                for (e, place) in entries {
                    entry(e, place)?;
                }
                Ok(())
            }
        }
    }
}

/// With the Rust versions of what it calls.
pub unsafe fn draw_carousel_in_game(lone_row: u32) -> Result<(), Refused> {
    unsafe {
        draw_carousel(lone_row, &mut |e, place| {
            draw_carousel_entry(e, place, &mut || draw_menu_object_in_game())
        })
    }
}
