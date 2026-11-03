//! `install::measure_pieces` (0x414DF5): each of a page's pieces measured into its width
//! (+0x10) and height (+0x14). A piece whose item has 0x20 first has bits 0, 1 of its kind
//! moved to bits 2, 3. A piece of a plain text kind (no 0x10) is its string measured
//! (`text::measure_string_by_id`) in the font of its marks and kind (`byte_51D52D`,
//! `dword_51D4C4`); else by the item's kind: an edit its record's width and its font's
//! height (`byte_51D52F`); a text its string and the item's appended text (`[item+0xC]`
//! with eax `[item+8]`, if any) in `text`, measured (`text::measure_text`); an action its
//! box's size; any other what ecx and edx hold.
//!
//! In:  nothing (the pieces at `dword_51C407`)
//! Out: every register kept (`pusha` / `popa`); no flags read
//! No pieces would go round 2^32 times: refused. Caller `install::run_page`.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{
    append_byte, append_string_by_id, measure_string, measure_text, rb, rd, rw, text_action, wb, wd,
};
use crate::install::lay_out_pieces::PIECE;
use crate::install::vars::INSTALLER_PAGES;
use crate::strings::text_buffer::TEXT_BUFFER;
use rc2k_core::install::pieces::{self, Way};

fn font_of(index: u32) -> Result<u32, BadInput> {
    crate::install::vars::InstallerPages::font_of(index)
}

pub fn measure_pieces() -> Result<(), Refused> {
    let count = INSTALLER_PAGES.layout.get_u32(0);
    if count == 0 {
        return Err(BadInput("a page of no pieces").into());
    }
    // The size each piece gets is what ecx and edx hold: ecx the pieces left (the measures
    // get it as their x), edx the last height (a piece measured with neither keeps it).
    let mut height = None;
    for k in 0..count {
        let left = count - k;
        let piece = crate::place::Place::from(&INSTALLER_PAGES.layout).add(4 + PIECE * k);
        // Converted: `rc2k_core::install::pieces` (`measured_kind`, `Way`,
        // `measure_colour_at`).
        let marks = rb(piece);
        if marks & 0x20 != 0 {
            wb(piece.add(1), pieces::measured_kind(marks, rb(piece.add(1))));
        }
        let (width, high) = match Way::of(marks, rb(piece.add(1))) {
            Way::String { font_kind } => {
                let font = rb(crate::place::Place::from(&INSTALLER_PAGES.kind_fonts)
                    .add(pieces::measure_colour_at(font_kind))) as u32;
                measure_string(rw(piece.add(2)) as u32, font_of(font)?, left)?
            }
            Way::Edit => {
                let item = rd(piece.add(4));
                let record = rd(item + 0xC);
                let font_height = rb(crate::place::Place::from(&INSTALLER_PAGES.kind_fonts)
                    .add(7 + 8 * rb(record + 4) as u32)) as u32;
                let width = crate::install::folders::field_width(rd(item + 8), rw(record + 6));
                (width as u32, font_height)
            }
            Way::Text => {
                let item = rd(piece.add(4));
                let mut cursor = append_string_by_id(rw(piece.add(2)) as u32, &TEXT_BUFFER.text);
                let action = rd(item + 0xC);
                if action != 0 {
                    cursor = text_action(action, cursor)?;
                }
                append_byte(cursor, 0);
                let kind = rb(item + 1) as u32;
                let font = rb(crate::place::Place::from(&INSTALLER_PAGES.kind_fonts)
                    .add(pieces::measure_colour_at(kind))) as u32;
                measure_text(font_of(font)?, &TEXT_BUFFER.text, left)?
            }
            Way::Action => {
                let the_box = rd(rd(piece.add(4)) + 4);
                (rd(the_box + 8), rd(the_box + 0xC))
            }
            Way::Other => {
                let Some(height) = height else {
                    return Err(BadInput(
                        "a first piece measured with nothing (the original's entry edx)",
                    )
                    .into());
                };
                (left, height)
            }
        };
        height = Some(high);
        wd(piece.add(0x10), width);
        wd(piece.add(0x14), high);
    }
    Ok(())
}
