//! The screens' text buffers, typed: the line buffer the screens build their lines in, the
//! cells, the stage line, the caption, the title and the number text (0x100 bytes each), the stage
//! result's value and the ten page-number texts.

use crate::session::{GameThread, PlainBytes, PlainDword};

/// The screens' text buffers.
pub static TEXT_BUFFERS: GameThread<TextBuffers> = GameThread::new(TextBuffers::new());

/// The screens' text buffers, for the game's thread.
pub fn text_buffers() -> &'static TextBuffers {
    TEXT_BUFFERS.get()
}

#[derive(rc2k_derive::Cells)]
pub struct TextBuffers {
    /// +0x0: the line the draw lists build (`screens::line_buffer`, `screens::draw_menu_items`).
    pub line: PlainBytes<0x100>,
    /// +0x100: the cells of the spaced numbers, the bindings' and keys' texts
    /// (`screens::draw_spaced_number`, `screens::game_screen`).
    pub cells: PlainBytes<0x100>,
    /// +0x200: the stage page's second heading (`screens::page_settings`, `screens::stage_panel`,
    /// `screens::stage_results`).
    pub stage_heading: PlainBytes<0x100>,
    /// +0x300: the caption text (`screens::car_caption`, `screens::best_scores`,
    /// `screens::yes_no_caption`, `menu::player_actions`, ...).
    pub caption: PlainBytes<0x100>,
    /// +0x400: the car page's title, the gearing text, the slideshow's title
    /// (`screens::car_page_title`, `screens::gearing_text`, `screens::slideshow_title`).
    pub title: PlainBytes<0x100>,
    /// +0x500: the player's number text; the installer's patch text starts here
    /// (`screens::player_number_text`, `install::check_at_start`).
    pub number_text: PlainBytes<0x100>,
    /// +0x600: the stage record's value (`screens::stage_results`).
    pub stage_result_value: PlainDword,
    /// +0x604: ten texts of four bytes, the page numbers (`screens::savegame_page_numbers`).
    pub page_numbers: PlainBytes<40>,
}

impl TextBuffers {
    /// The bytes, zero as in the image.
    pub const fn new() -> Self {
        crate::cells::zeroed_cells()
    }
}
