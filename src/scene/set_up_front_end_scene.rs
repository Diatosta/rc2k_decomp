//! `scene::set_up_front_end_scene`: the front end's 3D set-up: which textures and objects the
//! menus need, and everything loaded.
//!
//! One list of stores and 50 calls, in this order:
//! - the loading bar's range 0 .. 100; the loading picture; `scene::reset_texture_tables`;
//! - the textures needed (`NEEDS`): level 0 and a tag for the menus' own 42 system and 8 video
//!   textures from 0x258, the carousel pictures' 0x110 and 0x20 from 0x2BC, and texture 0x87;
//! - `scene::set_up_textures`, `scene::set_up_object_memory`, `scene::clear_slot_kinds`;
//! - the scene's colours and sizes, the front end's own fog (`scene::fog`), the `scene_set_up`
//!   switch = 1, `scene::fill_light_tables`;
//! - the slots' kinds: slot 0 kind 1, slots 1 ..= 0x28 kinds 0x259 .. 0x280, slots 0x29 ..= 0x2C
//!   kinds 0x28B .. 0x28E, slots 0x2D ..= 0x4E the 0x22 known kinds (`scene::known_kind`);
//! - `scene::load_front_end_objects` and nine lines in the game's log with the points and faces
//!   loaded (`scene::load_front_end_objects_logged`); `scene::reset_texture_places`;
//! - the kinds' pictures (the kind pictures table): kind 0x87 picture 0x82A, kinds 0x258 ..
//!   0x271 the 26 of `MENU_PICTURES`, and eight in a row from kind 0x2BC + 8 * place for each
//!   of 33 known kinds (`scene::known_kind_picture` of places 0 ..= 0x1D, 0x1F, 0x20, 0x21, and
//!   the seven numbers after it; place 0x1E's eight are left alone);
//! - `scene::reload_system_textures` with the loading bar,
//!   `scene::allocate_point_records`;
//! - the colours again, `scene::fill_scene_light_tables`, `scene::set_view_matrices`,
//!   `scene::reload_slot_textures`;
//! - the colours a third time with the stage's lamp level = 0x7F, `scene_set_up` = 1,
//!   `scene::fill_light_tables`, the reflections setting = 1.
//!
//! Thirteen callers (the main menu's loop, the films, the ends of the events). A refusal by one
//! of its parts comes with the work half done.

use crate::errors::Refused;
use crate::memory::session::PlainDword;
use crate::scene::allocate_point_records::allocate_point_records_in_game;
use crate::scene::clear_slot_kinds::clear_slot_kinds;
use crate::scene::fill_light_tables::fill_light_tables;
use crate::scene::fill_scene_light_tables::fill_scene_light_tables;
use crate::scene::known_kind::known_kind;
use crate::scene::known_kind_picture::known_kind_picture;
use crate::scene::owner::Scene;
use crate::scene::reload_slot_textures::reload_slot_textures;
use crate::scene::reset_texture_places::reset_texture_places;
use crate::scene::reset_texture_tables::reset_texture_tables;
use crate::scene::set_up_object_memory::set_up_object_memory;
use crate::scene::view_matrix::set_view_matrices;

// ---- game data ----
fn system_levels(scene: &Scene) -> &[PlainDword; 1024] {
    &scene.objects.tables.get().system_levels
}
fn video_levels(scene: &Scene) -> &[PlainDword; 1024] {
    &scene.objects.tables.get().video_levels
}
type Levels = for<'a> fn(&'a Scene) -> &'a [PlainDword; 1024];
/// The textures needed: how many, the first, the levels' table, the tag.
const NEEDS: [(usize, usize, Levels, u32); 6] = [
    (0x2A, 0x258, system_levels, 0x63),
    (8, 0x258, video_levels, 0x63),
    (0x110, 0x2BC, system_levels, 0x51),
    (0x20, 0x2BC, video_levels, 0x51),
    (1, 0x87, system_levels, 0x63),
    (1, 0x87, video_levels, 0x63),
];

/// The pictures of kinds 0x258 on.
const MENU_PICTURES: [u32; 26] = [
    0x7D1, 0x7D2, 0x7D3, 0x7D5, 0x7D7, 0x7D8, 0x7D9, 0x7DA, 0x7DB, 0x7DC, 0x7DE, 0x7DF, 0x7E1,
    0x7E2, 0x7E3, 0x7E6, 0x7E8, 0x7EE, 0x7EF, 0x7F0, 0x7F3, 0x7F4, 0x7F5, 0x7F6, 0x7F7, 0x7F8,
];

