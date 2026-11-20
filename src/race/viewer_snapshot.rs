//! `race::viewer_snapshot`: saves the viewer's frame as a picture. The three digits of
//! `savedata\snapshot\000.png` go up by one (999 rounds to 000; the number is found by
//! `race::find_snapshot_number`), and the name is used as `Picture.name` while the surface is
//! locked (`gfx::lock_surface`; if that fails, nothing is saved), written
//! (`gfx::write_screen_pcx`) and unlocked (`gfx::unlock_surface`); `Picture.name` is put back.

use crate::errors::Refused;
use crate::race::find_snapshot_number::DIGITS_AT;
use crate::race::owner_replay::Replay;

pub fn viewer_snapshot(
    replay: &Replay,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let what = "the snapshot's digits";
    let digits: [u8; 3] = replay
        .snapshot()
        .read_at(DIGITS_AT, 3, what)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e))
        .try_into()
        .expect("3 bytes");
    let (next, _, _) = rc2k_core::race::viewer::next_snapshot(digits);
    replay
        .snapshot()
        .write_at(DIGITS_AT, &next, what)
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
    // Only when the surface could be locked.
    crate::gfx::with_locked_surface::with_locked_surface(
        crate::gfx::with_locked_surface::lock(gfx, system),
        || {
            crate::gfx::write_screen_pcx::write_screen_pcx(
                &gfx.shots,
                Some(&replay.snapshot().text()),
                true,
                gfx,
                system,
            )
        },
    )?;
    Ok(())
}
