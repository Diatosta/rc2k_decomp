//! The overlay and countdown part of the race block: the stage-bar parts (distances, two colour tables), the start lights, the countdown sounds and the position labels with their kept copies. Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::race::first_cells::{plain_dwords, plain_dwords_of_text};
use crate::session::{PlainDword, PlainFloat};

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct OverlayBlock {
    /// +0x0 (0x6D150C): `g_overlayPartMiles`: the overlay parts' distances (`race::init_overlay`, `hud::draw_stage_bar`).
    pub overlay_part_miles: [PlainFloat; 64],
    /// +0x100 (0x6D160C): `dword_6D160C`: the parts' colours/kinds, first view (`race::init_overlay`, `race::frame_draw` COLOUR_BUFFERS, `hud::draw_stage_bar`).
    pub overlay_part_kinds_a: [PlainDword; 64],
    /// +0x200 (0x6D170C): `dword_6D170C`: the same, second view (`race::init_overlay`, `hud::draw_stage_bars`).
    pub overlay_part_kinds_b: [PlainDword; 64],
    /// +0x300 (0x6D180C): `flt_6D180C`: the polygon w factor (`scene::draw_polygon`).
    pub w_factor: PlainFloat,
    /// +0x304 (0x6D1810): `g_startLightsLit`: the lights lit, 0 .. 5 (`race::frame_draw`, `race::stage_loading`).
    pub start_lights_lit: PlainDword,
    /// +0x308 (0x6D1814): `g_countdownStart`: the countdown's start (`race::frame_draw`, `drive::countdown_beep`).
    pub countdown_start: PlainDword,
    /// +0x30c (0x6D1818): `dword_6D1818`: the sound of each countdown second (`drive::countdown_beep`).
    pub countdown_sounds: [PlainDword; 6],
    /// +0x324 (0x6D1830): `dword_6D1830`: view 0 forget flag (`race::frame_views`, `race::frame_draw`).
    pub forget_flag_0: PlainDword,
    /// +0x328 (0x6D1834): `dword_6D1834`: view 1 forget flag (`race::frame_views`, `race::frame_draw`).
    pub forget_flag_1: PlainDword,
    /// +0x32c (0x6D1838): `dword_6D1838` + `a64`: a 7-byte text, the label behind (`race::frame_draw`, `race::name_plates`, `physics::rank_split_time`, `race::race_init`).
    pub label_behind: [PlainDword; 2],
    /// +0x334 (0x6D1840): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_334: PlainDword,
    /// +0x338 (0x6D1844): `dword_6D1844` + `a32`: a 7-byte text, the label ahead (same modules).
    pub label_ahead: [PlainDword; 2],
    /// +0x340 (0x6D184C): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_340: PlainDword,
    /// +0x344 (0x6D1850): `dword_6D1850` + `a64_0`: the kept copy of label_behind (`race::name_plates`, `race::race_init`).
    pub kept_behind: [PlainDword; 2],
    /// +0x34c (0x6D1858): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_34c: PlainDword,
    /// +0x350 (0x6D185C): `dword_6D185C` + `a32_0`: the kept copy of label_ahead (same).
    pub kept_ahead: [PlainDword; 2],
    /// +0x358 (0x6D1864): no code reads or writes it (the field's name nowhere in Rust, and no IDA xref to any of its bytes, 2026-11-10); it only keeps the image's layout.
    pub unused_358: [PlainDword; 3],
}

impl OverlayBlock {
    /// The race overlay as the game starts: the stage bar's scale, the countdown and the position labels.
    pub const fn new() -> Self {
        OverlayBlock {
            w_factor: PlainFloat::new(0.01),
            countdown_start: PlainDword::new(0x12C),
            countdown_sounds: plain_dwords([0xF5, 0xF3, 0xF1, 0xEF, 0xED, 0xF7]),
            label_behind: plain_dwords_of_text::<2>(b"car 64  "),
            label_ahead: plain_dwords_of_text::<2>(b"car 32  "),
            kept_behind: plain_dwords_of_text::<2>(b"car 64  "),
            kept_ahead: plain_dwords_of_text::<2>(b"car 32  "),
            ..ZeroCells::ZERO
        }
    }
}
