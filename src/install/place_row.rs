//! The pieces of one box of a page placed in it: labels in a column as wide as the widest, choices
//! after their label and each other 10 pixels apart, wrapped where they would pass the box's width,
//! the rest centred (or at 0 / the right edge by the piece's kind bits); then the whole centred
//! across and down in the box (by an even number of pixels down in the title's box).

use crate::errors::{BadInput, Refused};
use crate::install::lay_out_pieces::Piece;
use rc2k_core::install::layout;
use rc2k_core::install::model::{BoxId, box_area};

/// `first` the first piece, `count` the pieces, `the_box` the box.
pub fn place_row(
    pieces: &mut [Piece],
    first: u32,
    count: u32,
    the_box: BoxId,
) -> Result<(), Refused> {
    let range = first as usize..first as usize + count as usize;
    let Some(row) = pieces.get_mut(range) else {
        return Err(BadInput("a row of pieces past the page's pieces").into());
    };
    let (area, title) = box_area(the_box);
    layout::place_row(row, area, title)?;
    Ok(())
}
