//! `net::set_hud_options` (0x463D3F): this machine's car (`g_carBlocks[g_netPlayer]`) given
//! the player's HUD: its bits (+0x220) from `dword_75B00C`, or, when that is -1, made from
//! the switches (`dword_75ADF8` 0x10, `75ADFC` 8, `75AE00` 1, `75AE08` 4, `75AE14` 2,
//! `75AE04` 0x100, `75AE10` 0x40, `75AE0C` 0x20, `75AE18` 0x80, each when not 0); its view
//! (+0x684) from `dword_75AE24`; `dword_623898` from `dword_4F50D0`; five more settings
//! (`dword_75AD00`, `75AD0C` .. `75AD18`) to `dword_6238A8` ...
//!
//! In:  nothing
//! Out: esi = the car's block, eax = its HUD bits; with the switches read, ebx = the last
//!      (`dword_75AE18`); the flags of the last compare (no caller reads them)
//! Caller: `net::lets_race`. Shadow mode: both run, the block's two dwords and the
//! settings compared. Self-test: a copy against the Rust version, on bits given and on
//! the switches in two patterns, on a test block.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{PLAYER, Part, g, p};
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

/// The Rust version: the registers it leaves.
pub fn set_hud_options(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = own_block();
    r.eax = g(HUD_BITS);
    // Converted: `rc2k_core::net::cars::hud_bits`.
    if r.eax == cars::NOT_KEPT {
        let switches = SWITCHES.map(g);
        r.eax = cars::hud_bits(switches);
        r.ebx = switches[8];
    }
    unsafe {
        ((r.esi + 0x220) as *mut u32).write_unaligned(r.eax);
        ((r.esi + 0x684) as *mut u32).write_unaligned(g(VIEW));
    }
    for (to, from) in COPIED {
        p(to, g(from));
    }
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        set_hud_options(r, &mut crate::net::lobby::game_part)
    })
}
