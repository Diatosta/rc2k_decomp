//! `race::replay_screenshot` (0x41F3DB): while the replay's frames are being recorded
//! (`dword_5C2CCC` 1, a reading), the frame saved as a picture: the clock started at the
//! first (`dword_5C2CD4` 1, `dword_5C2CD0` = `GetTickCount`); the six digits of the name
//! `rpl\000000.pcx` (0x5C2CDC) one more (each past '9' back to '0' and the one before
//! raised; all six at most); the name as `dword_4F49C8` while the surface is locked
//! (`gfx::lock_surface`; carry: not saved), written (`sub_411E0C`, the PCX writer) and
//! unlocked (`gfx::unlock_surface`), `dword_4F49C8` put back; the clock on by `dword_5C2CF3`
//! (the recording's frame time) for the next.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); no flags read
//!
//! Caller: `drive::race_frame` (each frame drawn).

use crate::errors::Refused;
use crate::pointers::GamePtr;
use crate::race::viewer_screen_store::VIEWER_PANEL;

/// The Rust version (see the header). (The original keeps every register.)
pub fn replay_screenshot() -> Result<(), Refused> {
    if VIEWER_PANEL.replay_recording.bits() != 1 {
        return Ok(());
    }
    if VIEWER_PANEL.pictures_started.bits() != 1 {
        VIEWER_PANEL.pictures_started.set_bits(1u32);
        VIEWER_PANEL.picture_clock.set(crate::os::ticks());
    }
    let name = GamePtr::to(&VIEWER_PANEL.picture_name);
    // Converted: `rc2k_core::race::viewer::count_up` (the name's six digits from +4).
    let digits = name.offset(4);
    let (counted, _, _) =
        rc2k_core::race::viewer::count_up(digits.read::<[u8; 6]>("the picture name")?);
    digits.write(counted, "the picture name")?;
    let name = crate::files::cstring(name, "the picture name")?;
    // The surface locked (the carry says it was not), the picture written, the surface
    // unlocked.
    if crate::platform::lock_surface() >= 1 {
        crate::gfx::write_screen_pcx::write_screen_pcx(Some(&name))?;
        crate::platform::unlock_surface();
    }
    VIEWER_PANEL.picture_clock.set(
        VIEWER_PANEL
            .picture_clock
            .bits()
            .wrapping_add(VIEWER_PANEL.picture_time()),
    );
    Ok(())
}
