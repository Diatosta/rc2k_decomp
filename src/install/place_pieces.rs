//! A page's pieces placed, box by box: the pieces in a row whose items share a box go to
//! `install::place_row` together. Refused: no pieces. Called by `install::run_page`.

use crate::errors::Refused;
use crate::install::page_links::PageLink;
use crate::install::page_records::{BoxRecord, Piece, piece_count};
use crate::install::place_row::place_row;

pub fn place_pieces(install: &'static crate::install::owner::Install) -> Result<(), Refused> {
    let rows = rc2k_core::install::layout::rows(piece_count(install), |k| {
        Piece::nth(install, k).item().box_link().raw()
    })?;
    for (first, n, the_box) in rows {
        let link = PageLink::from_raw(the_box);
        let title = link.is_none() || link == title_box_link();
        place_row(install, first, n, BoxRecord::linked(link), title)?;
    }
    Ok(())
}

fn title_box_link() -> PageLink {
    PageLink::box_at(1)
}
