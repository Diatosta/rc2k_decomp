//! `race::viewer_snapshot` (0x41EFAB): the viewer's frame saved as a picture: the three digits of
//! `savedata\snapshot\000.pcx` one more (999 round to 000; the number found by
//! `race::find_snapshot_number`), the name as `Picture.name` while the surface is locked
//! (`gfx::lock_surface`; failing, nothing), written (`gfx::write_screen_pcx`) and unlocked
//! (`gfx::unlock_surface`); `Picture.name` put back.
//!
//! Caller: `race::viewer_frame` (when `snapshot_asked` asks), which calls it directly; the
//! lock and unlock (the platform's) and the writer are called directly.

use crate::errors::Refused;
use crate::race::find_snapshot_number::DIGITS_AT;
use crate::race::owner_replay::Replay;

pub fn viewer_snapshot(replay: &Replay, gfx: &crate::gfx::owner::Gfx) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer::next_snapshot`.
    let what = "the snapshot's digits";
    let digits: [u8; 3] = replay
        .snapshot
        .read_at(DIGITS_AT, 3, what)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e))
        .try_into()
        .expect("3 bytes");
    let (next, _, _) = rc2k_core::race::viewer::next_snapshot(digits);
    replay
        .snapshot
        .write_at(DIGITS_AT, &next, what)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
    // `cmp g_surfaceLocked, 1`'s carry: not locked.
    if crate::platform::lock_surface(gfx) >= 1 {
        crate::gfx::write_screen_pcx::write_screen_pcx(
            &gfx.shots,
            Some(&replay.snapshot.text()),
            gfx,
        )?;
        crate::platform::unlock_surface(gfx);
    }
    Ok(())
}
