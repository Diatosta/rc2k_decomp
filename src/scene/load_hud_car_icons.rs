//! `scene::load_hud_car_icons` (0x4255AA): the two car icons painted into the HUD's texture (kind 0xC1):
//! car 0's model and car 1's, from the icon sheets `tx1822` .. `tx1825` (the 34 cars and
//! the bonus vehicles, 82 x 42 each, with their masks). The game's log calls every texture a
//! "plate"; this was `scene::load_number_plates` until 2026-10-24.
//!
//! In:  nothing
//! Out: kind 0xC1 has no first texture: ebx = -1, the rest preserved; else eax, ecx =
//!      the second icon's two picture names, edx, esi its two numbers, edi = 0xAD,
//!      ebp = 0x94, ebx = the texture
//!
//! Unless kind 0xC1's first texture (`dword_5D0DEC`) is none, twice, for the player's
//! car (`dword_70F410`, or with `dword_52A910` set the block of the car
//! `dword_52A920`, its +0x24) and then for `dword_7103F0`:
//! - `scene::car_icon_number` of it gives the icon's number;
//! - under 0x12 the pictures `files\b\tx1822.pcx` and `tx1823.pcx`, else `tx1824.pcx`
//!   and `tx1825.pcx` (eax, ecx);
//! - edx, esi = `dword_5DCD9C[number]`, `dword_5DCE2C[number]`, edi = 0xAD, ebp = 0x68
//!   for the first and 0x94 for the second, ebx = the texture;
//! - `scene::upload_car_icon` (`pusha` .. `popa`).
//!
//! One caller, `scene::reload_system_textures`. Refuses a number of 0x24 or more (the tables'
//! room) and a car without its block.

use crate::errors::Refused;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::work_store::RACE_WORK;
use crate::scene::car_icon_number::car_icon_number;
use crate::scene::file_numbers_store::texts;
use crate::scene::upload_car_icon::{CarIcon, upload_car_icon};

// ---- game data ----
// scene::upload_car_icon
/// `files\b\tx1822.pcx` .. `tx1825.pcx`.
const NAMES: [usize; 4] = [
    texts::CAR_ICON_0,
    texts::CAR_ICON_1,
    texts::CAR_ICON_2,
    texts::CAR_ICON_3,
];

// ---- Rust version ----

/// The Rust version. The registers the original leaves (the last icon's, or ebx -1 when
/// kind 0xC1 has no texture) are read by no caller (taint at its one site, in
/// `reload_system_textures`: 0 read points).
// Converted: `rc2k_core::scene::car_icons`
pub fn load_hud_car_icons(scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    use rc2k_core::scene::car_icons;
    let texture: u32 = scene.textures.kinds.kind_first_textures[193].bits();
    if texture == u32::MAX {
        return Ok(());
    }
    let player = if crate::net::store::NET.session.get() != 0 {
        let car = car_icons::one_car(crate::net::store::NET.player.get())?;
        let block: u32 = RACE_WORK.car_block(car);
        crate::race::blocks_store::block_at(block, "a car without its block")?
            .car_kind
            .bits()
    } else {
        CAR_BLOCKS.cars[0].car_kind.bits()
    };
    let numbers = [
        car_icon_number(player),
        car_icon_number(CAR_BLOCKS.cars[1].car_kind.bits()),
    ];
    car_icons::check_numbers(numbers)?;
    for (number, target_y) in numbers.into_iter().zip(car_icons::ROWS) {
        let names = car_icons::pictures(number);
        upload_car_icon(
            &scene.icon_upload,
            &CarIcon {
                sheet: scene.file_numbers.text_ptr(NAMES[names]),
                alpha_sheet: scene.file_numbers.text_ptr(NAMES[names + 1]),
                texture,
                sheet_x: scene.textures.places.car_icon_sheet_x[number as usize].get(),
                sheet_y: scene.textures.places.car_icon_sheet_y[number as usize].get(),
                target_x: car_icons::COLUMN,
                target_y,
            },
        )?;
    }
    Ok(())
}
