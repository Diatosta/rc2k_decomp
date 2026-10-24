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

use crate::net::lobby::{PLAYER, g, p};
use crate::net::reset_cars::CAR_BLOCKS;
use rc2k_core::net::cars;

pub const HUD_BITS: usize = 0x75B00C; // dword_75B00C
/// The switches, in the original's order (their bits: `rc2k_core::net::cars::HUD_SWITCH_BITS`).
const SWITCHES: [usize; 9] = [
    0x75ADF8, 0x75ADFC, 0x75AE00, 0x75AE08, 0x75AE14, 0x75AE04, 0x75AE10, 0x75AE0C, 0x75AE18,
];
const VIEW: usize = 0x75AE24;
/// The settings copied: (to, from).
const COPIED: [(usize, usize); 6] = [
    (0x623898, 0x4F50D0),
    (0x6238A8, 0x75AD00),
    (0x6238AC, 0x75AD0C),
    (0x6238B0, 0x75AD10),
    (0x6238B4, 0x75AD14),
    (0x6238B8, 0x75AD18),
];

/// This machine's car's block.
pub fn own_block() -> u32 {
    g(CAR_BLOCKS + 4 * g(PLAYER) as usize)
}

/// The Rust version.
pub fn set_hud_options() {
    let block = own_block();
    let mut bits = g(HUD_BITS);
    // Converted: `rc2k_core::net::cars::hud_bits`.
    if bits == cars::NOT_KEPT {
        bits = cars::hud_bits(SWITCHES.map(g));
    }
    unsafe {
        ((block + 0x220) as *mut u32).write_unaligned(bits);
        ((block + 0x684) as *mut u32).write_unaligned(g(VIEW));
    }
    for (to, from) in COPIED {
        p(to, g(from));
    }
}
