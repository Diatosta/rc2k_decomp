//! `scene::car_icon_number` (0x41F519): a known kind of car's icon number (its place on the HUD's car icon sheets).
//!
//! Takes: which of the known kinds (held to 0 .. 0x21, signed)
//! Returns: the number
//!
//! `dword_5C5A38` at the place: 0x22 dwords, one for each known kind of car. Two call sites in
//! `scene::load_hud_car_icons`.

use crate::race::kind_store::KIND_TABLES;

// ---- Rust version ----

pub fn car_icon_number(place: u32) -> u32 {
    // Converted: `rc2k_core::scene::kinds::known_place`.
    let place = rc2k_core::scene::kinds::known_place(place);
    KIND_TABLES.car_icon_numbers[place].get()
}
