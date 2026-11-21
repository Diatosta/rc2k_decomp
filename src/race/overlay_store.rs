//! The overlay and countdown part of the race block: the stage-bar parts (distances, two colour
//! tables), the countdown's forget flags and the position labels with their kept copies. A plain
//! record the game's thread reads and writes in short borrows (`owner_frame::Frame::overlay`).

/// The countdown's start, in the car's run ticks (`race::frame_draw`, `drive::countdown_beep`).
pub const COUNTDOWN_START: u32 = 0x12C;

/// The sound of each countdown second (`drive::countdown_beep`).
pub const COUNTDOWN_SOUNDS: [u32; 6] = [0xF5, 0xF3, 0xF1, 0xEF, 0xED, 0xF7];

/// The polygon w factor (`scene::draw_polygon`).
pub const W_FACTOR: f32 = 0.01;

pub struct OverlayBlock {
    /// The overlay parts' distances (`race::init_overlay`, `hud::draw_stage_bar`).
    pub overlay_part_miles: [f32; 64],
    /// The parts' colours/kinds, first view (`race::init_overlay`, `race::frame_draw`
    /// colour buffers, `hud::draw_stage_bar`).
    pub overlay_part_kinds_a: [u32; 64],
    /// The same, second view (`race::init_overlay`, `hud::draw_stage_bars`).
    pub overlay_part_kinds_b: [u32; 64],
    /// Each view's forget flag (`race::frame_views`, `race::frame_draw`).
    pub forget_flags: [u32; 2],
    /// A 7-byte text and a spare byte, the label behind (`race::frame_draw`, `race::name_plates`,
    /// `physics::rank_split_time`, `race::race_init`).
    pub label_behind: [u32; 2],
    /// The same, the label ahead.
    pub label_ahead: [u32; 2],
    /// The kept copy of `label_behind` (`race::name_plates`, `race::race_init`).
    pub kept_behind: [u32; 2],
    /// The kept copy of `label_ahead` (same).
    pub kept_ahead: [u32; 2],
}

impl OverlayBlock {
    /// The race overlay at the start: the stage bar's scale, the countdown and the position
    /// labels.
    pub const fn new() -> Self {
        OverlayBlock {
            overlay_part_miles: [0.0; 64],
            overlay_part_kinds_a: [0; 64],
            overlay_part_kinds_b: [0; 64],
            forget_flags: [0; 2],
            label_behind: plain_dwords_of_text_u32(b"car 64  "),
            label_ahead: plain_dwords_of_text_u32(b"car 32  "),
            kept_behind: plain_dwords_of_text_u32(b"car 64  "),
            kept_ahead: plain_dwords_of_text_u32(b"car 32  "),
        }
    }
}

/// The two dwords of an eight-byte text.
const fn plain_dwords_of_text_u32(text: &[u8; 8]) -> [u32; 2] {
    [
        u32::from_le_bytes([text[0], text[1], text[2], text[3]]),
        u32::from_le_bytes([text[4], text[5], text[6], text[7]]),
    ]
}
