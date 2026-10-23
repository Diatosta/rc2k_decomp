//! `records::seal_time` (0x40264D): a time sealed for the records. The time eax, at most
//! 0x22550FF; negated when a cheat of 0x16000 is on, or negated and bit 30 cleared when
//! cheat 0x40000 is on or `g_patchSwitch` is set; into the block's first dword
//! (`dword_4E0534`), bl / cl / dl its next three bytes, its eighth byte 0x5A plus the
//! seven before it each inverted (a byte sum); then `race::seal_block`.
//!
//! In:  eax the time, bl, cl, dl three bytes (the car, the stage, ...)
//! Out: eax `value`, ebx `dword_4E0540` (the sealed block); the rest kept
//!
//! Caller: `sub_4A3D38`. The cipher is its Rust version.
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::framework::{game_read, game_write, rva};
use crate::race::save_seal::{BLOCK_RVA, SEALED_RVA, seal_block};

const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const PATCH_SWITCH_RVA: usize = rva(0x71C6D0); // g_patchSwitch
/// The time as sealed (converted: `rc2k_core::files::records::sealed_value`).
pub fn sealed_value(time: u32, cheats: u32, patch_switch: u32) -> u32 {
    rc2k_core::files::records::sealed_value(time, cheats, patch_switch != 0)
}

/// The block's eight bytes (converted: `rc2k_core::files::records::seal_block`).
pub use rc2k_core::files::records::seal_block as block;

/// The Rust version (see the header): `time` and the block's three bytes (the original's
/// bl, cl, dl) in; the sealed block's two dwords (its eax, ebx) out. Its callers read
/// nothing else it leaves (register liveness, 2026-10-23).
pub unsafe fn seal_time(time: u32, bytes: [u8; 3]) -> [u32; 2] {
    unsafe {
        let value = sealed_value(time, game_read(CHEATS_RVA), game_read(PATCH_SWITCH_RVA));
        game_write(BLOCK_RVA, block(value, bytes));
        seal_block();
        SEALED_RVA.map(|r| game_read(r))
    }
}
