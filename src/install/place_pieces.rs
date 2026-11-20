//! A page's pieces placed, box by box: the pieces in a row whose items share a box go to
//! `install::place_row` together. Refused: no pieces.

use crate::errors::Refused;
use crate::install::page_links::PageLink;
use crate::install::page_records::{BoxRecord, item_of};
use crate::install::place_row::place_row;

pub(super) fn place_pieces(
    install: &'static crate::install::owner::Install,
) -> Result<(), Refused> {
    let rows = {
        let laid = install.pieces.borrow();
        rc2k_core::install::layout::rows(laid.len() as u32, |k| {
            item_of(&laid[k as usize]).box_link().raw()
        })?
    };
    for (first, n, the_box) in rows {
        let link = PageLink::from_raw(the_box);
        let title = link.is_none() || link == title_box_link();
        place_row(install, first, n, BoxRecord::linked(install, link), title)?;
    }
    Ok(())
}

fn title_box_link() -> PageLink {
    PageLink::box_at(1)
}
