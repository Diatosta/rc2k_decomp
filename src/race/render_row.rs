//! The replay viewer's "render to video" button, a third row under the two rows of buttons: the
//! snapshot button's camera picture, centred, which the down arrow reaches from the lower row.
//! Enter on it opens the dialog of `race::render_dialog` (size, frame rate, part of the replay,
//! quality, sound); "render" there has a program of its own render the part chosen into a video
//! file in `savedata/videos`, named by the date and time (`race::render_progress` shows how far it
//! has come), and "cancel" leaves the viewer as it was.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use rc2k_core::race::render_dialog::Part;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

/// The render row's state, one per race (`Replay::render_row`).
pub struct RenderRow {
    /// The blocks a render started from the command line plays, from the first to the end (for its
    /// progress).
    from_block: AtomicU32,
    end_block: AtomicU32,
    /// The row has the highlight (the lower row's button is not lit then).
    selected: AtomicBool,
}

impl RenderRow {
    pub const fn new() -> Self {
        RenderRow {
            from_block: AtomicU32::new(0),
            end_block: AtomicU32::new(0),
            selected: AtomicBool::new(false),
        }
    }
}

pub fn selected(race: &Race) -> bool {
    race.replay.render_row.selected.load(Relaxed)
}

pub fn select(race: &Race, on: bool) {
    race.replay.render_row.selected.store(on, Relaxed);
}

/// The viewer's number of the key table's down, up and left arrows, and Enter.
pub const KEY_UP: u8 = 0x66;
pub const KEY_DOWN: u8 = 0x6B;
pub const KEY_LEFT: u8 = 0x68;
pub const KEY_ENTER: u8 = 0x1C;

/// The lower row's buttons (the down arrow from them reaches the row).
pub(super) const LOWER_ROW: std::ops::RangeInclusive<u32> = 13..=25;

/// The video file's name for a moment: `replay_YYYY-MM-DD_HH-MM-SS.mp4`.
fn file_name(t: &rc2k_core::system::time::LocalTime) -> String {
    format!(
        "replay_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.mp4",
        t.year, t.month, t.day, t.hour, t.minute, t.second
    )
}

/// Whether this build offers rendering a replay to a video: it starts a child process of the
/// program and `ffmpeg`, which phones, the web build and the Vita cannot do, so the viewer shows no
/// render button there.
pub(super) const OFFERED: bool = cfg!(not(any(
    target_os = "ios",
    target_os = "android",
    web,
    target_os = "vita"
)));

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
    if !OFFERED {
        return Ok(());
    }
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
        selected(race),
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

/// How far the part being rendered has been played, 0 ..= 1, for the render's bar.
pub fn fraction(race: &Race) -> f64 {
    rc2k_core::race::render_dialog::progress(
        race.replay.replay_view().replay_block.get(),
        race.replay.render_row.from_block.load(Relaxed),
        race.replay.render_row.end_block.load(Relaxed),
    )
}

/// Plays the part of the replay from `from` to `to` (hundredths of a second from its start; none:
/// to its end), as the render of a program started for it does. The part ends at the whole second
/// after `to`.
pub(super) fn play_range(
    race: &'static Race,
    from: u32,
    to: Option<u32>,
    o: Owners<'static>,
) -> Result<(), Refused> {
    let blocks = race.shared.replay_file().index.block_count.get();
    let start = (from / 100).min(blocks.saturating_sub(1));
    let end = to
        .map_or(blocks, |t| t.div_ceil(100).min(blocks))
        .max(start + 1)
        .min(blocks);
    race.replay.render_row.from_block.store(start, Relaxed);
    race.replay.render_row.end_block.store(end, Relaxed);
    let view = race.replay.replay_view();
    view.replay_block.set(start);
    view.replay_frame
        .set(if start == from / 100 { from % 100 } else { 0 });
    crate::race::play_marked::play_tail(
        race,
        o,
        rc2k_core::race::viewer_flow::Play {
            from: start,
            frame: view.replay_frame.get(),
            end,
        },
    )
}

/// The render dialog's answer: the part of the replay chosen is rendered by a program of its own
/// (this one, started with `--render-replay` on a copy of the replay as it is viewed), which
/// draws the race at the size chosen. The viewer waits behind the progress panel
/// (`race::render_progress`) and is as it was when that ends; cancelling it removes what was
/// written. Cancelling the dialog changes nothing.
pub fn render(race: &'static Race, o: Owners<'static>) -> Result<(), Refused> {
    let replay = &race.replay;
    let blocks = race.shared.replay_file().index.block_count.get();
    if blocks == 0 {
        return Ok(());
    }
    let folder = crate::files::host_path(b"savedata\\videos");
    if let Err(e) = std::fs::create_dir_all(&folder) {
        crate::program_log::log(&format!("viewer: {}: {e}", folder.display()));
        return Ok(());
    }
    let name = file_name(&crate::platform::os::local_time());
    let out = folder.join(&name);
    let (mark_in, mark_out) = (
        race.viewer_buffers().mark_in.get(),
        race.viewer_buffers().mark_out.get(),
    );
    let marks = (mark_in, mark_out) != rc2k_core::race::viewer_flow::opened_marks(blocks);
    let target = crate::race::render_dialog::Target {
        folder: "savedata/videos",
        file: &name,
    };
    let Some(chosen) = crate::race::render_dialog::ask(o, marks, &target) else {
        return Ok(());
    };
    let view = replay.replay_view();
    let (from, to) = match chosen.part {
        Part::Whole => (None, None),
        Part::FromHere => (
            Some(view.replay_block.get() * 100 + view.replay_frame.get()),
            None,
        ),
        Part::Marked => (Some(mark_in * 100), Some(mark_out * 100)),
    };
    let stem = name.trim_end_matches(".mp4");
    let temp_name = format!("{stem}.replay.tmp");
    let saved = crate::race::save_viewed_replay::save_to_file(
        race,
        o.system,
        format!("savedata\\videos\\{temp_name}").as_bytes(),
    )?;
    if !saved {
        crate::race::dialogs::notice(
            rc2k_core::strings::own::id::RENDER_FAILED,
            "the replay could not be copied for the render",
        );
        return Ok(());
    }
    let screen = o.system.session.screen.get();
    let size = chosen
        .size
        .unwrap_or((screen.width.get(), screen.height.get()));
    let job = crate::race::render_progress::Render {
        replay: folder.join(temp_name),
        out,
        size,
        fps: chosen.fps,
        quality: chosen.quality,
        encoder: chosen.encoder,
        sound: chosen.sound,
        from,
        to,
    };
    crate::race::render_progress::run(o, &job);
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
