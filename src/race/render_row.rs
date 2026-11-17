//! The replay viewer's "render to video" row, under the two rows of buttons: a text button the
//! down arrow reaches from the lower row, Enter on it plays the whole replay from its start into
//! a video file (`video`), asking nothing: the screen's size, 60 frames a second, a file in
//! `savedata/videos` named by the date and time. A bar along the screen's foot shows how far the
//! render has come (it is not in the video); Escape stops the render and keeps what was written.
//! The viewer is back where the play ended when the render is over.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// The rate the viewer's renders are made at.
const FPS: u32 = 60;

/// The row has the highlight (the lower row's button is not lit then).
static SELECTED: AtomicBool = AtomicBool::new(false);

/// The viewer's frame showed the row since it was last drawn on the screen.
static VISIBLE: AtomicBool = AtomicBool::new(false);

pub fn selected() -> bool {
    SELECTED.load(Relaxed)
}

pub fn select(on: bool) {
    SELECTED.store(on, Relaxed);
}

/// The viewer's frame has the row to show.
pub fn mark_visible() {
    VISIBLE.store(true, Relaxed);
}

/// The viewer's number of the key table's down, up and left arrows, and Enter.
pub const KEY_UP: u8 = 0x66;
pub const KEY_DOWN: u8 = 0x6B;
pub const KEY_LEFT: u8 = 0x68;
pub const KEY_ENTER: u8 = 0x1C;

/// The lower row's buttons (the down arrow from them reaches the row).
pub const LOWER_ROW: std::ops::RangeInclusive<u32> = 13..=25;

/// The video file's name for a moment: `replay_YYYY-MM-DD_HH-MM-SS.mp4`.
fn file_name(t: &rc2k_core::system::time::LocalTime) -> String {
    format!(
        "replay_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.mp4",
        t.year, t.month, t.day, t.hour, t.minute, t.second
    )
}

/// The row's place, in the viewer's 800 x 600 coordinates.
const BOX: [i32; 4] = [10, 0xB3, 270, 34];
const SCREEN: (i32, i32) = (800, 600);

/// The row drawn on the frame about to be shown, if the viewer's frame has it to show: a button
/// (blue, red when it has the highlight) with its text, written into the frame's pixels as the
/// on-screen keyboard's panel is.
pub fn draw(o: Owners<'static>) {
    use crate::gfx::typefaces::fonts;
    if !VISIBLE.swap(false, Relaxed) {
        return;
    }
    let locked_here = o.gfx.locked.surface.get() != 1;
    if locked_here && crate::platform::lock_surface(o.gfx, o.system) < 1 {
        return;
    }
    let (w, h) = (
        o.gfx.drawing.draw_surface.width.get() as i32,
        o.gfx.drawing.draw_surface.height.get() as i32,
    );
    let at = |x: i32, y: i32| (x * w / SCREEN.0, y * h / SCREEN.1);
    let (left, top) = at(BOX[0], BOX[1]);
    let (right, bottom) = at(BOX[0] + BOX[2], BOX[1] + BOX[3]);
    let colour = if selected() { 0xC8202A } else { 0x27347A };
    crate::input::osk_draw::fill(o, [left, top, right - left, bottom - top], colour);
    let mut canvas = crate::text::draw_text::GameCanvas::new(o.gfx);
    if let Err(why) = crate::text::draw_string_by_id::draw_string_by_id_at(
        crate::strings::append_string_by_id::StringOwners::of(o),
        rc2k_core::strings::own::id::RENDER_VIDEO,
        fonts::LIST,
        left + 8,
        top + (bottom - top) / 2 - 12,
        &mut canvas,
    ) {
        crate::logfile::program_log::log(&format!("viewer: the render row not drawn: {why:?}"));
    }
    if locked_here {
        crate::platform::unlock_surface(o.gfx);
    }
}

/// How far the replay has been played, 0 ..= 1, for the render's bar.
pub fn fraction(race: &Race) -> f64 {
    let blocks = race.shared.replay_file().index.block_count.get().max(1);
    let block = race.replay.replay_view().replay_block.get();
    (f64::from(block) / f64::from(blocks)).clamp(0.0, 1.0)
}

/// The whole replay played into a video file, from its first block. Whatever was written when
/// the play stops (the replay's end, Escape or an error) is kept.
pub fn render(race: &'static Race, o: Owners<'static>) -> Result<(), Refused> {
    let replay = &race.replay;
    if race.shared.replay_file().index.block_count.get() == 0 {
        return Ok(());
    }
    let folder = crate::files::host_path(b"savedata\\videos");
    if let Err(e) = std::fs::create_dir_all(&folder) {
        crate::logfile::program_log::log(&format!("viewer: {}: {e}", folder.display()));
        return Ok(());
    }
    let out = folder.join(file_name(&crate::platform::os::local_time()));
    let now = crate::system::tick_count::tick_count(&o.system.session.picture_clock);
    if !crate::video::begin_in_game(out, FPS, now) {
        return Ok(());
    }
    replay.replay_view().replay_block.set(0);
    replay.replay_view().replay_frame.set(0);
    let played = crate::race::play_marked::play_from_here(race, o);
    crate::video::end_in_game();
    played?;
    // The frame the play ended on is read again, as after a saved replay.
    crate::drive::read_replay_frame::read_replay_frame(
        race,
        o.system,
        replay.replay_view().replay_block.get(),
        replay.replay_view().replay_frame.get(),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_is_named_by_the_date() {
        let t = rc2k_core::system::time::LocalTime {
            year: 2026,
            month: 11,
            weekday: 2,
            day: 17,
            hour: 9,
            minute: 5,
            second: 7,
            milliseconds: 0,
        };
        assert_eq!(file_name(&t), "replay_2026-11-17_09-05-07.mp4");
    }
}
