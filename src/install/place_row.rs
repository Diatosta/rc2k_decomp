//! The pieces of one box of a page placed in it: labels in a column as wide as the widest, choices
//! after their label and each other 10 pixels apart, wrapped where they would pass the box's width,
//! the rest centred (or at 0 / the right edge by the piece's kind bits); then the whole centred
//! across and down in the box (by an even number of pixels down in the title's box).

use crate::errors::{BadInput, Refused};
use crate::install::page_records::BoxRecord;
use rc2k_core::install::layout::{self, Area};

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
    let mut laid = install.pieces.borrow_mut();
    let range = first as usize..first as usize + count as usize;
    let Some(pieces) = laid.get_mut(range) else {
        return Err(BadInput("a row of pieces past the page's pieces").into());
    };
    let area = Area {
        x: the_box.x(),
        y: the_box.y(),
        width: the_box.width(),
        height: the_box.height(),
    };
    layout::place_row(pieces, area, title)?;
    Ok(())
}
