//! The players' results figures, out of RAL.EXE's image (IDA 0x758714 .. 0x7587E8): the
//! rounds of a multi-player event, the duel's order, and the four players' places, points and totals
//! (`menu::results_setups`, `menu::arcade_steps`). A `#[repr(C)]` struct with the image's layout, one
//! static; `moved::MOVED` points the range at it. The captions' tables hold pointers to its arrays
//! (`CaptionTables::place_pointers`).

use crate::session::{PlainBytes, PlainDword, with_runs};

/// The players' results figures (IDA 0x758714 .. 0x7587E8).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Results {
    /// `g_round`: The rounds of a multi-player event.
    pub round: PlainDword,
    /// `dword_758718`: The stage groups' title string ids (`screens::draw_time_trial_stage_info`).
    pub group_titles: [PlainDword; 2],
    /// `g_duelFirst, g_duelSecond, Results.order[2], Results.order[3]`: The players in the order of their places (the offsets of their figures: 0, 4, 8, 12).
    pub order: [PlainDword; 4],
    /// Unreferenced bytes (IDA 0x758730 .. 0x758758).
    pub _gap_758730: PlainBytes<40>,
    /// `dword_758758`: Each player's time bonus (the stage record's time less the player's, 0 when greater).
    pub time_bonus: [PlainDword; 4],
    /// `dword_758768`: Each player's place, 1 .. 4.
    pub positions: [PlainDword; 4],
    /// `dword_758778`: The points of each player's place.
    pub points: [PlainDword; 4],
    /// `dword_758788`: Each player's +0x94 (the arcade's extra points).
    pub extra: [PlainDword; 4],
    /// `dword_758798`: The sum of points, bonus and extra.
    pub sums: [PlainDword; 4],
    /// `dword_7587A8`: Each player's total before the stage.
    pub before: [PlainDword; 4],
    /// `dword_7587B8`: Each player's total.
    pub totals: [PlainDword; 4],
    /// `dword_7587C8`: Four (offset, value) pairs the ranking sorts.
    pub pairs: [PlainDword; 8],
}

/// First values the image's (`tests::result_figures_starts_as_the_games`).
impl Results {
    pub const fn new() -> Self {
        Results {
            round: PlainDword::new(0x0),
            group_titles: [PlainDword::new(0x340), PlainDword::new(0x341)],
            order: [const { PlainDword::new(0x0) }; 4],
            _gap_758730: PlainBytes::new(with_runs::<40>(&[(0x14, &[0xf5, 0x11])])),
            time_bonus: [const { PlainDword::new(0x0) }; 4],
            positions: [const { PlainDword::new(0x0) }; 4],
            points: [const { PlainDword::new(0x0) }; 4],
            extra: [const { PlainDword::new(0x0) }; 4],
            sums: [const { PlainDword::new(0x0) }; 4],
            before: [const { PlainDword::new(0x0) }; 4],
            totals: [const { PlainDword::new(0x0) }; 4],
            pairs: [const { PlainDword::new(0x0) }; 8],
        }
    }
}

/// IDA 0x758714, the first byte of the piece.
pub const START: usize = 0x758714;
/// IDA 0x7587E8, the end of the piece.
pub const END: usize = 0x7587E8;

/// Where IDA 0x758714 lives (`moved::MOVED`'s home).
#[cfg(test)]
pub fn results_addr() -> u32 {
    crate::pointers::game_address(&raw const crate::menu::owner::menus().play.results)
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
                crate::pointers::host(results_addr()).cast_const(),
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
