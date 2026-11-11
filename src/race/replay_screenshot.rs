//! `race::replay_screenshot` (0x41F3DB): while the replay viewer's frame-picture saving is on
//! (`replay_recording` = `g_replayRecording` 1, set by `race::start_frame_pictures` from the viewer's
//! actions 0x25 .. 0x28, cleared by `race::stop_frame_pictures`; the "recording" is of pictures,
//! not of the replay), the frame saved as a picture: the clock started at the first
//! (`pictures_started` 1, `picture_clock` = `GetTickCount`); the six digits of the name `rpl\000000.pcx`
//! (0x5C2CDC) one more (each past '9' back to '0' and the one before raised; all six at most); the
//! name as `dword_4F49C8` while the surface is locked, written (`gfx::write_screen_pcx`, the PCX
//! writer) and unlocked (`gfx::unlock_surface`), `dword_4F49C8` put back; the clock on by
//! `dword_5C2CF3` (the recording's frame time) for the next.
//!
//! Caller: `drive::race_frame` (each frame drawn).

use crate::errors::Refused;
use crate::pointers::Store;
use crate::race::viewer_screen_store::VIEWER_PANEL;

pub fn replay_screenshot() -> Result<(), Refused> {
    let gfx = crate::gfx::owner::gfx();
    if VIEWER_PANEL.replay_recording.get() != 1 {
        return Ok(());
    }
    if VIEWER_PANEL.pictures_started.get() != 1 {
        VIEWER_PANEL.pictures_started.set(1u32);
        VIEWER_PANEL.picture_clock.set(crate::os::ticks());
    }
    let name = &VIEWER_PANEL.picture_name;
    // Converted: `rc2k_core::race::viewer::count_up` (the name's six digits from +4).
    // The name is a text: its digits are bytes inside it.
    let mut digits = [0u8; 6];
    Store::load(name, 4, &mut digits)?;
    let (counted, _, _) = rc2k_core::race::viewer::count_up(digits);
    Store::store(name, 4, &counted)?;
    let name = crate::files::cstring(name, "the picture name")?;
    // The surface locked (the carry says it was not), the picture written, the surface
    // unlocked.
    if crate::platform::lock_surface() >= 1 {
        crate::gfx::write_screen_pcx::write_screen_pcx(&gfx.shots, Some(&name), gfx)?;
        crate::platform::unlock_surface();
    }
    VIEWER_PANEL.picture_clock.set(
        VIEWER_PANEL
            .picture_clock
            .get()
            .wrapping_add(VIEWER_PANEL.picture_time()),
    );
    Ok(())
}
