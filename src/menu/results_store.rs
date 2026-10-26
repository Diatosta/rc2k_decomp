//! The players' results figures, out of RAL.EXE's image (IDA 0x758714 .. 0x7587E8; phase 7, typed): the
//! rounds of a multi-player event, the duel's order, and the four players' places, points and totals
//! (`menu::results_setups`, `menu::arcade_steps`). A `#[repr(C)]` struct with the image's layout, one
//! static; `moved::MOVED` points the range at it. The captions' tables hold pointers to its arrays
//! (`CaptionTables::place_pointers`). (Split 2026-10-25 from the front end's block `menu::store`.)

use crate::session::{Bytes, Dword, with_runs};

/// The players' results figures (IDA 0x758714 .. 0x7587E8).
#[repr(C)]
pub struct Results {
    /// `g_round`: The rounds of a multi-player event.
    pub round: Dword,
    /// `dword_758718`: The stage groups' title string ids (`screens::draw_time_trial_stage_info`).
    pub group_titles: [Dword; 2],
    /// `g_duelFirst, g_duelSecond, dword_758728, dword_75872C`: The players in the order of their places (the offsets of their figures: 0, 4, 8, 12).
    pub order: [Dword; 4],
    /// Unreferenced bytes (IDA 0x758730 .. 0x758758).
    pub _gap_758730: Bytes<40>,
    /// `dword_758758`: Each player's time bonus (the stage record's time less the player's, 0 when greater).
    pub time_bonus: [Dword; 4],
    /// `dword_758768`: Each player's place, 1 .. 4.
    pub positions: [Dword; 4],
    /// `dword_758778`: The points of each player's place.
    pub points: [Dword; 4],
    /// `dword_758788`: Each player's +0x94 (the arcade's extra points).
    pub extra: [Dword; 4],
    /// `dword_758798`: The sum of points, bonus and extra.
    pub sums: [Dword; 4],
    /// `dword_7587A8`: Each player's total before the stage.
    pub before: [Dword; 4],
    /// `dword_7587B8`: Each player's total.
    pub totals: [Dword; 4],
    /// `dword_7587C8`: Four (offset, value) pairs the ranking sorts.
    pub pairs: [Dword; 8],
}

/// First values the image's (`tests::result_figures_starts_as_the_games`).
pub static RESULT_FIGURES: Results = Results {
    round: Dword::new(0x0),
    group_titles: [Dword::new(0x340), Dword::new(0x341)],
    order: [const { Dword::new(0x0) }; 4],
    _gap_758730: Bytes::new(with_runs::<40>(&[(0x14, &[0xf5, 0x11])])),
    time_bonus: [const { Dword::new(0x0) }; 4],
    positions: [const { Dword::new(0x0) }; 4],
    points: [const { Dword::new(0x0) }; 4],
    extra: [const { Dword::new(0x0) }; 4],
    sums: [const { Dword::new(0x0) }; 4],
    before: [const { Dword::new(0x0) }; 4],
    totals: [const { Dword::new(0x0) }; 4],
    pairs: [const { Dword::new(0x0) }; 8],
};

/// IDA 0x758714, the first byte of the piece.
pub const START: usize = 0x758714;
/// IDA 0x7587E8, the end of the piece.
pub const END: usize = 0x7587E8;

/// Where IDA 0x758714 is now (`moved::MOVED`'s home).
pub fn results_addr() -> usize {
    crate::pointers::host_number(&RESULT_FIGURES as *const Results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn result_figures_starts_as_the_games() {
        assert!(
            END - START <= size_of::<Results>()
                && size_of::<Results>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(results_addr()) as *const u8,
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(Results, round), 0x758714);
        at(offset_of!(Results, group_titles), 0x758718);
        at(offset_of!(Results, order), 0x758720);
        at(offset_of!(Results, time_bonus), 0x758758);
        at(offset_of!(Results, positions), 0x758768);
        at(offset_of!(Results, points), 0x758778);
        at(offset_of!(Results, extra), 0x758788);
        at(offset_of!(Results, sums), 0x758798);
        at(offset_of!(Results, before), 0x7587A8);
        at(offset_of!(Results, totals), 0x7587B8);
        at(offset_of!(Results, pairs), 0x7587C8);
    }
}
