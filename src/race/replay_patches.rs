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
use crate::race::owner::race;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::session::Dword;

/// `g_patchSwitch` and the twelve settings after it (13 dwords from 0x71C6D0).
pub fn switches() -> impl Iterator<Item = &'static Dword> {
    let shared = &race().shared;
    std::iter::once(&shared.work.patch_switch).chain(shared.work.patch_settings.iter())
}

/// `rep movsd` of the dwords, forwards.
fn move_dwords<'a>(to: impl Iterator<Item = &'a Dword>, from: impl Iterator<Item = &'a Dword>) {
    for (to, from) in to.zip(from) {
        to.set(from.get());
    }
}

/// `apply_replay_patches`.
pub fn apply_replay_patches() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::patches_change`.
    use rc2k_core::race::viewer_flow::patches_change;
    if patches_change(
        REPLAY_WORK.patches_in_header.get(),
        VIEWER_PANEL.patches_in_force.get(),
        false,
    ) {
        VIEWER_PANEL.patches_in_force.set(1);
        move_dwords(VIEWER_PANEL.kept_patch_switches.iter(), switches()); // unk_5C2C98
        move_dwords(switches(), REPLAY_WORK.header_patches.iter()); // unk_719C58
        race().shared.work.patches_applied.set(0);
        crate::physics::apply_patches::apply_patches();
    }
    Ok(())
}

/// `restore_replay_patches`.
pub fn restore_replay_patches() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::patches_change`.
    use rc2k_core::race::viewer_flow::patches_change;
    if patches_change(
        REPLAY_WORK.patches_in_header.get(),
        VIEWER_PANEL.patches_in_force.get(),
        true,
    ) {
        VIEWER_PANEL.patches_in_force.set(0);
        move_dwords(switches(), VIEWER_PANEL.kept_patch_switches.iter());
        race().shared.work.patches_applied.set(0);
        crate::physics::apply_patches::apply_patches();
    }
    Ok(())
}
