//! The replay viewer's "render to video" row: its state (whether it has the highlight, the blocks
//! a render plays), the playing of a part of the replay, and the dialog's choices kept between
//! sessions. The row is drawn and driven by `menu::replay_viewer::render_row`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use std::sync::Mutex;
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

/// The render dialog's choices as last made (and read from the options file).
static KEPT: Mutex<Option<rc2k_core::options::model::VideoChoices>> = Mutex::new(None);

/// What the options file keeps of the dialog.
pub fn kept() -> rc2k_core::options::model::VideoChoices {
    KEPT.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
        .unwrap_or_default()
}

/// The choices the options file held.
pub fn set_kept(choices: rc2k_core::options::model::VideoChoices) {
    *KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(choices);
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
pub fn play_range(
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
