//! `scene::draw_highlighted_object` (0x49004E): the 3D part of a frame of the menus
//! that show one object, the highlighted item's.
//!
//! `scene::clear_depth_buffer`, `scene::assign_kind_textures`,
//! `scene::assign_menu_textures`, `scene::map_texture_places`,
//! `scene::load_slot_textures`, then 0x49006F (one caller): `dword_6365C4` = 0 and
//! 0x49007F (one caller), between `pusha` and `popa`:
//! - the turn about y from the clock (`g_sceneClock` times 8, its low sixteen bits)
//!   to `dword_635040`; the place's x (`dword_63501C`) 0 and its z (`dword_635024`)
//!   `dword_75A66C`;
//! - the kind the highlighted item (`dword_755525`, from 1) has in the carousel's
//!   kinds (`dword_75A674`); a kind above 0x258 takes its size (`dword_635038`) from
//!   the twelve-byte records at `unk_75950E`, and the y and the turn about x
//!   (`dword_635020`, `dword_63503C`) unless they are -1 there;
//! - the kind to `dword_75A658`, its first object (`scene::find_object_of_kind`) to
//!   `g_objectNumber`, and `scene::draw_menu_object` with eax = it.
//!
//! In twelve of the menus' scene lists (the options and the screens under them, by the
//! standalone program's run of 2026-10-14). Refuses a highlighted item of 0 or past the 0x100
//! kinds, a kind past the records, and what its parts refuse. The menus and draw lists run its
//! `run` (`menu::table_routes`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::{Menus, menus};
use crate::race::owner::race;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::clear_depth_buffer::clear_depth_buffer;
use crate::scene::draw_carousel_entry::{ENTRIES, OWN, find_object_of_kind, own_place};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- game data ----

// ---- Rust version ----

/// 0x49007F: the highlighted item's object set up and drawn by `draw`.
pub fn draw_highlighted(
    menus: &Menus,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let scene = crate::scene::owner::scene();
    let shared = &race().shared;
    let drawing = &scene.drawing;
    let screens = &menus.screens;
    let item = screens.loop_flags.item.get().wrapping_sub(1);
    if item >= ENTRIES {
        return Err(BadInput("a highlighted item of 0 or past the carousel's kinds").into());
    }
    let kind = screens.state.carousel.kinds[item as usize].get();
    let own = (kind as i32 > 0x258).then(|| kind.wrapping_sub(0x259));
    if own.is_some_and(|own| own >= OWN) {
        return Err(BadInput("a kind past the records of the kinds' own places").into());
    }
    let clock: u32 = shared.physics_tables.scene_clock.bits();
    drawing.object_work.angles[1].set((clock << 3) & 0xFFFF);
    drawing.object_work.place[2].set(screens.state.carousel.z.get());
    drawing.object_work.place[0].set_bits(0u32);
    if let Some(own) = own {
        let [size, y, angle] = own_place(&menus.tables, own)?;
        drawing.object_work.size.set_bits(size);
        if y != u32::MAX {
            drawing.object_work.place[1].set(y);
        }
        if angle != u32::MAX {
            drawing.object_work.angles[0].set_bits(angle);
        }
    }
    screens.state.carousel.kind.set(kind);
    let object = find_object_of_kind(kind, scene);
    shared.switches.object_number.set_bits(object);
    draw()
}

/// The Rust version the typed tables run: its six calls in order. It reads no register of
/// its caller's (register liveness, 2026-10-23), and the drawing reads none of what the
/// calls before it leave (the eax and ebx 0x49007F gives it included).
pub fn run() -> Result<(), crate::errors::Refused> {
    let scene = crate::scene::owner::scene();
    let race = race();
    let menus = menus();
    clear_depth_buffer(scene);
    assign_kind_textures(scene)?;
    assign_menu_textures(&menus.tables, scene)?;
    map_texture_places(scene);
    load_slot_textures_in_game(&race.frame, scene)?;
    scene.lights.state.draw_mode.set_bits(0u32);
    draw_highlighted(menus, &mut || draw_menu_object_in_game(race, scene))
}
