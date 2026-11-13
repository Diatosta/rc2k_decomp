//! The screens' text buffers, typed: IDA 0xA8BB54 .. 0xA8C180, in the image's
//! zeroed tail: the line buffer the screens build their lines in,
//! the cells, the stage line, the caption, the title and the number text (0x100 bytes each),
//! the stage result's value and the ten page-number texts. Names are readings unless IDA's.

use crate::session::{PlainBytes, PlainDword};

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct TextBuffers {
    /// +0x0 (0xA8BB54): `g_lineBuffer`: the line the draw lists build (`screens::line_buffer`, `screens::draw_menu_items`).
    pub line: PlainBytes<0x100>,
    /// +0x100 (0xA8BC54): `byte_A8BC54`: the cells of the spaced numbers, the bindings' and keys' texts (`screens::draw_spaced_number`, `screens::game_screen`).
    pub cells: PlainBytes<0x100>,
    /// +0x200 (0xA8BD54): `byte_A8BD54`: the stage page's second heading (`screens::page_settings`, `screens::stage_panel`, `screens::stage_results`).
    pub stage_heading: PlainBytes<0x100>,
    /// +0x300 (0xA8BE54): `g_captionText` (`screens::car_caption`, `screens::best_scores`, `screens::yes_no_caption`, `menu::player_actions`, ...).
    pub caption: PlainBytes<0x100>,
    /// +0x400 (0xA8BF54): `byte_A8BF54`: the car page's title, the gearing text, the slideshow's title (`screens::car_page_title`, `screens::gearing_text`, `screens::slideshow_title`).
    pub title: PlainBytes<0x100>,
    /// +0x500 (0xA8C054): `byte_A8C054`: the player's number text; the installer's patch text starts here (`screens::player_number_text`, `install::check_at_start`).
    pub number_text: PlainBytes<0x100>,
    /// +0x600 (0xA8C154): `dword_A8C154`: the stage record's +4 (`screens::stage_results`).
    pub stage_result_value: PlainDword,
    /// +0x604 (0xA8C158): `byte_A8C158`: ten texts of four bytes, the page numbers (`screens::savegame_page_numbers`).
    pub page_numbers: PlainBytes<40>,
}

impl TextBuffers {
    /// The bytes, zero as in the image.
    pub const fn new() -> Self {
        crate::cells::cells_of_bytes([0u8; core::mem::size_of::<TextBuffers>()])
    }
}
