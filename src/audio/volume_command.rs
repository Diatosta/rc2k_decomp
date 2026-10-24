//! `audio::volume_command` (0x40B016): command 8: the master volume.
//!
//! In:  eax = the volume (the sender's eax)
//! Out: every register preserved
//!
//! Nothing for a negative volume; else it is kept in `g_masterVolume` and
//! `audio::set_master_volume` called with it. One caller, `audio::take_commands`.

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
