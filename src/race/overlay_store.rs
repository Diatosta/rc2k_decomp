//! The overlay and countdown part of the race block: the stage-bar parts (distances, two colour
//! tables), the start lights, the countdown sounds and the position labels with their kept copies.

use crate::memory::cells::ZeroCells;
use crate::memory::first_cells::plain_dwords_of_text;
use crate::memory::session::{PlainDword, PlainFloat};

/// The countdown's start, in the car's run ticks (`race::frame_draw`, `drive::countdown_beep`).
pub const COUNTDOWN_START: u32 = 0x12C;

/// The sound of each countdown second (`drive::countdown_beep`).
pub const COUNTDOWN_SOUNDS: [u32; 6] = [0xF5, 0xF3, 0xF1, 0xEF, 0xED, 0xF7];

/// The polygon w factor (`scene::draw_polygon`).
pub const W_FACTOR: f32 = 0.01;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct OverlayBlock {
    /// The overlay parts' distances (`race::init_overlay`, `hud::draw_stage_bar`).
    pub overlay_part_miles: [PlainFloat; 64],
    /// The parts' colours/kinds, first view (`race::init_overlay`, `race::frame_draw`
    /// COLOUR_BUFFERS, `hud::draw_stage_bar`).
    pub overlay_part_kinds_a: [PlainDword; 64],
    /// The same, second view (`race::init_overlay`, `hud::draw_stage_bars`).
    pub overlay_part_kinds_b: [PlainDword; 64],
    /// The lights lit, 0 .. 5 (`race::frame_draw`, `race::stage_loading`).
    pub start_lights_lit: PlainDword,
    /// View 0 forget flag (`race::frame_views`, `race::frame_draw`).
    pub forget_flag_0: PlainDword,
    /// View 1 forget flag (`race::frame_views`, `race::frame_draw`).
    pub forget_flag_1: PlainDword,
    /// A 7-byte text, the label behind (`race::frame_draw`, `race::name_plates`,
    /// `physics::rank_split_time`, `race::race_init`).
    pub label_behind: [PlainDword; 2],
    /// Unused; it only keeps the layout.
    pub reserved_334: PlainDword,
    /// A 7-byte text, the label ahead (same modules).
    pub label_ahead: [PlainDword; 2],
    /// Unused; it only keeps the layout.
    pub reserved_340: PlainDword,
    /// The kept copy of `label_behind` (`race::name_plates`, `race::race_init`).
    pub kept_behind: [PlainDword; 2],
    /// The kept copy of `label_ahead` (same).
    pub kept_ahead: [PlainDword; 2],
}

impl OverlayBlock {
    /// The race overlay at the start: the stage bar's scale, the countdown and the position
    /// labels.
    pub const fn new() -> Self {
        OverlayBlock {
            label_behind: plain_dwords_of_text::<2>(b"car 64  "),
            label_ahead: plain_dwords_of_text::<2>(b"car 32  "),
            kept_behind: plain_dwords_of_text::<2>(b"car 64  "),
            kept_ahead: plain_dwords_of_text::<2>(b"car 32  "),
            ..ZeroCells::ZERO
        }
    }
}
