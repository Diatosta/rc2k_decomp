//! `race::replay_screenshot` (0x41F3DB): while the replay viewer's frame-picture saving is on
//! (`replay_recording` = `g_replayRecording` 1, set by `race::start_frame_pictures` from the viewer's
//! actions 0x25 .. 0x28, cleared by `race::stop_frame_pictures`; the "recording" is of pictures,
//! not of the replay), the frame saved as a picture: the clock started at the first
//! (`pictures_started` 1, `picture_clock` = `GetTickCount`); the six digits of the name `rpl\000000.pcx`
//! (0x5C2CDC) one more (each past '9' back to '0' and the one before raised; all six at most); the
//! name as `Picture.name` while the surface is locked, written (`gfx::write_screen_pcx`, the PCX
//! writer) and unlocked (`gfx::unlock_surface`), `Picture.name` put back; the clock on by
//! `ViewerAndKinds.viewer_panel.untyped_tail[2]` (the recording's frame time) for the next.
//!
//! Caller: `drive::race_frame` (each frame drawn).

use crate::cells::Store;
use crate::errors::Refused;

pub fn replay_screenshot(
    race: &crate::race::owner::Race,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let panel = race.viewer_panel();
    if panel.replay_recording.get() != 1 {
        return Ok(());
    }
    if panel.pictures_started.get() != 1 {
        panel.pictures_started.set(1u32);
        panel.picture_clock.set(crate::os::ticks());
    }
    let name = &panel.picture_name;
    // Converted: `rc2k_core::race::viewer::count_up` (the name's six digits from +4).
    // The name is a text: its digits are bytes inside it.
    let mut digits = [0u8; 6];
    Store::load(name, 4, &mut digits)?;
    let (counted, _, _) = rc2k_core::race::viewer::count_up(digits);
    Store::store(name, 4, &counted)?;
    let name = crate::files::cstring(name, "the picture name")?;
    // The surface locked (the carry says it was not), the picture written, the surface
    // unlocked.
    if crate::platform::lock_surface(gfx, system) >= 1 {
        crate::gfx::write_screen_pcx::write_screen_pcx(&gfx.shots, Some(&name), gfx)?;
        crate::platform::unlock_surface(gfx);
    }
    panel
        .picture_clock
        .set(panel.picture_clock.get().wrapping_add(panel.picture_time()));
    Ok(())
}
