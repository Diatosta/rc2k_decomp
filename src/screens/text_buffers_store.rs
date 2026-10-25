//! The screens' text buffers, typed (phase 7): IDA 0xA8BB54 .. 0xA8C180, in the image's
//! zeroed tail (`race::store::buffers`): the line buffer the screens build their lines in,
//! the cells, the stage line, the caption, the title and the number text (0x100 bytes each),
//! the stage result's value and the ten page-number texts. Names are readings unless IDA's.

use crate::session::{Bytes, Dword};

#[repr(C)]
pub struct TextBuffers {
    /// +0x0 (0xA8BB54): `g_lineBuffer`: the line the draw lists build (`screens::line_buffer`, `screens::draw_menu_items`).
    pub line: Bytes<0x100>,
    /// +0x100 (0xA8BC54): `byte_A8BC54`: the cells of the spaced numbers, the bindings' and keys' texts (`screens::draw_spaced_number`, `screens::game_screen`).
    pub cells: Bytes<0x100>,
    /// +0x200 (0xA8BD54): `byte_A8BD54`: the stage page's second heading (`screens::page_settings`, `screens::stage_panel`, `screens::stage_results`).
    pub stage_heading: Bytes<0x100>,
    /// +0x300 (0xA8BE54): `g_captionText` (`screens::car_caption`, `screens::best_scores`, `screens::yes_no_caption`, `menu::player_actions`, ...).
    pub caption: Bytes<0x100>,
    /// +0x400 (0xA8BF54): `byte_A8BF54`: the car page's title, the gearing text, the slideshow's title (`screens::car_page_title`, `screens::gearing_text`, `screens::slideshow_title`).
    pub title: Bytes<0x100>,
    /// +0x500 (0xA8C054): `byte_A8C054`: the player's number text; the installer's patch text starts here (`screens::player_number_text`, `install::check_at_start`).
    pub number_text: Bytes<0x100>,
    /// +0x600 (0xA8C154): `dword_A8C154`: the stage record's +4 (`screens::stage_results`).
    pub stage_result_value: Dword,
    /// +0x604 (0xA8C158): `byte_A8C158`: ten texts of four bytes, the page numbers (`screens::savegame_page_numbers`).
    pub page_numbers: Bytes<40>,
}

pub struct InBuffers;

impl core::ops::Deref for InBuffers {
    type Target = TextBuffers;
    fn deref(&self) -> &TextBuffers {
        // SAFETY: the block lives as long as the program and the piece is aligned to 4 (the
        // test checks); every field is a cell.
        unsafe { &*(piece_addr() as *const TextBuffers) }
    }
}

pub static TEXT_BUFFERS: InBuffers = InBuffers;

/// IDA 0xA8BB54, the first byte of the piece.
pub const START: usize = 0xA8BB54;
/// IDA 0xA8C180, the end of the piece.
pub const END: usize = 0xA8C180;

const _: () = assert!(END - START == core::mem::size_of::<TextBuffers>());

/// Where IDA 0xA8BB54 is now.
pub fn piece_addr() -> usize {
    crate::moved::place(START)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_buffers() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            piece_addr() + (END - START) - 1
        );
        assert_eq!(offset_of!(TextBuffers, cells), 0xA8BC54 - START);
        assert_eq!(offset_of!(TextBuffers, stage_heading), 0xA8BD54 - START);
        assert_eq!(offset_of!(TextBuffers, caption), 0xA8BE54 - START);
        assert_eq!(offset_of!(TextBuffers, title), 0xA8BF54 - START);
        assert_eq!(offset_of!(TextBuffers, number_text), 0xA8C054 - START);
        assert_eq!(
            offset_of!(TextBuffers, stage_result_value),
            0xA8C154 - START
        );
        assert_eq!(offset_of!(TextBuffers, page_numbers), 0xA8C158 - START);
    }
}
