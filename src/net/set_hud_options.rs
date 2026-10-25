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

use crate::net::lobby::{g, p};
use crate::net::reset_cars::CAR_BLOCKS;
use crate::net::store::NET;
use crate::race::state_store::RACE_STATE;
use crate::session::Dword;
use rc2k_core::net::cars;

/// The switches, in the original's order (their bits: `rc2k_core::net::cars::HUD_SWITCH_BITS`).
const SWITCHES: [&Dword; 9] = [
    &RACE_STATE.settings.cars[0].map_area,
    &RACE_STATE.settings.cars[0].rear_mirror,
    &RACE_STATE.settings.cars[0].rev_counter,
    &RACE_STATE.settings.cars[0].message_display,
    &RACE_STATE.settings.cars[0].driver_info,
    &RACE_STATE.settings.cars[0].metric_units,
    &RACE_STATE.settings.cars[0].progress_bar,
    &RACE_STATE.settings.cars[0].car_info,
    &RACE_STATE.settings.cars[0].pace_notes,
];
/// The settings copied: (to, from).
const COPIED: [(usize, usize); 1] = [(0x623898, 0x4F50D0)];
/// The settings words copied after them to the race's variables: (to, from).
const COPIED_WORDS: [(usize, &Dword); 5] = [
    (0x6238A8, &RACE_STATE.settings.setting_0),
    (0x6238AC, &RACE_STATE.settings.setting_1),
    (0x6238B0, &RACE_STATE.settings.setting_2),
    (0x6238B4, &RACE_STATE.settings.setting_3),
    (0x6238B8, &RACE_STATE.settings.setting_4),
];

/// This machine's car's block.
pub fn own_block() -> u32 {
    g(CAR_BLOCKS + 4 * NET.player.get() as usize)
}

/// The Rust version.
pub fn set_hud_options() {
    let block = own_block();
    let mut bits = RACE_STATE.results.hud_bits_current.get();
    // Converted: `rc2k_core::net::cars::hud_bits`.
    if bits == cars::NOT_KEPT {
        bits = cars::hud_bits(SWITCHES.map(|cell| cell.get()));
    }
    unsafe {
        ((block + 0x220) as *mut u32).write_unaligned(bits);
        ((block + 0x684) as *mut u32).write_unaligned(RACE_STATE.settings.cars[0].view.get());
    }
    for (to, from) in COPIED {
        p(to, g(from));
    }
    for (to, from) in COPIED_WORDS {
        p(to, from.get());
    }
}
