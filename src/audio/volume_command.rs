//! `audio::volume_command` (0x40B016): command 8: the master volume.
//!
//! In:  eax = the volume (the sender's eax)
//! Out: every register preserved
//!
//! Nothing for a negative volume; else it is kept in `g_masterVolume` and
//! `audio::set_master_volume` called with it. One caller, `audio::take_commands`.
//! Manual patch with a private copy (a short `js` in its first bytes; its call relocated).
//! Shadow mode runs the original alone. Self-test: a redirected copy whose variable is the
//! test's and whose call notes eax, 8 cases.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::audio::voices::*;

// ---- Rust version ----

/// `master`: where the volume is kept; `set`: the master volume's setting.
pub unsafe fn volume_command(volume: u32, master: u32, set: &mut dyn FnMut(u32)) {
    // Converted: `rc2k_core::audio::levels::kept_master_volume`.
    let Some(volume) = rc2k_core::audio::levels::kept_master_volume(volume) else {
        return;
    };
    unsafe { write(master, volume) };
    set(volume);
}
