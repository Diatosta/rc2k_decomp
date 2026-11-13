//! The pieces of one box of a page placed in it: labels in a column as wide as the widest,
//! choices after their label and each other 10 pixels apart, wrapped where they would pass the
//! box's width, the rest centred (or at 0 / the right edge by the piece's kind bits); then the
//! whole centred across and down in the box (by an even number of pixels down in the title's
//! box). Called by `install::place_pieces`.

use crate::errors::Refused;
use crate::install::page_records::{BoxRecord, Piece as LaidPiece};
use rc2k_core::install::layout::{self, Area, Piece};

/// `first` the first piece, `count` the pieces, `the_box` the box.
///
/// `title`: the box is the title's (`InstallerPages.title_box`).
pub fn place_row(
    install: &'static crate::install::owner::Install,
    first: u32,
    count: u32,
    the_box: BoxRecord,
    title: bool,
) -> Result<(), Refused> {
    let at = |k: u32| LaidPiece::nth(install, first.wrapping_add(k));
    let mut pieces: Vec<Piece> = (0..count)
        .map(|k| Piece {
            marks: at(k).marks(),
            kind: at(k).kind(),
            x: at(k).x(),
            y: at(k).y(),
            width: at(k).width(),
            height: at(k).height(),
        })
        .collect();
    let area = Area {
        x: the_box.x(),
        y: the_box.y(),
        width: the_box.width(),
        height: the_box.height(),
    };
    layout::place_row(&mut pieces, area, title)?;
    for (k, p) in (0..).zip(&pieces) {
        at(k).set_x(p.x);
        at(k).set_y(p.y);
    }
    Ok(())
}
