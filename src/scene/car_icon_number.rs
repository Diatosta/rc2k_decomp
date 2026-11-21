//! `scene::car_icon_number`: a known kind of car's icon number (its place on the HUD's car icon
//! sheets).
//!
//! Takes which of the known kinds (held to 0 ..= 0x21, signed) and returns
//! `ViewerAndKinds.kind_tables.car_icon_numbers` at that place: 0x22 dwords, one for each known
//! kind of car. Two call sites in `scene::load_hud_car_icons`.

use crate::race::owner::Race;

pub fn car_icon_number(race: &Race, place: u32) -> u32 {
    // Logic: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    race.kind_tables().car_icon_numbers[place]
}
