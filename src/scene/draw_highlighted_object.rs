//! `scene::draw_highlighted_object`: the 3D part of a frame of the menus that show one object,
//! the highlighted item's.
//!
//! `scene::clear_depth_buffer`, `scene::assign_kind_textures`, `scene::assign_menu_textures`,
//! `scene::map_texture_places`, `scene::load_slot_textures`, then `draw_highlighted`: the
//! lights' draw mode = 0 and
//! - the turn about y from the clock (the scene clock times 8, its low sixteen bits), the
//!   place's x 0 and its z the carousel's z;
//! - the kind the highlighted item (numbered from 1) has in the carousel's kinds; a kind above
//!   0x258 takes its size from the twelve-byte records of the kinds' own places (`OWN`), and
//!   the y and the turn about x unless they are -1 there;
//! - the kind to the shown kind, its first object (`scene::find_object_of_kind`) to the
//!   current object, and `scene::draw_menu_object` with it.
//!
//! In twelve of the menus' scene lists (the options and the screens under them). Refuses a
//! highlighted item of 0 or past the 0x100 kinds, a kind past the records, and what its parts
//! refuse. The menus and draw lists run its `run` (`menu::table_routes`).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::clear_depth_buffer::clear_depth_buffer;
use crate::scene::draw_carousel_entry::{ENTRIES, OWN, find_object_of_kind, own_place};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- game data ----

/// The highlighted item's object set up and drawn by `draw`.
pub fn draw_highlighted(
    shared: &crate::race::owner_shared::RaceShared,
    menus: &Menus,
    cars: &crate::race::car_draw_state::CarDrawState,
    object_place: &crate::scene::object_place::ObjectPlace,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    let item = screens.loop_flags.item.get().wrapping_sub(1);
    if item >= ENTRIES {
        return Err(BadInput("a highlighted item of 0 or past the carousel's kinds").into());
    }
    let kind = screens.carousel.kinds[item as usize].get();
    let own = (kind as i32 > 0x258).then(|| kind.wrapping_sub(0x259));
    if own.is_some_and(|own| own >= OWN) {
        return Err(BadInput("a kind past the records of the kinds' own places").into());
    }
    let clock: u32 = shared.physics_tables.get().scene_clock.get();
    object_place.angles[1].set((clock << 3) & 0xFFFF);
    object_place.place[2].set(screens.carousel.z.get());
    object_place.place[0].set(0u32);
    if let Some(own) = own {
        let [size, y, angle] = own_place(&menus.lists, own)?;
        object_place.size.set(size);
        if y != u32::MAX {
            object_place.place[1].set(y);
        }
        if angle != u32::MAX {
            object_place.angles[0].set(angle);
        }
    }
    screens.carousel.kind.set(kind);
    let object = find_object_of_kind(kind, scene);
    cars.object_number.set(object);
    draw()
}

/// The six calls in order.
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    let (scene, race, menus, system) = (o.scene, o.race, o.menus, o.system);
    let scratch = crate::race::view_scratch::front_end();
    clear_depth_buffer(
        scene,
        system,
        crate::workers::game_scratch(),
        &scratch.polygon,
    );
    assign_kind_textures(scene)?;
    assign_menu_textures(&menus.screens.carousel, scene)?;
    map_texture_places(scene);
    load_slot_textures_in_game(&race.frame, scene)?;
    scene.lights.state.get().draw_mode.set(0u32);
    // The front end's 3D draws on a slot of its own.
    let ctx = crate::race::view_scratch::DrawCtx::new(
        race,
        scene,
        o.input,
        scratch,
        crate::workers::game_scratch(),
        crate::workers::DrawReads::on_game_thread(crate::workers::LentOwners::of(o)),
    );
    let _held = ctx.view.matrices.hold(ctx.reads.matrices, ctx.part);
    draw_highlighted(
        &race.shared,
        menus,
        &ctx.view.cars,
        &ctx.view.object,
        &mut || draw_menu_object_in_game(&ctx),
        scene,
    )
}
