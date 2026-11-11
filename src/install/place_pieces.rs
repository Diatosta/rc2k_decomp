//! `install::place_pieces` (0x414ED1): a page's pieces placed, box by box: the pieces in a
//! row whose items share a box (the item's +4: `{x, y, width, height}`; none: the title's
//! box `title_box`) go to `install::place_row` together.
//!
//! No pieces would go round 2^32 times: refused. Caller `install::run_page`.

use crate::errors::Refused;
use crate::install::page_links::PageLink;
use crate::install::page_records::{BoxRecord, Piece, piece_count};
use crate::install::place_row::place_row;
use crate::install::vars::InstallerPages;

pub fn place_pieces(install: &'static crate::install::owner::Install) -> Result<(), Refused> {
    // Converted: `rc2k_core::install::layout::rows` (each piece's box read through its item).
    let rows = rc2k_core::install::layout::rows(piece_count(install), |k| {
        Piece::nth(k).item().box_link().raw()
    })?;
    for (first, n, the_box) in rows {
        let link = PageLink::from_raw(the_box);
        let title = link.is_none() || link == title_box_link();
        place_row(first, n, BoxRecord::linked(link), title)?;
    }
    Ok(())
}

/// The link to the title's box.
fn title_box_link() -> PageLink {
    PageLink::in_pages_at(core::mem::offset_of!(InstallerPages, title_box))
}
