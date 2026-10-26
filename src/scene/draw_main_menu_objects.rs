//! `scene::draw_main_menu_objects` (0x490106): the 3D part of a frame of the menus
//! with a carousel.
//!
//! In:  ecx, passed on to the carousel
//! Out: every register preserved
//!
//! `scene::clear_depth_buffer`; `scene::assign_kind_textures`;
//! `scene::choose_carousel_pictures`; `scene::assign_menu_textures`;
//! `scene::map_texture_places`; `scene::load_slot_textures`;
//! `scene::draw_main_menu_carousel`. Each is entered with the registers the one
//! before left.
//! Reached through the menus' scene lists.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, rva};
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::choose_carousel_pictures::choose_carousel_pictures;
use crate::scene::clear_depth_buffer::clear_depth_buffer_in_game;
use crate::scene::draw_carousel::draw_carousel_in_game;
use crate::scene::draw_main_menu_carousel::draw_main_menu_carousel;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;

const STEPS: [usize; 7] = [
    rva(0x44E967),
    rva(0x4256D8),
    rva(0x4901B9),
    rva(0x490DF3),
    rva(0x42570D),
    rva(0x425778),
    rva(0x49012C),
];

// ---- Rust version ----

/// `step`: what stands for the seven calls, by their order.
pub unsafe fn draw_main_menu_objects(
    r: &Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let mut inner = *r;
    for n in 0..STEPS.len() {
        step(n, &mut inner)?;
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let done = unsafe {
        draw_main_menu_objects(r, &mut |n, r| {
            match n {
                0 => clear_depth_buffer_in_game(r)?,
                1 => assign_kind_textures()?.apply(r),
                2 => choose_carousel_pictures()?.apply(r),
                3 => assign_menu_textures()?,
                4 => map_texture_places().apply(r),
                5 => {
                    let entry = *r;
                    load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry))?
                        .apply(r)
                }
                _ => draw_main_menu_carousel(r, &mut |r| draw_carousel_in_game(r))?,
            }
            Ok(())
        })
    };
    done?;
    Ok(())
}
