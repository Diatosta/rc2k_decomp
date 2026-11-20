//! The screens' text buffers, typed: the line buffer the screens build their lines in, the
//! cells, the stage line, the caption, the title and the number text (0x100 bytes each), the stage
//! result's value and the ten page-number texts.

use crate::memory::session::{PlainBytes, PlainDword};

#[derive(rc2k_derive::Cells)]
pub struct TextBuffers {
    /// The line the draw lists build (`screens::line_buffer`, `screens::draw_menu_items`).
    pub line: PlainBytes<0x100>,
    /// The cells of the spaced numbers, the bindings' and keys' texts
    /// (`screens::draw_spaced_number`, `screens::game_screen`).
    pub cells: PlainBytes<0x100>,
    /// The stage page's second heading (`screens::page_settings`, `screens::stage_panel`,
    /// `screens::stage_results`).
    pub stage_heading: PlainBytes<0x100>,
    /// The caption text (`screens::car_caption`, `screens::best_scores`,
    /// `screens::yes_no_caption`, `menu::player_actions`, ...).
    pub caption: PlainBytes<0x100>,
    /// The car page's title, the gearing text, the slideshow's title
    /// (`screens::car_page_title`, `screens::gearing_text`, `screens::slideshow_title`).
    pub title: PlainBytes<0x100>,
    /// The player's number text; the installer's patch text starts here
    /// (`screens::player_number_text`, `install::check_at_start`).
    pub number_text: PlainBytes<0x100>,
    /// The stage record's value (`screens::stage_results`).
    pub stage_result_value: PlainDword,
    /// Ten texts of four bytes, the page numbers (`screens::savegame_page_numbers`).
    pub page_numbers: PlainBytes<40>,
}

impl TextBuffers {
    /// The bytes, zero as in the image.
    pub const fn new() -> Self {
        crate::memory::cells::zeroed_cells()
    }
}
