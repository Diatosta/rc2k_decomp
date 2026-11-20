//! A page's pieces placed, box by box: the pieces in a row whose items share a box go to
//! `install::place_row` together. Refused: no pieces.

use crate::errors::Refused;
use crate::install::lay_out_pieces::{Piece, item_of};
use crate::install::place_row::place_row;
use rc2k_core::install::model::PageDef;

pub(super) fn place_pieces(def: &PageDef, pieces: &mut [Piece]) -> Result<(), Refused> {
    let mut boxes = Vec::with_capacity(pieces.len());
    for piece in pieces.iter() {
        // A box's number, 0 for the page's own.
        boxes.push(
            item_of(def, piece)?
                .box_id
                .map_or(0, |id| u32::from(id) + 1),
        );
    }
    let rows = rc2k_core::install::layout::rows(pieces.len() as u32, |k| boxes[k as usize])?;
    for (first, n, the_box) in rows {
        let id = the_box.checked_sub(1).map(|id| id as u8);
        place_row(pieces, first, n, id)?;
    }
    Ok(())
}
