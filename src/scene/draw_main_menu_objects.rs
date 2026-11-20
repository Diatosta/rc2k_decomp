//! `scene::draw_main_menu_objects`: the 3D part of a frame of the menus with a carousel.
//!
//! `scene::clear_depth_buffer`; `scene::assign_kind_textures`;
//! `scene::choose_carousel_pictures`; `scene::assign_menu_textures`;
//! `scene::map_texture_places`; `scene::load_slot_textures`; `scene::draw_main_menu_carousel`.
//! Reached through the menus' scene lists; the menus and draw lists run its `run`
//! (`menu::table_routes`).

use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::choose_carousel_pictures::choose_carousel_pictures;
use crate::scene::clear_depth_buffer::clear_depth_buffer;
use crate::scene::draw_main_menu_carousel::draw_main_menu_carousel;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;

/// Its seven calls in order. The slots-in-use count the textures' loading gives back is the
/// row place of a lone entry.
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    let (scene, race, menus, system) = (o.scene, o.race, o.menus, o.system);
    let scratch = o.scene.scratch.front_end();
    clear_depth_buffer(
        scene,
        system,
        o.scene.scratch.game_scratch(),
        &scratch.polygon,
    );
    assign_kind_textures(scene)?;
    choose_carousel_pictures(menus, scene)?;
    assign_menu_textures(&menus.screens.carousel, scene)?;
    map_texture_places(scene);
    let count = load_slot_textures(&race.frame, scene)?;
    // The front end's 3D draws on a slot of its own.
    let ctx = crate::race::view_scratch::DrawCtx::new(
        race,
        scene,
        o.input,
        scratch,
        o.scene.scratch.game_scratch(),
        crate::scene::workers::DrawReads::on_game_thread(crate::scene::workers::LentOwners::of(o)),
    );
    let _held = ctx.view.matrices.hold(ctx.reads.matrices, ctx.part);
    draw_main_menu_carousel(menus, &ctx, count)
}
