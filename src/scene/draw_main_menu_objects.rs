//! `scene::draw_main_menu_objects` (0x490106): the 3D part of a frame of the menus
//! with a carousel.
//!
//! `scene::clear_depth_buffer`; `scene::assign_kind_textures`;
//! `scene::choose_carousel_pictures`; `scene::assign_menu_textures`;
//! `scene::map_texture_places`; `scene::load_slot_textures`; `scene::draw_main_menu_carousel`.
//! Each is entered with the registers the one before left. Reached through the menus' scene
//! lists. The menus and draw lists run its `run` (`menu::table_routes`); the registers above
//! are the original's, for reference.

use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::choose_carousel_pictures::choose_carousel_pictures;
use crate::scene::clear_depth_buffer::clear_depth_buffer;
use crate::scene::draw_main_menu_carousel::draw_main_menu_carousel;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- Rust version ----

/// Its seven calls in order. Of what the calls leave in the original's registers one value is
/// read: the slots-in-use count the textures' loading leaves (the row place of a lone entry).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    let (scene, race, menus, system) = (o.scene, o.race, o.menus, o.system);
    let scratch = crate::race::view_scratch::front_end();
    clear_depth_buffer(scene, system, &scratch.polygon);
    assign_kind_textures(scene)?;
    choose_carousel_pictures(menus, scene)?;
    assign_menu_textures(&menus.tables, scene)?;
    map_texture_places(scene);
    let count = load_slot_textures_in_game(&race.frame, scene)?;
    // The front end's 3D draws on a slot of its own (frame plan F4).
    let ctx = crate::race::view_scratch::DrawCtx::new(
        race,
        scene,
        system,
        scratch,
        crate::workers::game_scratch(),
    );
    let _held = ctx.view.matrices.hold(&race.frame.matrices);
    draw_main_menu_carousel(menus, &ctx, count)
}
