//! The players' results figures (IDA 0x758714 .. 0x7587E8): the
//! rounds of a multi-player event, the duel's order, and the four players' places, points and totals
//! (`menu::results_setups`, `menu::arcade_steps`). A `#[repr(C)]` struct, one static.

use crate::session::PlainDword;

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

impl Results {
    pub const fn new() -> Self {
        Results {
            round: PlainDword::new(0x0),
            group_titles: [PlainDword::new(0x340), PlainDword::new(0x341)],
            order: [const { PlainDword::new(0x0) }; 4],
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
