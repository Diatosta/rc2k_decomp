//! The name of the highlighted button, shown under the viewer's render button for a while when the
//! highlight moves: what had the highlight when the name was last started, and when. It is drawn
//! by `menu::replay_viewer::tip`; the choice of the text and the timing are
//! `rc2k_core::race::viewer_tip`.

use crate::race::owner::Race;
use rc2k_core::race::viewer_tip::Highlight;
use std::sync::Mutex;
use std::time::Instant;

/// What had the highlight when the name was last started, and when.
struct Shown {
    highlight: Highlight,
    since: Instant,
}

/// The name last started, one per race (`Replay::viewer_tip`).
pub struct ViewerTip(Mutex<Option<Shown>>);

impl ViewerTip {
    pub const fn new() -> Self {
        ViewerTip(Mutex::new(None))
    }
}

/// The viewer opens: the first highlight shown names itself.
pub fn reset(race: &Race) {
    *race
        .replay
        .viewer_tip
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

impl ViewerTip {
    /// How long `highlight` has had the name shown, in milliseconds: the name starts afresh
    /// when the highlight is not the one it was last started for.
    pub fn elapsed_millis(&self, highlight: Highlight) -> u32 {
        let mut shown = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        match shown.as_mut() {
            Some(s) if s.highlight == highlight => {}
            _ => {
                *shown = Some(Shown {
                    highlight,
                    since: now,
                });
            }
        }
        shown
            .as_ref()
            .map_or(0, |s| s.since.elapsed().as_millis() as u32)
    }
}
