//! `scene::reload_system_textures` (0x4254B4): every system texture gets the picture
//! it should have ("reload system plates").
//!
//! In:  nothing
//! Out: the registers as `scene::load_hud_car_icons` leaves them
//!
//! - Two lines in the game's log (a rule and "Reload system plates - start").
//! - Each kind's first texture (`g_kindFirstTextures`), where it has one, wants the
//!   kind's picture (`dword_5D7968`; 0 counts as none, -1).
//! - For each of the `dword_5DC974` system textures: `dword_4F4A24` = 1 and the size
//!   pictures are read at (`dword_4F4A2C`, `dword_4F4A28`) = the scene's
//!   (`dword_609300`, `dword_609304`), or 256 x 256 for picture 0x3DE and pictures
//!   0x706 .. 0x7CF; `scene::load_changed_texture_picture`;
//!   `scene::step_loading_bar`.
//! - `dword_4F4A24` = 0 and the size 256 x 256 again; `scene::load_hud_car_icons`;
//!   two lines in the log.
//!
//! Two callers.
//! Refuses no system textures (the original goes through 2^32) or more than 0x400,
//! and a first texture past the table.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note what was set by then.
//!
//! Converted 2026-10-21: the kinds' pictures and the read sizes through
//! `rc2k_core::scene::pictures`.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::gfx::block::{game_ptr, game_read, game_write};
use crate::gfx::picture_store::Slot as PicSlot;
use crate::logfile::log_write::log_write;
use crate::scene::load_changed_texture_picture::{
    TEXTURES, WANTED_PICTURES_RVA, load_changed_texture_picture_in_game,
};
use crate::scene::load_hud_car_icons::load_hud_car_icons;
use crate::scene::step_loading_bar::step_loading_bar;
use crate::scene::textures::{KIND_TEXTURE_RVA, KINDS};
use rc2k_core::scene::pictures::{kind_picture, read_size};

// ---- game data ----

const KIND_PICTURES_RVA: usize = rva(0x5D7968); // dword_5D7968
const SYSTEM_TEXTURES_RVA: usize = rva(0x5DC974); // dword_5DC974
const READING_RVA: PicSlot = PicSlot::at(0x4F4A24); // dword_4F4A24
/// `dword_4F4A28`, `dword_4F4A2C`: the size pictures are read at.
const READ_SIZE_RVA: PicSlot = PicSlot::at(0x4F4A28);
/// `dword_609300`, `dword_609304`: the scene's size for them.
const SCENE_SIZE_RVA: usize = rva(0x609300);
// scene::load_hud_car_icons
const TEXTS: [usize; 4] = [rva(0x4DB7E2), rva(0x4DB800), rva(0x4DB81D), rva(0x4DB83A)];

// ---- Rust version ----

/// The Rust version. The registers the original leaves (those of `load_hud_car_icons`) are
/// read by no caller (taint at the site in `race::set_all_plates`: 0 read points; the other
/// is the game's own code, `sub_443478`, not Rust). It reads no register at entry but edi,
/// esi (for `load_changed_texture_picture` and `step_loading_bar`, which set them
/// themselves).
// Converted: `rc2k_core::scene::pictures::{kind_picture, read_size}`
pub unsafe fn reload_system_textures() -> Result<(), Refused> {
    unsafe {
        let textures: u32 = game_read(SYSTEM_TEXTURES_RVA);
        if textures == 0 || textures > TEXTURES {
            return Err(BadInput("no system textures, or more than the table has").into());
        }
        let first = |kind: u32| game_read::<u32>(KIND_TEXTURE_RVA + kind as usize * 4);
        if (0..KINDS).any(|kind| first(kind) != u32::MAX && first(kind) >= TEXTURES) {
            return Err(BadInput("a kind's first texture past the pictures' table").into());
        }
        log_write(game_ptr(TEXTS[0]), true, 1);
        log_write(game_ptr(TEXTS[1]), true, 1);
        for kind in 0..KINDS {
            let texture = first(kind);
            if texture == u32::MAX {
                continue;
            }
            let picture = kind_picture(game_read(KIND_PICTURES_RVA + kind as usize * 4));
            game_write(WANTED_PICTURES_RVA + texture as usize * 4, picture);
        }
        let scene: [u32; 2] = game_read(SCENE_SIZE_RVA);
        for texture in 0..textures {
            game_write(READING_RVA, 1u32);
            let wanted = game_read(WANTED_PICTURES_RVA + texture as usize * 4);
            game_write(READ_SIZE_RVA, read_size(wanted, scene));
            load_changed_texture_picture_in_game(texture)?;
            step_loading_bar(texture)?;
        }
        game_write(READING_RVA, 0u32);
        game_write(READ_SIZE_RVA, [0x100u32, 0x100]);
        load_hud_car_icons()?;
        log_write(game_ptr(TEXTS[2]), true, 1);
        log_write(game_ptr(TEXTS[3]), true, 1);
        Ok(())
    }
}
