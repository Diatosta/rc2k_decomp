//! `race::set_all_plates` (0x42518B): the texture kinds' pictures for the stage
//! (`LOADLEVEL_INIT : setplatesall`), then the system and slot textures reloaded.
//!
//! In:  nothing
//! Out: nothing its caller reads (conversion phase 3, 2026-10-24)
//!
//! - The stage prefs' (`localdat`) picture lists copied to `g_kindPictures` (0x5D7968), 0x32C14
//!   bytes below: 0x60A57C .. 0x60A64C, 0x60A67C .. 0x60A73C, 0x60A77C .. 0x60A79C (the
//!   original in 4-dword `rep movsd` pieces; esi, edi and ecx left as its last).
//! - 23 kinds' pictures fixed (`FIXED_PICTURES`).
//! - With cars, each car's eight from 0x5D7D68: its kind's picture
//!   (`scene::known_kind_picture` of `dword_623858`[car]) and the seven after it.
//! - `scene::reload_system_textures`, `scene::reload_slot_textures`.
//!
//! Callers: `sub_44426F` (loadlevel_init) and the code at 0x425B29. Refuses more than 8 cars.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::race::view_store::RACE_VIEW;

const PICTURES_RVA: usize = rva(0x5D7968); // g_kindPictures
/// The prefs' picture lists (`PREF_PIECES`' kinds) start here, 0x32C14 bytes above the
/// pictures.
const PREF_LISTS: usize = 0x60A57C;
const CAR_KINDS_RVA: usize = rva(0x623858); // g_raceCarKinds
/// `race::set_all_plates` (see the header).
pub unsafe fn set_all_plates() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::stage_objects::{check_picture_cars, PREF_PIECES,
    // FIXED_PICTURES, car_pictures}`.
    use rc2k_core::race::stage_objects::{
        CAR_PICTURE_KINDS, FIXED_PICTURES, PREF_PIECES, car_pictures, check_picture_cars,
    };
    unsafe {
        let cars: u32 = RACE_VIEW.race_cars.bits();
        check_picture_cars(cars)?;
        let pictures = game_addr(PICTURES_RVA);
        for (kind, dwords) in PREF_PIECES {
            let from = game_addr(rva(PREF_LISTS)) + 4 * kind;
            let to = pictures + 4 * kind;
            std::ptr::copy(from as *const u32, to as *mut u32, dwords);
        }
        for (kind, picture) in FIXED_PICTURES {
            game_write(PICTURES_RVA + 4 * kind, picture);
        }
        let mut at = (pictures + 4 * CAR_PICTURE_KINDS) as *mut u32;
        for car in 0..cars {
            let first = crate::scene::known_kind_picture::known_kind_picture(game_read(
                CAR_KINDS_RVA + 4 * car as usize,
            ));
            for picture in car_pictures(first) {
                at.write_unaligned(picture);
                at = at.add(1);
            }
        }
        crate::scene::reload_system_textures::reload_system_textures()?;
        crate::scene::reload_slot_textures::reload_slot_textures()?;
    }
    Ok(())
}
