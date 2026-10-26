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
//! In the scene list at 0x758F76; IDA had not made it a function.
//! Refuses an entry past the 0x100 kinds, and what its parts refuse.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::carousel_pictures::update;
use crate::scene::clear_depth_buffer::clear_depth_buffer_in_game;
use crate::scene::draw_carousel_entry::{
    ANGLE_Y_RVA, CLOCK_RVA, ENTRIES, KIND_RVA, KINDS_RVA, OBJECT_RVA, PLACE_RVA,
    find_object_of_kind,
};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;

// ---- game data ----

const PAGE_RVA: usize = rva(0x755998); // dword_755998
const CAR_RVA: usize = rva(0x755990); // dword_755990: the entry shown
const MENU_RVA: usize = rva(0x75AA74); // g_carouselMenu
const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const PICKED_RVA: usize = rva(0x75AAA8); // dword_75AAA8 .. dword_75AAB4
const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4
const Z_RVA: usize = rva(0x75A66C); // dword_75A66C
const STEPS: [usize; 6] = [
    rva(0x44E967),
    rva(0x4256D8),
    rva(0x490238),
    rva(0x42570D),
    rva(0x425778),
    rva(0x4901A3),
];

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
pub unsafe fn pick_pictures(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(MENU_RVA) != 0xB {
            return Ok(());
        }
        let entries = picked(game_read(CAR_RVA), game_read(COUNT_RVA));
        game_write(PICKED_RVA, entries);
        update(entries)?.apply(r);
        Ok(())
    }
}

/// `sub_49040D`: the car shown's object set up and drawn by `draw`.
pub unsafe fn draw_shown_car(
    r: &Regs,
    draw: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
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
        let mut inner = Regs {
            eax: object,
            ebx: car << 2,
            ..*r
        };
        draw(&mut inner)
    }
}

/// `step`: what stands for the six calls, by their order. Nothing on a page other than
/// the first.
pub unsafe fn draw_car_object(
    r: &mut Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    if unsafe { game_read::<u32>(PAGE_RVA) } != 0 {
        return Ok(());
    }
    for n in 0..STEPS.len() {
        step(n, r)?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    let done = unsafe {
        draw_car_object(r, &mut |n, r| {
            match n {
                0 => clear_depth_buffer_in_game(r)?,
                1 => assign_kind_textures()?.apply(r),
                2 => pick_pictures(r)?,
                3 => map_texture_places().apply(r),
                4 => {
                    let entry = *r;
                    load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry))?
                        .apply(r)
                }
                _ => {
                    game_write(MODE_RVA, 0u32);
                    draw_shown_car(r, &mut |r| draw_menu_object_in_game(r))?;
                }
            }
            Ok(())
        })
    };
    if let Err(e) = done {
        *r = entry;
        return Err(e);
    }
    Ok(())
}
