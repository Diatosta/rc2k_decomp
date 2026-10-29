//! `race::viewer_snapshot` (0x41EFAB): the viewer's frame saved as a picture: the three
//! digits of `savedata\snapshot\000.pcx` one more (999 round to 000; the number found by
//! `race::find_snapshot_number`), the name as `dword_4F49C8` while the surface is locked
//! (`gfx::lock_surface`; failing, nothing), written (`gfx::write_screen_pcx`) and
//! unlocked (`gfx::unlock_surface`); `dword_4F49C8` put back.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read (the caller `test`s eax)
//!
//! Caller: `race::viewer_frame` (when `dword_5C2540` asks), which calls it directly; the
//! lock and unlock (the platform's) and the writer are called directly (2026-10-23).

use crate::errors::Refused;
use crate::gfx::picture_store::PICTURE;
use crate::race::find_snapshot_number::{SNAPSHOT, digits_place};

/// The Rust version (see the header).
pub fn viewer_snapshot() -> Result<(), Refused> {
    let kept: u32 = PICTURE.name.get();
    let digits = digits_place();
    // Converted: `rc2k_core::race::viewer::next_snapshot`.
    unsafe {
        let (next, _, _) = rc2k_core::race::viewer::next_snapshot(digits.cast::<[u8; 3]>().read());
        digits.cast::<[u8; 3]>().write(next);
    }
    PICTURE.name.set(SNAPSHOT.addr());
    // `cmp g_surfaceLocked, 1`'s carry: not locked.
    if crate::platform::lock_surface() >= 1 {
        { crate::gfx::write_screen_pcx::write_screen_pcx() }?;
        crate::platform::unlock_surface();
    }
    PICTURE.name.set(kept);
    Ok(())
}
