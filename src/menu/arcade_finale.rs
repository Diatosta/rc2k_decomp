//! `menu::arcade_finale` (0x488155): after an arcade level is won: `dword_75B100` player
//! 0's +0x28, `dword_75B104` and `dword_75ACE8` its +8, `dword_75ACE4` 0, `dword_75B0FC`
//! 5 - the level, `dword_75B108` 0; then the race's sound objects and sounds
//! (`race::set_up_race_sound_objects`, `sound::set_up_race_sounds`), the loading picture,
//! the 3D tidied (`render3d_tidy`, `objects_tidy`, `nullsub_6`, `release_textures`),
//! `drive::finish_race_0`, the music stopped, the menu's screen mode, the loading picture,
//! `scene::set_up_engine`, `set_up_front_end_scene`, `set_up_engine` again, the music
//! stopped, tune 3, and the pending key taken (eax 0). What `finish_race_0` shows is a
//! reading (the level's ending drive).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! The callees get eax 5 - level, ebx the level and edi player 0's record at first, then
//! what the one before left. All at the game's addresses (their hooks choose) but the key's
//! taking (its Rust version). Refuses (before anything) a missing player 0. Ungated thunk;
//! shadow mode runs the original alone. Self-test: a copy whose 17 calls go to stand-ins,
//! against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::parts::{Part, parted_entry};
use crate::menu::player::player_at;

/// The callees by place.
const CALLEES: [usize; 14] = [
    0x415C93, // 0 race::set_up_race_sound_objects
    0x40F02E, // 1 sound::set_up_race_sounds
    0x47CEA8, // 2 menu::show_loading_picture
    0x4433A7, // 3 race::render3d_tidy
    0x44346B, // 4 race::objects_tidy
    0x44347E, // 5 nullsub_6
    0x443396, // 6 race::release_textures
    0x4278C7, // 7 drive::finish_race_0
    0x40F826, // 8 sound::stop_music
    0x404823, // 9 gfx::set_menu_screen_mode
    0x442AF6, // 10 scene::set_up_engine
    0x474E04, // 11 scene::set_up_front_end_scene
    0x40F780, // 12 sound::play_tune_3
    0x407F95, // 13 input::take_key
];
/// The calls in order, by place.
const ORDER: [u32; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 2, 10, 11, 10, 8, 12];

fn p(a: usize, v: u32) {
    unsafe { game_write(rva(a), v) }
}

/// The Rust version: the registers it leaves.
pub unsafe fn arcade_finale(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let player0 = player_at(0);
    if player0 == 0 {
        return Err(BadInput("no player 0").into());
    }
    let f = |o: u32| unsafe { ((player0 + o) as *const u32).read_unaligned() };
    let mut r = *entry;
    r.edi = player0;
    p(0x75B100, f(0x28));
    p(0x75B104, f(8));
    p(0x75ACE8, f(8));
    p(0x75ACE4, 0);
    r.ebx = unsafe { game_read(rva(0x7587EC)) };
    // Converted: `rc2k_core::menu::arcade::levels_left`.
    r.eax = rc2k_core::menu::arcade::levels_left(r.ebx);
    p(0x75B0FC, r.eax);
    p(0x75B108, 0);
    for place in ORDER {
        part(place, &mut r)?;
    }
    r.eax = 0;
    part(13, &mut r)?;
    Ok(*entry)
}

/// In the game: the key's taking in Rust, the rest at the game's addresses.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 13 {
        r.eax = r.eax & !0xFF | crate::input::take_key::take_key() as u32;
    } else {
        crate::calls::call(rva(CALLEES[place as usize]), r);
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(r, &|r, part| unsafe { arcade_finale(r, part) }, game_part)
}
