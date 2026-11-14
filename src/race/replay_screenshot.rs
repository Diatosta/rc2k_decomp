//! `race::replay_screenshot`: while the replay viewer's frame-picture saving is on
//! (`replay_recording` 1, set by `race::start_frame_pictures` from the viewer's actions 0x25 ..
//! 0x28 and cleared by `race::stop_frame_pictures`; the "recording" is of pictures, not of the
//! replay), saves the frame as a picture. The clock is started at the first
//! (`PICTURE_CLOCK.started` 1, `PICTURE_CLOCK.clock` the tick count); the six digits of the name
//! `rpl\000000.pcx` count one more (each past '9' back to '0' and the one before raised; all six
//! at most); the name is set as
//! `Picture.name` while the surface is locked, the picture is written (`gfx::write_screen_pcx`)
//! and the surface unlocked (`gfx::unlock_surface`), then `Picture.name` is put back; the clock
//! moves on by `ViewerAndKinds.viewer_panel.untyped_tail[2]` (the recording's frame time) for the
//! next.
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
    if crate::system::tick_count::PICTURE_CLOCK.started.get() != 1 {
        crate::system::tick_count::PICTURE_CLOCK.started.set(1u32);
        crate::system::tick_count::PICTURE_CLOCK
            .clock
            .set(crate::os::ticks());
    }
    let name = &panel.picture_name;
    // Logic in `rc2k_core::race::viewer::count_up` (the name's six digits from +4).
    // The name is a text: its digits are bytes inside it.
    let mut digits = [0u8; 6];
    Store::load(name, 4, &mut digits)?;
    let (counted, _, _) = rc2k_core::race::viewer::count_up(digits);
    Store::store(name, 4, &counted)?;
    let name = crate::files::cstring(name, "the picture name")?;
    // The surface locked (the carry says it was not), the picture written, the surface
    // unlocked.
    if crate::platform::lock_surface(gfx, system) >= 1 {
        crate::gfx::write_screen_pcx::write_screen_pcx(&gfx.shots, Some(&name), gfx, system)?;
        crate::platform::unlock_surface(gfx);
    }
    crate::system::tick_count::PICTURE_CLOCK.clock.set(
        crate::system::tick_count::PICTURE_CLOCK
            .clock
            .get()
            .wrapping_add(panel.picture_time()),
    );
    Ok(())
}
