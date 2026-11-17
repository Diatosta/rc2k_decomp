//! The replay viewer's "render to video" button, a third row under the two rows of buttons: the
//! snapshot button's camera picture, centred, which the down arrow reaches from the lower row.
//! Enter on it plays the whole replay from its start into a video file (`video`), asking nothing: the screen's size, 60 frames a second, a file in
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

pub fn selected() -> bool {
    SELECTED.load(Relaxed)
}

pub fn select(on: bool) {
    SELECTED.store(on, Relaxed);
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

/// The lower row's snapshot button, whose camera picture the render button shows.
const CAMERA_ITEM: u32 = 23;

/// The render button drawn on the viewer's frame, after the list's items: the snapshot button's
/// picture, centred under the lower row as far below it as the lower row is below the upper,
/// lit as a highlighted button is when it has the highlight.
pub fn draw_button(
    race: &Race,
    list_state: &crate::menu::list_state::ListState,
    system: &crate::system::owner::System,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    use crate::race::viewer_state_store::{ReplayViewBlock, ViewerPicture};
    let replay = &race.replay;
    let panel = race.viewer_panel();
    let ViewerPicture::Rectangle(ida) = ReplayViewBlock::item_picture(replay, CAMERA_ITEM)? else {
        return Ok(());
    };
    let uv = panel.rectangle(ida)?;
    let place = |index: u32| -> Result<[u32; 4], Refused> {
        let bytes = ReplayViewBlock::item_place(replay, index)?
            .read_array::<16>("a viewer item's place")?;
        Ok(std::array::from_fn(|k| {
            u32::from_le_bytes([
                bytes[4 * k],
                bytes[4 * k + 1],
                bytes[4 * k + 2],
                bytes[4 * k + 3],
            ])
        }))
    };
    let (upper, lower) = (place(0)?, place(*LOWER_ROW.start())?);
    let width = (uv[2] as i32).unsigned_abs();
    let y = lower[1].wrapping_add(lower[1].wrapping_sub(upper[1]));
    panel.item_place[0].set(400u32.wrapping_sub(width / 2));
    panel.item_place[1].set(y);
    panel.item_place[2].set(width);
    panel.item_place[3].set(lower[3]);
    let colour = rc2k_core::race::viewer::item_colour(
        0,
        selected(),
        || false,
        || {
            list_state
                .frame_clock
                .get()
                .wrapping_sub(list_state.move_clock.get())
        },
    );
    crate::drive::tyre_quad::tyre_quad(
        uv,
        &panel.item_place,
        colour,
        panel,
        system,
        scene.scratch.game_scratch(),
        rc2k_core::hud::anchor::Spread::Centred,
    )
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
