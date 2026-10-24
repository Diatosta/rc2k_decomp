//! The physics patches a replay was recorded with (IDA 2026-10-19):
//! - `race::apply_replay_patches` (0x41F2C9): when the replay's header has them
//!   (`dword_719C54`) and they are not in force (`dword_5C2C94` 0): in force, the game's
//!   switches (`g_patchSwitch`, 13 dwords) kept in `unk_5C2C98`, the replay's
//!   (`unk_719C58`) put in their place, `g_patchesApplied` 0 and the patches applied
//!   (`physics::apply_patches`).
//! - `race::restore_replay_patches` (0x41F31D): when the header has them and they are in
//!   force: no longer, the kept switches back, applied again.
//!
//! Both keep every register (`pusha` / `popa`).

use crate::errors::Refused;
use crate::race::viewer::{get, move_dwords, put};

const IN_HEADER: usize = 0x719C54; // dword_719C54
const IN_FORCE: usize = 0x5C2C94; // dword_5C2C94
const SWITCHES: usize = 0x71C6D0; // g_patchSwitch, 13 dwords
const KEPT: usize = 0x5C2C98; // unk_5C2C98
const REPLAYS: usize = 0x719C58; // unk_719C58
const APPLIED: usize = 0x71C72C; // g_patchesApplied

/// `apply_replay_patches`.
pub fn apply_replay_patches() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{patches_change, PATCH_DWORDS}`.
    use rc2k_core::race::viewer_flow::{PATCH_DWORDS, patches_change};
    if patches_change(get(IN_HEADER), get(IN_FORCE), false) {
        put(IN_FORCE, 1);
        move_dwords(KEPT, SWITCHES, PATCH_DWORDS);
        move_dwords(SWITCHES, REPLAYS, PATCH_DWORDS);
        put(APPLIED, 0);
        unsafe { crate::physics::apply_patches::apply_patches() };
    }
    Ok(())
}

/// `restore_replay_patches`.
pub fn restore_replay_patches() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{patches_change, PATCH_DWORDS}`.
    use rc2k_core::race::viewer_flow::{PATCH_DWORDS, patches_change};
    if patches_change(get(IN_HEADER), get(IN_FORCE), true) {
        put(IN_FORCE, 0);
        move_dwords(SWITCHES, KEPT, PATCH_DWORDS);
        put(APPLIED, 0);
        unsafe { crate::physics::apply_patches::apply_patches() };
    }
    Ok(())
}
