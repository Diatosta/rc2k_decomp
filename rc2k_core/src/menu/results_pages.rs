//! The results and standings pages: the first player's figure each page holds, and the
//! order of the tables' sorts before them (from
//! `menu::show_overall_stage_results`, `show_overall_rally_results` 0x4A3BA2,
//! `show_championship_standings` 0x4A5A47, `show_class_stage_results`,
//! `show_class_rally_results`, `show_class_championship_standings`,
//! `menu::rally_results` 0x4A94EC and `championship_standings` 0x4A9583).

/// The A8 championship's `g_eventMode`.
pub const A8: u32 = 0x13;

/// The most a page's figure (`dword_7556B4`) may be (signed).
pub const OVERALL_MOST: u32 = 0xAB;
pub const CLASS_MOST: u32 = 0x33;
pub const A8_CLASS_MOST: u32 = 0x51;

/// The results pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    OverallStage,
    /// Held to 0xAB, but a figure above it becomes 0x51 (the original's slip, bugs.md).
    OverallRally,
    ChampionshipStandings,
    ClassStage,
    /// The A8 championship's (`g_eventMode` 0x13) held to 0x51, the others' to 0x33.
    ClassRally {
        a8: bool,
    },
    ClassChampionship,
}

/// The figure a page holds (`figure` the first player's, signed).
pub fn held_figure(page: Page, figure: u32) -> u32 {
    let (most, above) = match page {
        Page::OverallStage | Page::ChampionshipStandings => (OVERALL_MOST, OVERALL_MOST),
        Page::OverallRally => (OVERALL_MOST, A8_CLASS_MOST),
        Page::ClassStage | Page::ClassChampionship | Page::ClassRally { a8: false } => {
            (CLASS_MOST, CLASS_MOST)
        }
        Page::ClassRally { a8: true } => (A8_CLASS_MOST, A8_CLASS_MOST),
    };
    if figure as i32 > most as i32 {
        above
    } else {
        figure
    }
}

/// `rally_results` and `championship_standings`: the A8 championship shows the class
/// pages only.
pub fn class_pages_only(event_mode: u32) -> bool {
    event_mode == A8
}

/// A sort of standings table B.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    ByFigure,
    ByScore,
    ByPoints,
}

/// `championship_standings`' sorts before its pages: by figure and by score (then the
/// table numbered); in the A8 championship by figure twice and by points.
pub fn standings_sorts(a8: bool) -> &'static [Sort] {
    if a8 {
        &[Sort::ByFigure, Sort::ByFigure, Sort::ByPoints]
    } else {
        &[Sort::ByFigure, Sort::ByScore]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures() {
        assert_eq!(held_figure(Page::OverallStage, 0xAC), 0xAB);
        assert_eq!(held_figure(Page::OverallStage, 0xAB), 0xAB);
        assert_eq!(held_figure(Page::OverallRally, 0xAC), 0x51);
        assert_eq!(held_figure(Page::OverallRally, 0x90), 0x90);
        assert_eq!(held_figure(Page::ClassStage, 0x34), 0x33);
        assert_eq!(held_figure(Page::ClassStage, u32::MAX), u32::MAX);
        assert_eq!(held_figure(Page::ClassRally { a8: true }, 0x40), 0x40);
        assert_eq!(held_figure(Page::ClassRally { a8: false }, 0x40), 0x33);
        assert!(class_pages_only(0x13) && !class_pages_only(2));
        assert_eq!(standings_sorts(false), &[Sort::ByFigure, Sort::ByScore]);
    }
}
