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
//! standalone program's run of 2026-10-14).
//! Refuses a highlighted item of 0 or past the 0x100 kinds, a kind past the records, and what
//! its parts refuse.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::assign_menu_textures::assign_menu_textures;
use crate::scene::clear_depth_buffer::clear_depth_buffer_in_game;
use crate::scene::draw_carousel_entry::{
    ANGLE_X_RVA, ANGLE_Y_RVA, CLOCK_RVA, ENTRIES, KIND_RVA, KINDS_RVA, OBJECT_RVA, OWN, OWN_RVA,
    PLACE_RVA, SIZE_RVA, find_object_of_kind,
};
use crate::scene::draw_menu_object::draw_menu_object_in_game;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;

// ---- game data ----

const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4
const HIGHLIGHTED_RVA: usize = rva(0x755525); // dword_755525
const Z_RVA: usize = rva(0x75A66C); // dword_75A66C
const STEPS: [usize; 6] = [
    rva(0x44E967),
    rva(0x4256D8),
    rva(0x490DF3),
    rva(0x42570D),
    rva(0x425778),
    rva(0x49006F),
];

// ---- Rust version ----

/// `sub_49007F`: the highlighted item's object set up and drawn by `draw`.
pub unsafe fn draw_highlighted(
    r: &Regs,
    draw: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let item = game_read::<u32>(HIGHLIGHTED_RVA).wrapping_sub(1);
        if item >= ENTRIES {
            return Err(BadInput("a highlighted item of 0 or past the carousel's kinds").into());
        }
        let kind: u32 = game_read(KINDS_RVA + item as usize * 4);
        let own = (kind as i32 > 0x258).then(|| kind.wrapping_sub(0x259));
        if own.is_some_and(|own| own >= OWN) {
            return Err(BadInput("a kind past the records of the kinds' own places").into());
        }
        let clock: u32 = game_read(CLOCK_RVA);
        game_write(ANGLE_Y_RVA, (clock << 3) & 0xFFFF);
        game_write(PLACE_RVA + 8, game_read::<u32>(Z_RVA));
        game_write(PLACE_RVA, 0u32);
        if let Some(own) = own {
            let [size, y, angle]: [u32; 3] = game_read(OWN_RVA + own as usize * 12);
            game_write(SIZE_RVA, size);
            if y != u32::MAX {
                game_write(PLACE_RVA + 4, y);
            }
            if angle != u32::MAX {
                game_write(ANGLE_X_RVA, angle);
            }
        }
        game_write(KIND_RVA, kind);
        let object = find_object_of_kind(kind);
        game_write(OBJECT_RVA, object);
        let mut inner = Regs {
            eax: object,
            ebx: item,
            ..*r
        };
        draw(&mut inner)
    }
}

/// `step`: what stands for the six calls, by their order.
pub unsafe fn draw_highlighted_object(
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
        draw_highlighted_object(r, &mut |n, r| {
            match n {
                0 => clear_depth_buffer_in_game(r)?,
                1 => assign_kind_textures()?.apply(r),
                2 => assign_menu_textures()?,
                3 => map_texture_places().apply(r),
                4 => {
                    let entry = *r;
                    load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry))?
                        .apply(r)
                }
                _ => {
                    game_write(MODE_RVA, 0u32);
                    draw_highlighted(r, &mut |r| draw_menu_object_in_game(r))?;
                }
            }
            Ok(())
        })
    };
    done?;
    Ok(())
}