/// The known kinds whose pictures the carousels show: every place but 0x1E.
fn picture_places() -> impl Iterator<Item = u32> {
    (0..0x22).filter(|&place| place != 0x1E)
}

fn colours(course: &crate::race::owner_course::Course, dark_third: u32) {
    {
        crate::memory::session::write_bits(&course.stage_prefs.get().lit_colour, [0xFFu32; 3]);
        crate::memory::session::write_bits(
            &course.stage_prefs.get().dark_colour,
            [0u32, 0, dark_third],
        );
    }
}

/// The front end's scene set up.
pub fn set_up_front_end_scene_in(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (race, scene) = (o.race, o.scene);
    let shared = &race.shared;
    let frame = &race.frame;
    let course = &race.course;
    crate::menu::show_loading_picture::show_loading_picture_in(o)?;
    reset_texture_tables(scene);
    for (count, first, levels, tag) in NEEDS {
        for texture in first..first + count {
            levels(scene)[texture].set(0);
            scene.objects.tables.get().texture_tags[texture].set(tag);
        }
    }
    crate::scene::set_up_textures::set_up_textures(shared, scene, o.gfx, &o.gfx.d3d)?;
    set_up_object_memory(race, scene)?;
    clear_slot_kinds(scene);
    colours(course, 0);
    // The scene's settings stored before the fog is cleared, after the two colours.
    course.stage_prefs.get().scene_size_a.set(0x100u32);
    course.stage_prefs.get().scene_size_b.set(0x100u32);
    course.stage_prefs.get().fog_low.set(0u32);
    course.stage_prefs.get().fog_high.set(0u32);
    course.stage_prefs.get().fog_colour[0].set(0u32);
    course.stage_prefs.get().fog_colour[1].set(0u32);
    course.stage_prefs.get().fog_colour[2].set(0u32);
    course.stage_prefs.get().fog_near.set(0u32);
    course.stage_prefs.get().fog_start.set(0u32);
    course.stage_prefs.get().fog_end.set(0u32);
    o.scene.scratch.front_end().fog.clear();
    shared.switches.get().scene_set_up.set(1u32);
    fill_light_tables(scene, race)?;

    scene.objects.tables.get().slot_kinds[0].set(1);
    for slot in 1..0x29 {
        scene.objects.tables.get().slot_kinds[slot].set(0x258 + slot as u32);
    }
    for slot in 0x29..0x2D {
        scene.objects.tables.get().slot_kinds[slot].set(0x262 + slot as u32);
    }
    for place in 0..0x22usize {
        scene.objects.tables.get().slot_kinds[0x2D + place]
            .set(known_kind(race.kind_tables(), place as u32));
    }
    crate::scene::load_front_end_objects_logged::load_front_end_objects_logged(race, scene)?;
    reset_texture_places(scene);

    scene.textures.places.get().kind_pictures[0x87].set(0x82A);
    for (k, &picture) in MENU_PICTURES.iter().enumerate() {
        scene.textures.places.get().kind_pictures[0x258 + k].set(picture);
    }
    for place in picture_places() {
        let mut picture = known_kind_picture(race.kind_tables(), place);
        for k in 0..8 {
            scene.textures.places.get().kind_pictures[0x2BC + place as usize * 8 + k].set(picture);
            picture = picture.wrapping_add(1);
        }
    }
    crate::scene::reload_system_textures::reload_system_textures(
        o,
        &mut crate::race::loading_bar_frame::LoadingBar::new(o),
    )?;
    allocate_point_records_in_game(&scene.scratch, course)?;
    colours(course, 0);
    fill_scene_light_tables(race, scene)?;
    set_view_matrices(
        race.course.reads(),
        frame,
        &o.scene.scratch.front_end().matrices,
    );
    reload_slot_textures(frame, scene)?;
    colours(course, 0x7F);
    shared.switches.get().scene_set_up.set(1u32);
    fill_light_tables(scene, race)?;
    shared.vars.get().settings_words[1].set(1u32);
    Ok(())
}
