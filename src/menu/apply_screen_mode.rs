//! `menu::apply_screen_mode` (0x4703C2): the graphics page's "apply": the screen mode chosen
//! set (`sub_4045D6` with eax 1, the platform's: DirectDraw modes). Taken: the fonts freed
//! (`gfx::free_fonts`, `gfx::free_glyph_tables`) and built again (`gfx::build_fonts`,
//! `gfx::build_menu_sprites`), the 3D tidied (`render3d_tidy`, `objects_tidy`,
//! `nullsub_6`, `release_textures`), `scene::set_up_front_end_scene`, tune 4, the
//! highlighted item (`dword_755525`) and `dword_74771C` 1, the menu's sound 2. Refused (a
//! set carry): `sub_4045D6` with eax 0 (the mode before, a reading), the 3D tidied, the
//! front end's scene, tune 4, the item and `dword_74771C` 3, the menu's sound 8.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! An item action of the graphics page (0x7548D4). Its calls through `part`: the screen mode
//! at the game's address (section 15), the glyph tables, the scene and the sound in Rust,
//! the rest at the game's addresses (their hooks choose).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_write, rva};
use crate::menu::parts::{Part, parted_entry};

const CALLEES: [usize; 12] = [
    0x4045D6, // 0 the screen mode (sub_4045D6)
    0x415C55, // 1 gfx::free_fonts
    0x474D12, // 2 gfx::free_glyph_tables
    0x415563, // 3 gfx::build_fonts
    0x47C4A1, // 4 gfx::build_menu_sprites
    0x4433A7, // 5 race::render3d_tidy
    0x44346B, // 6 race::objects_tidy
    0x44347E, // 7 nullsub_6
    0x443396, // 8 race::release_textures
    0x474E04, // 9 scene::set_up_front_end_scene
    0x40F792, // 10 sound::play_tune_4
    0x40F059, // 11 sound::play_menu_sound
];
const ITEM_RVA: usize = rva(0x755525); // dword_755525
const AFTER_RVA: usize = rva(0x74771C); // dword_74771C

/// The Rust version: the registers it leaves.
pub unsafe fn apply_screen_mode(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::steps::applied_mode`.
    use rc2k_core::menu::steps::{BACK_TO_MODE, TRY_CHOSEN_MODE, applied_mode};
    let mut r = *entry;
    r.eax = TRY_CHOSEN_MODE;
    part(0, &mut r)?;
    let taken = !r.carry();
    if taken {
        for place in [1, 2, 3, 4] {
            part(place, &mut r)?;
        }
    } else {
        r.eax = BACK_TO_MODE;
        part(0, &mut r)?;
    }
    for place in [5, 6, 7, 8, 9, 10] {
        part(place, &mut r)?;
    }
    let applied = applied_mode(taken);
    unsafe {
        game_write(ITEM_RVA, applied.item);
        game_write(AFTER_RVA, applied.item);
    }
    r.eax = applied.sound;
    part(11, &mut r)?;
    Ok(*entry)
}

pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            2 => crate::gfx::free_glyph_tables::entry(r)?,
            9 => crate::scene::set_up_front_end_scene::set_up_front_end_scene_in_game(r)?,
            11 => crate::sound::play_menu_sound::play_menu_sound_in_game(r),
            _ => crate::calls::call(rva(CALLEES[place as usize]), r),
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    parted_entry(
        r,
        &|r, part| unsafe { apply_screen_mode(r, part) },
        game_part,
    )
}
