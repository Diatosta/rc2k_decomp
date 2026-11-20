//! `scene::load_hud_car_icons`: the two car icons painted into the HUD's texture (kind 0xC1):
//! car 0's model and car 1's, from the icon sheets `tx1822` .. `tx1825` (the 34 cars and the
//! bonus vehicles, 82 x 42 each, with their masks). The game's log calls every texture a
//! "plate".
//!
//! Unless kind 0xC1's first texture is none, twice, for the player's car (the own car, or in a
//! network session the block of the network player's car, its `car_kind`) and then for the second
//! player's car:
//! - `scene::car_icon_number` of it gives the icon's number;
//! - under 0x12 the pictures `files\b\tx1822.pcx` and `tx1823.pcx`, else `tx1824.pcx` and
//!   `tx1825.pcx`;
//! - the icon's place on the sheet from the icon tables by the number, and its place on the
//!   texture (0xAD, 0x68 for the first and 0x94 for the second);
//! - `scene::upload_car_icon`.
//!
//! Refuses a number of 0x24 or more (the tables' room) and a car without its block.

use crate::errors::Refused;
use crate::scene::car_icon_number::car_icon_number;
use crate::scene::file_numbers_store::texts;
use crate::scene::upload_car_icon::{CarIcon, upload_car_icon};

// ---- game data ----
/// `files\b\tx1822.pcx` .. `tx1825.pcx`.
const NAMES: [usize; 4] = [
    texts::CAR_ICON_0,
    texts::CAR_ICON_1,
    texts::CAR_ICON_2,
    texts::CAR_ICON_3,
];

// Logic: `rc2k_core::scene::car_icons`
pub fn load_hud_car_icons(
    scene: &crate::scene::owner::Scene,
    race: &crate::race::owner::Race,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
    net_race: &rc2k_core::net::net_race::NetRaceSlot,
) -> Result<(), Refused> {
    let shared = &race.shared;
    use rc2k_core::scene::car_icons;
    let texture: u32 = scene.textures.kinds.get().kind_first_textures[193].get();
    if texture == u32::MAX {
        return Ok(());
    }
    let player = if let Some(net_race) = net_race.get() {
        let car = car_icons::one_car(net_race.player)?;
        shared
            .race_car_or(car, "a car without its block")?
            .car_kind
            .get()
    } else {
        race.car_blocks().cars[0].car_kind.get()
    };
    let players = shared.view.get().players.get();
    if players > 2 && net_race.get().is_none() {
        // Three or four players: each part's own car, at half size, four to the first row's place.
        for n in 0..players.min(4) as usize {
            let number = car_icon_number(race, race.car_blocks().cars[n].car_kind.get());
            car_icons::check_numbers([number, 0])?;
            let names = car_icons::pictures(number);
            let [target_x, target_y] = car_icons::half_place(n);
            upload_car_icon(
                &CarIcon {
                    sheet: scene.path_text(NAMES[names]),
                    alpha_sheet: scene.path_text(NAMES[names + 1]),
                    texture,
                    sheet_x: scene.textures.places.get().car_icon_sheet_x[number as usize].get(),
                    sheet_y: scene.textures.places.get().car_icon_sheet_y[number as usize].get(),
                    target_x,
                    target_y,
                    half: true,
                },
                gfx,
                system,
            )?;
        }
        return Ok(());
    }
    let numbers = [
        car_icon_number(race, player),
        car_icon_number(race, race.car_blocks().cars[1].car_kind.get()),
    ];
    car_icons::check_numbers(numbers)?;
    for (number, target_y) in numbers.into_iter().zip(car_icons::ROWS) {
        let names = car_icons::pictures(number);
        upload_car_icon(
            &CarIcon {
                sheet: scene.path_text(NAMES[names]),
                alpha_sheet: scene.path_text(NAMES[names + 1]),
                texture,
                sheet_x: scene.textures.places.get().car_icon_sheet_x[number as usize].get(),
                sheet_y: scene.textures.places.get().car_icon_sheet_y[number as usize].get(),
                target_x: car_icons::COLUMN,
                target_y,
                half: false,
            },
            gfx,
            system,
        )?;
    }
    Ok(())
}
