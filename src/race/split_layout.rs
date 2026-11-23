//! The layout of a split screen of three or four players, among the ones `Split::many_layouts`
//! lists: the one chosen in the race (the split key cycles them) and kept between sessions in
//! `rc2k.toml` (`display.split_layout`). The two-player split keeps its orientation in the race's
//! own settings. A replay's group of players is split as its layout byte says instead
//! (`shown_split`).

use crate::race::owner::Race;
use rc2k_core::race::layouts::Shown;
use rc2k_core::race::view::Split;
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

static INDEX: AtomicU32 = AtomicU32::new(0);

/// The layout's index (wrapped round the layouts of the race's player count where it is used).
pub fn index() -> u32 {
    INDEX.load(Relaxed)
}

/// Sets the layout's index.
pub fn set_index(index: u32) {
    INDEX.store(index, Relaxed);
}

/// The layout after the current one for `players` players, set and given back.
pub fn cycle(players: u32) -> u32 {
    let next = Split::next_many(players, index());
    set_index(next);
    next
}

/// The race's split of the screen between `players` players (three or four): the layout chosen.
pub fn chosen(players: u32) -> Option<Split> {
    Split::many(players, index())
}

/// How the screen is split between `views` views (three or four) as it is drawn now: a replay's
/// frame (`replay_frames::draws_replay`: playing, or the viewer's still frame) of a layout of a
/// group of players, in the group's arrangement that the layout names (`layouts::Shown::Group`);
/// else as the race's layout chosen (`chosen`).
pub fn shown_split(race: &Race, views: u32) -> Option<Split> {
    let still = race.frame.frame_work.borrow().replay_still_on != 0;
    if rc2k_core::drive::replay_frames::draws_replay(race.course.replay_mode(), still) {
        let layout = u8::try_from(race.shared.view.get().view_switch.get()).ok();
        if let Some(Shown::Group { split, .. }) =
            layout.and_then(|layout| crate::race::viewer::layout_shown(race, layout))
        {
            return Some(split);
        }
    }
    chosen(views)
}
