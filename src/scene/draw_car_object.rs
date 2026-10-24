//! `scene::draw_car_object` (0x49017B): the 3D part of a frame of the car
//! specification screen.
//!
//! In:  nothing
//! Out: the registers as its parts leave them (the scene lists' caller keeps them all);
//!      nothing at all on a page other than the first
//!
//! Only with `dword_755998` (the page shown) at 0: `scene::clear_depth_buffer`,
//! `scene::assign_kind_textures`, `sub_490238`, `scene::map_texture_places`,
//! `scene::load_slot_textures`, `sub_4901A3`. The three under it have one caller each
//! and are inlined:
//! - `sub_490238`: with the carousel menu (`g_carouselMenu`) at 0xB, the entry before
//!   `dword_755990` (the car shown) and the three after it, wrapping at
//!   `g_carouselCount`, to `dword_75AAA8` .., then `scene::update_carousel_pictures`;
//! - `sub_4901A3` -> `sub_4901B3` -> `sub_49040D`: `dword_6365C4` = 0; the turn about y
//!   from the clock (times 8, sixteen bits), the place's z `dword_75A66C`, the kind
//!   `dword_75A674[dword_755990]` to `dword_75A658`, its object to `g_objectNumber`,
//!   `scene::draw_menu_object` with eax = it (between `pusha` and `popa`).
//!
//! In the scene list at 0x758F76; IDA had not made it a function. Refuses an entry past the
//! 0x100 kinds, and what its parts refuse. The menus and draw lists run its `run`
//! (`menu::table_routes`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::carousel_pictures::update;
use crate::scene::clear_depth_buffer::clear_depth_buffer;
use crate::scene::draw_carousel_entry::{
    ANGLE_Y_RVA, CLOCK_RVA, ENTRIES, KIND_RVA, KINDS_RVA, OBJECT_RVA, PLACE_RVA,
    find_object_of_kind,
};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- game data ----

const PAGE_RVA: usize = rva(0x755998); // dword_755998
const CAR_RVA: usize = rva(0x755990); // dword_755990: the entry shown
const MENU_RVA: usize = rva(0x75AA74); // g_carouselMenu
const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const PICKED_RVA: usize = rva(0x75AAA8); // dword_75AAA8 .. dword_75AAB4
const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4
const Z_RVA: usize = rva(0x75A66C); // dword_75A66C

// ---- Rust version ----

/// `sub_490238`'s four entries around the car shown.
pub fn picked(car: u32, count: u32) -> [u32; 4] {
    let mut entry = if car == 0 { count } else { car }.wrapping_sub(1);
    let mut entries = [entry; 4];
    for next in &mut entries[1..] {
        entry = entry.wrapping_add(1);
        if entry == count {
            entry = 0;
        }
        *next = entry;
    }
    entries
}

/// `sub_490238`.
pub unsafe fn pick_pictures() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(MENU_RVA) != 0xB {
            return Ok(());
        }
        let entries = picked(game_read(CAR_RVA), game_read(COUNT_RVA));
        game_write(PICKED_RVA, entries);
        update(entries)?;
        Ok(())
    }
}

/// `sub_49040D`: the car shown's object set up and drawn by `draw`.
pub unsafe fn draw_shown_car(draw: &mut dyn FnMut() -> Result<(), Refused>) -> Result<(), Refused> {
    unsafe {
        let car: u32 = game_read(CAR_RVA);
        if car >= ENTRIES {
            return Err(BadInput("a car shown past the carousel's kinds").into());
        }
        let clock: u32 = game_read(CLOCK_RVA);
        game_write(ANGLE_Y_RVA, (clock << 3) & 0xFFFF);
        game_write(PLACE_RVA + 8, game_read::<u32>(Z_RVA));
        let kind: u32 = game_read(KINDS_RVA + car as usize * 4);
        game_write(KIND_RVA, kind);
        let object = find_object_of_kind(kind);
        game_write(OBJECT_RVA, object);
        draw()
    }
}

/// The Rust version the typed tables run: nothing on a page other than the first, else
/// its six calls in order. It reads no register of its caller's (register liveness,
/// 2026-10-23), and the drawing reads none of what the calls before it leave (the eax and
/// ebx `sub_49040D` gives it included).
pub fn run() -> Result<(), crate::errors::Refused> {
    unsafe {
        if game_read::<u32>(PAGE_RVA) != 0 {
            return Ok(());
        }
        clear_depth_buffer();
        assign_kind_textures()?;
        pick_pictures()?;
        map_texture_places();
        load_slot_textures_in_game()?;
        game_write(MODE_RVA, 0u32);
        draw_shown_car(&mut || draw_menu_object_in_game())
    }
}
