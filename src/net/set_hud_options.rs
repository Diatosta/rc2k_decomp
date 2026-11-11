//! `net::set_hud_options` (0x463D3F): this machine's car (`g_carBlocks[g_netPlayer]`) given
//! the player's HUD: its bits (+0x220) from the player's saved HUD bits, or, when that is
//! -1, made from the switches (rear mirror 0x10, rev counter 8, message display 1, driver
//! info 4, metric units 2, progress bar 0x100, car info 0x40, pace notes 0x20 and the
//! last one 0x80, each when not 0); its view (+0x684) from the player's view setting;
//! `mip_setting` from the graphics options' mipmapping; five more settings (lens flare,
//! reflections, car shadows, car detail, headlights) to `settings_words`.
//!
//! Caller: `net::lets_race`.

use crate::gfx::owner::gfx;
use crate::net::store::NetVars;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use crate::session::Dword;
use rc2k_core::net::cars;

/// The switches, in the original's order (their bits: `rc2k_core::net::cars::HUD_SWITCH_BITS`).
const SWITCHES: [&Dword; 9] = [
    &crate::race::owner::RACE.shared.state.settings.cars[0].map_area,
    &crate::race::owner::RACE.shared.state.settings.cars[0].rear_mirror,
    &crate::race::owner::RACE.shared.state.settings.cars[0].rev_counter,
    &crate::race::owner::RACE.shared.state.settings.cars[0].message_display,
    &crate::race::owner::RACE.shared.state.settings.cars[0].driver_info,
    &crate::race::owner::RACE.shared.state.settings.cars[0].metric_units,
    &crate::race::owner::RACE.shared.state.settings.cars[0].progress_bar,
    &crate::race::owner::RACE.shared.state.settings.cars[0].car_info,
    &crate::race::owner::RACE.shared.state.settings.cars[0].pace_notes,
];
/// The settings words copied to the race's variables (`crate::race::owner::race().shared.vars.settings_words`).
const COPIED_WORDS: [&Dword; 5] = [
    &crate::race::owner::RACE.shared.state.settings.lens_flare,
    &crate::race::owner::RACE.shared.state.settings.reflections,
    &crate::race::owner::RACE.shared.state.settings.car_shadows,
    &crate::race::owner::RACE.shared.state.settings.car_detail,
    &crate::race::owner::RACE.shared.state.settings.headlights,
];

/// This machine's car's block.
pub fn own_block(
    shared: &RaceShared,
    vars: &NetVars,
) -> Result<&'static crate::physics::car_store::Car, crate::errors::BadInput> {
    shared.work.car(vars.player.get() as usize)
}
pub fn set_hud_options(race: &Race, vars: &NetVars) -> Result<(), crate::errors::BadInput> {
    let mut bits = race.shared.state.results.hud_bits_current.get();
    // Converted: `rc2k_core::net::cars::hud_bits`.
    if bits == cars::NOT_KEPT {
        bits = cars::hud_bits(SWITCHES.map(super::super::session::Dword::get));
    }
    let block = own_block(&race.shared, vars)?;
    block.option_bits.set(bits);
    block
        .camera_view
        .set(race.shared.state.settings.cars[0].view.get());
    race.shared
        .vars
        .mip_setting
        .set(gfx().display.choices.mipmapping.get());
    for (to, from) in race.shared.vars.settings_words.iter().zip(COPIED_WORDS) {
        to.set(from.get());
    }
    Ok(())
}
