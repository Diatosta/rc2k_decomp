//! `records::seal_time` (0x40264D): a time sealed for the records. The time, at most
//! 0x22550FF; negated when a cheat of 0x16000 is on, or negated and bit 30 cleared when
//! cheat 0x40000 is on or `g_patchSwitch` is set; into the block's first dword
//! (the seal's first dword), then its next three bytes, its eighth byte 0x5A plus the
//! seven before it each inverted (a byte sum); then `race::seal_block`.
//!
//! Caller: `menu::sealed_player_time`.

use crate::race::save_seal::seal_block;
/// The time as sealed (converted: `rc2k_core::files::records::sealed_value`).
pub fn sealed_value(time: u32, cheats: u32, patch_switch: u32) -> u32 {
    rc2k_core::files::records::sealed_value(time, cheats, patch_switch != 0)
}

/// The block's eight bytes (converted: `rc2k_core::files::records::seal_block`).
pub use rc2k_core::files::records::seal_block as block;

/// `time` and the block's three bytes in; the sealed block's two dwords out.
pub fn seal_time(time: u32, bytes: [u8; 3]) -> [u32; 2] {
    let o = crate::owners::boundary();
    let replay = &o.race.replay;
    let value = sealed_value(
        time,
        o.menus.play.cheats.cheats.get(),
        o.race.shared.work.patch_switch.bits(),
    );
    let bytes = block(value, bytes);
    let [b0, b1, b2, b3, b4, b5, b6, b7] = bytes;
    replay.seal_work.block[0].set(u32::from_le_bytes([b0, b1, b2, b3]));
    replay.seal_work.block[1].set(u32::from_le_bytes([b4, b5, b6, b7]));
    seal_block(&replay.seal_work);
    [
        replay.seal_work.sealed[0].get(),
        replay.seal_work.sealed[1].get(),
    ]
}
