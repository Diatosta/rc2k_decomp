//! `net::set_hud_options` (0x463D3F): this machine's car (`g_carBlocks[g_netPlayer]`) given
//! the player's HUD: its bits (+0x220) from `dword_75B00C`, or, when that is -1, made from
//! the switches (`dword_75ADF8` 0x10, `75ADFC` 8, `75AE00` 1, `75AE08` 4, `75AE14` 2,
//! `75AE04` 0x100, `75AE10` 0x40, `75AE0C` 0x20, `75AE18` 0x80, each when not 0); its view
//! (+0x684) from `dword_75AE24`; `dword_623898` from `dword_4F50D0`; five more settings
//! (`dword_75AD00`, `75AD0C` .. `75AD18`) to `dword_6238A8` ...
//!
//! In:  nothing
//! Out: nothing (the original leaves esi the car's block, eax its HUD bits, ebx the last
//!      switch read; its one caller, `net::lets_race`, calls `drive::play_race` next, which
//!      reads no register: `lq 0x444CEE`, live_in empty)
//! Caller: `net::lets_race`.

use crate::gfx::owner::gfx;
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
    &crate::race::owner::RACE.shared.state.settings.setting_0,
    &crate::race::owner::RACE.shared.state.settings.setting_1,
    &crate::race::owner::RACE.shared.state.settings.setting_2,
    &crate::race::owner::RACE.shared.state.settings.setting_3,
    &crate::race::owner::RACE.shared.state.settings.setting_4,
];

/// This machine's car's block.
pub fn own_block() -> u32 {
    crate::race::owner::race()
        .shared
        .work
        .car_block(crate::net::owner::net().vars.player.get() as usize)
}

/// The Rust version.
pub fn set_hud_options() -> Result<(), crate::errors::BadInput> {
    let block = own_block();
    let mut bits = crate::race::owner::race()
        .shared
        .state
        .results
        .hud_bits_current
        .get();
    // Converted: `rc2k_core::net::cars::hud_bits`.
    if bits == cars::NOT_KEPT {
        bits = cars::hud_bits(SWITCHES.map(super::super::session::Dword::get));
    }
    let block = crate::race::trap::car_at(block)?;
    block.option_bits.set(bits);
    block.camera_view.set(
        crate::race::owner::race().shared.state.settings.cars[0]
            .view
            .get(),
    );
    crate::race::owner::race()
        .shared
        .vars
        .mip_setting
        .set(gfx().display.choices.mipmapping.get());
    for (to, from) in crate::race::owner::race()
        .shared
        .vars
        .settings_words
        .iter()
        .zip(COPIED_WORDS)
    {
        to.set(from.get());
    }
    Ok(())
}
