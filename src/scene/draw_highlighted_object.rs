//! `scene::draw_highlighted_object` (0x49004E): the 3D part of a frame of the menus
//! that show one object, the highlighted item's.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `scene::clear_depth_buffer`, `scene::assign_kind_textures`,
//! `scene::assign_menu_textures`, `scene::map_texture_places`,
//! `scene::load_slot_textures`, then `sub_49006F` (one caller): `dword_6365C4` = 0 and
//! `sub_49007F` (one caller), between `pusha` and `popa`:
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
use crate::framework::{game_read, game_write, rva};
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::state_store::MENU_STATE;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::clear_depth_buffer::clear_depth_buffer;
use crate::scene::draw_carousel_entry::{
    ANGLE_X_RVA, ANGLE_Y_RVA, CLOCK_RVA, ENTRIES, OBJECT_RVA, OWN, PLACE_RVA, SIZE_RVA,
    find_object_of_kind, own_place,
};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_textures::load_slot_textures_in_game;
use crate::scene::map_texture_places::map_texture_places;

// ---- game data ----

const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4

// ---- Rust version ----

/// `sub_49007F`: the highlighted item's object set up and drawn by `draw`.
pub unsafe fn draw_highlighted(
    draw: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let item = LOOP_FLAGS.item.get().wrapping_sub(1);
        if item >= ENTRIES {
            return Err(BadInput("a highlighted item of 0 or past the carousel's kinds").into());
        }
        let kind = MENU_STATE.carousel.kinds[item as usize].get();
        let own = (kind as i32 > 0x258).then(|| kind.wrapping_sub(0x259));
        if own.is_some_and(|own| own >= OWN) {
            return Err(BadInput("a kind past the records of the kinds' own places").into());
        }
        let clock: u32 = game_read(CLOCK_RVA);
        game_write(ANGLE_Y_RVA, (clock << 3) & 0xFFFF);
        game_write(PLACE_RVA + 8, MENU_STATE.carousel.z.get());
        game_write(PLACE_RVA, 0u32);
        if let Some(own) = own {
            let [size, y, angle] = own_place(own);
            game_write(SIZE_RVA, size);
            if y != u32::MAX {
                game_write(PLACE_RVA + 4, y);
            }
            if angle != u32::MAX {
                game_write(ANGLE_X_RVA, angle);
            }
        }
        MENU_STATE.carousel.kind.set(kind);
        let object = find_object_of_kind(kind);
        game_write(OBJECT_RVA, object);
        draw()
    }
}

/// The Rust version the typed tables run: its six calls in order. It reads no register of
/// its caller's (register liveness, 2026-10-23), and the drawing reads none of what the
/// calls before it leave (the eax and ebx `sub_49007F` gives it included).
pub fn run() -> Result<(), crate::errors::Refused> {
    unsafe {
        clear_depth_buffer();
        assign_kind_textures()?;
        assign_menu_textures()?;
        map_texture_places();
        load_slot_textures_in_game()?;
        game_write(MODE_RVA, 0u32);
        draw_highlighted(&mut || draw_menu_object_in_game())
    }
}
