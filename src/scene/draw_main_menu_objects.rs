//! `scene::draw_main_menu_objects` (0x490106): the 3D part of a frame of the menus
//! with a carousel.
//!
//! In:  ecx, passed on to the carousel
//! Out: every register preserved
//!
//! `scene::clear_depth_buffer`; `scene::assign_kind_textures`;
//! `scene::choose_carousel_pictures`; `scene::assign_menu_textures`;
//! `scene::map_texture_places`; `scene::load_slot_textures`; `scene::draw_main_menu_carousel`.
//! Each is entered with the registers the one before left. Reached through the menus' scene
//! lists. The menus and draw lists run its `run` (`menu::table_routes`); the registers above
//! are the original's, for reference.

use crate::menu::owner::menus;
use crate::race::owner::race;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::choose_carousel_pictures::choose_carousel_pictures;
use crate::scene::clear_depth_buffer::clear_depth_buffer;
use crate::scene::draw_main_menu_carousel::draw_main_menu_carousel;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- Rust version ----

/// The Rust version the typed tables run: its seven calls in order. It reads no register
/// of its caller's (register liveness, 2026-10-23). Of what the calls leave in the
/// registers one value is read: the carousel's ecx, the slots-in-use count the textures'
/// loading leaves (the row place of a lone entry).
pub fn run() -> Result<(), crate::errors::Refused> {
    let race = race();
    let menus = menus();
    clear_depth_buffer();
    assign_kind_textures()?;
    choose_carousel_pictures(menus)?;
    assign_menu_textures(&menus.tables)?;
    map_texture_places();
    let count = load_slot_textures_in_game()?;
    draw_main_menu_carousel(menus, race, count)
}
