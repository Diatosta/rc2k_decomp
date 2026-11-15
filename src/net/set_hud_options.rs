//! Gives this machine's car the player's HUD: its bits from the saved HUD bits, or, when that
//! is -1, from the individual switches; its view from the view setting; and the mipmapping, lens
//! flare, reflection, shadow, car detail and headlight settings.

use crate::gfx::owner::Gfx;
use crate::memory::session::PlainDword;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::net::cars;

/// The switches, in order (their bits: `rc2k_core::net::cars::HUD_SWITCH_BITS`).
fn switches(race: &Race) -> [&PlainDword; 9] {
    [
        &race.shared.state.settings.cars[0].map_area,
        &race.shared.state.settings.cars[0].rear_mirror,
        &race.shared.state.settings.cars[0].rev_counter,
        &race.shared.state.settings.cars[0].message_display,
        &race.shared.state.settings.cars[0].driver_info,
        &race.shared.state.settings.cars[0].metric_units,
        &race.shared.state.settings.cars[0].progress_bar,
        &race.shared.state.settings.cars[0].car_info,
        &race.shared.state.settings.cars[0].pace_notes,
    ]
}
/// The settings words copied to the race's variables
/// (`crate::test_support::own::race().shared.vars.settings_words`).
fn copied_words(race: &Race) -> [&PlainDword; 5] {
    [
        &race.shared.state.settings.lens_flare,
        &race.shared.state.settings.reflections,
        &race.shared.state.settings.car_shadows,
        &race.shared.state.settings.car_detail,
        &race.shared.state.settings.headlights,
    ]
}

/// This machine's car's block.
pub fn own_block(
    shared: &RaceShared,
    player: u32,
) -> Result<&crate::physics::car_store::Car, crate::errors::BadInput> {
    shared.race_car(player as usize)
}
pub fn set_hud_options(
    race: &Race,
    net: &crate::net::owner::Net,
    gfx: &Gfx,
) -> Result<(), crate::errors::BadInput> {
    let mut bits = race.shared.state.results.hud_bits_current.get();
    // The logic is in `rc2k_core::net::cars::hud_bits`.
    if bits == cars::NOT_KEPT {
        bits = cars::hud_bits(switches(race).map(PlainDword::get));
    }
    let block = own_block(&race.shared, net.race_slot.player())?;
    block.option_bits.set(bits);
    block
        .camera_view
        .set(race.shared.state.settings.cars[0].view.get());
    race.shared
        .vars
        .get()
        .mip_setting
        .set(gfx.display.choices.mipmapping.get());
    for (to, from) in race
        .shared
        .vars
        .get()
        .settings_words
        .iter()
        .zip(copied_words(race))
    {
        to.set(from.get());
    }
    Ok(())
}
