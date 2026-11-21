//! The screens' text buffers, typed: the line buffer the screens build their lines in, the
//! cells, the stage line, the caption, the title and the number text (0x100 bytes each), the stage
//! result's value and the ten page-number texts.

use crate::memory::plain_buffer::PlainBuffer;
use crate::memory::session::PlainDword;

pub struct TextBuffers {
    /// The line the draw lists build (`screens::line_buffer`, `screens::draw_menu_items`).
    pub line: PlainBuffer<0x100>,
    /// The cells of the spaced numbers, the bindings' and keys' texts
    /// (`screens::draw_spaced_number`, `screens::game_screen`).
    pub cells: PlainBuffer<0x100>,
    /// The stage page's second heading (`screens::page_settings`, `screens::stage_panel`,
    /// `screens::stage_results`).
    pub stage_heading: PlainBuffer<0x100>,
    /// The caption text (`screens::car_caption`, `screens::best_scores`,
    /// `screens::yes_no_caption`, `menu::player_actions`, ...).
    pub caption: PlainBuffer<0x100>,
    /// The car page's title, the gearing text, the slideshow's title
    /// (`screens::car_page_title`, `screens::gearing_text`, `screens::slideshow_title`).
    pub title: PlainBuffer<0x100>,
    /// The player's number text; the installer's patch text starts here
    /// (`screens::player_number_text`, `install::check_at_start`).
    pub number_text: PlainBuffer<0x100>,
    /// The stage record's value (`screens::stage_results`).
    pub stage_result_value: PlainDword,
    /// Ten texts of four bytes, the page numbers (`screens::savegame_page_numbers`).
    pub page_numbers: PlainBuffer<40>,
}

impl TextBuffers {
    /// The bytes, zero at the start.
    pub const fn new() -> Self {
        Self {
            line: PlainBuffer::new([0; 0x100]),
            cells: PlainBuffer::new([0; 0x100]),
            stage_heading: PlainBuffer::new([0; 0x100]),
            caption: PlainBuffer::new([0; 0x100]),
            title: PlainBuffer::new([0; 0x100]),
            number_text: PlainBuffer::new([0; 0x100]),
            stage_result_value: PlainDword::new(0),
            page_numbers: PlainBuffer::new([0; 40]),
        }
    }
}
