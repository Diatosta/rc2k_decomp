//! `scene::draw_car_object`: the 3D part of a frame of the car specification screen.
//!
//! Only with the shown page at 0: `scene::clear_depth_buffer`, `scene::assign_kind_textures`,
//! `pick_pictures`, `scene::map_texture_places`, `scene::load_slot_textures`,
//! `draw_shown_car`. The last two of those have one caller each:
//! - `pick_pictures`: with the carousel menu at 0xB, the entry before the car shown and the
//!   three after it, wrapping at the carousel's count, go to the carousel's picture slots,
//!   then `scene::update_carousel_pictures`;
//! - `draw_shown_car`: the lights' draw mode = 0; the turn about y from the clock (times 8,
//!   sixteen bits), the place's z, the car's kind from the carousel's kinds to the shown
//!   kind, its object to the current object, `scene::draw_menu_object` with it.
//!
//! Refuses an entry past the 0x100 kinds, and what its parts refuse. The menus and draw lists
//! run its `run` (`menu::table_routes`).

use crate::errors::{BadInput, Refused};
use crate::menu::owner::{Menus, Screens};
use crate::race::owner_shared::RaceShared;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::carousel_pictures::update;
use crate::scene::clear_depth_buffer::clear_depth_buffer;
use crate::scene::draw_carousel_entry::{ENTRIES, find_object_of_kind};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;
use crate::session::write_all;

// ---- game data ----

/// The four carousel entries around the car shown.
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

/// Picks the carousel's four pictures around the car shown.
pub fn pick_pictures(menus: &Menus, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let screens = &menus.screens;
    if screens.state.carousel.menu.get() != 0xB {
        return Ok(());
    }
    let entries = picked(
        screens.event.shown_car.get(),
        screens.state.carousel.count.get(),
    );
    write_all(&screens.state.carousel_pictures.wanted, entries);
    update(menus, scene, entries)?;
    Ok(())
}

/// The car shown's object set up and drawn by `draw`.
pub fn draw_shown_car(
    shared: &RaceShared,
    cars: &crate::race::car_draw_state::CarDrawState,
    object_place: &crate::scene::object_place::ObjectPlace,
    screens: &Screens,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let car: u32 = screens.event.shown_car.get();
    if car >= ENTRIES {
        return Err(BadInput("a car shown past the carousel's kinds").into());
    }
    let clock: u32 = shared.physics_tables.get().scene_clock.get();
    object_place.angles[1].set((clock << 3) & 0xFFFF);
    object_place.place[2].set(screens.state.carousel.z.get());
    let kind = screens.state.carousel.kinds[car as usize].get();
    screens.state.carousel.kind.set(kind);
    let object = find_object_of_kind(kind, scene);
    cars.object_number.set(object);
    draw()
}

/// Nothing on a page other than the first, else its six calls in order.
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    let (scene, race, menus, system) = (o.scene, o.race, o.menus, o.system);
    if menus.screens.event.car_page.get() != 0 {
        return Ok(());
    }
    let scratch = crate::race::view_scratch::front_end();
    clear_depth_buffer(
        scene,
        system,
        crate::workers::game_scratch(),
        &scratch.polygon,
    );
    assign_kind_textures(scene)?;
    pick_pictures(menus, scene)?;
    map_texture_places(scene);
    load_slot_textures_in_game(&race.frame, scene)?;
    scene.lights.state.get().draw_mode.set(0u32);
    // The front end's 3D draws on a slot of its own.
    let ctx = crate::race::view_scratch::DrawCtx::new(
        race,
        scene,
        o.net,
        o.input,
        scratch,
        crate::workers::game_scratch(),
        crate::workers::DrawReads::on_game_thread(crate::workers::LentOwners {
            race,
            scene,
            system,
        }),
    );
    let _held = ctx.view.matrices.hold(ctx.reads.matrices, ctx.part);
    draw_shown_car(
        &race.shared,
        &ctx.view.cars,
        &ctx.view.object,
        &menus.screens,
        &mut || draw_menu_object_in_game(&ctx),
        scene,
    )
}
