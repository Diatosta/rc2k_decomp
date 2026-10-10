//! What the two panels of the load-game screen's preview share: the three column headings and
//! the row numbers 1 .. 4, and where things sit. The preview shows the saved game under the
//! highlight: a title line and four rows, one per car.

use crate::errors::Refused;
use crate::screens::stage_panel::label;
use crate::screens::{font_at, line_buffer, set_line};
use crate::strings::append_u32::append_u32;
use crate::text::draw_text::{Canvas, draw_text_at};

pub const FONT: usize = 0x51E584;
pub const TITLE_FONT: usize = 0x51E5E4;
const ROW_NUMBER_FONT: usize = 0x51E5A4;

const HEADINGS: [(u32, i32); 3] = [(0x4E5, 0x8F), (0x4E6, 0x1D0), (0x4E7, 0x271)];
const HEADINGS_Y: i32 = 0x17A;
pub const TITLE_Y: i32 = 0x161;
pub const LEFT_X: i32 = 0x8F;
/// The four rows.
pub const ROWS: [i32; 4] = [0x193, 0x1AC, 0x1C5, 0x1DE];
/// The four number columns of a row.
pub const NUMBER_COLUMNS: [i32; 4] = [0x1D0, 0x215, 0x271, 0x2B6];
pub const NAME_X: i32 = 0xD6;
pub const FLAG_X: i32 = 0xAF;
pub const CAR_X: i32 = 0x18B;
/// Where "nothing" is written in place of the two sprites.
pub const NO_FLAG_X: i32 = 0xB8;
pub const NO_CAR_X: i32 = 0x1A1;

pub const NOTHING: u32 = 0x57A; // the string shown where there is no value
pub const NO_ENTRY: u32 = 0x578; // the string shown where there is no name

/// The fonts and strings both panels need.
pub const FONTS: [usize; 3] = [FONT, TITLE_FONT, ROW_NUMBER_FONT];
pub const STRINGS: [u32; 5] = [0x4E5, 0x4E6, 0x4E7, NOTHING, NO_ENTRY];

/// The headings and the row numbers, which both panels start with.
pub unsafe fn draw_frame(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        for (id, x) in HEADINGS {
            label(canvas, id, FONT, x, HEADINGS_Y)?;
        }
        for (n, y) in ROWS.into_iter().enumerate() {
            let mut line = append_u32(n as u32 + 1);
            line.push(0);
            set_line(&line);
            draw_text_at(font_at(ROW_NUMBER_FONT), line_buffer(), LEFT_X, y, canvas)?;
        }
    }
    Ok(())
}
