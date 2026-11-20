//! A page's pieces made from its items: the title, then for each item not hidden: a link a piece,
//! a value or edit a label and a piece, a text one piece. Places and sizes are zeroed. More than 85
//! pieces are refused.

use crate::errors::{BadInput, Refused};
use crate::install::installer::fatal;
use rc2k_core::install::model::{Item, Owner, PageDef};
use rc2k_core::install::pieces::{self, LayoutError};

/// A piece of the page being shown: it names the item it was made from.
pub type Piece = rc2k_core::install::layout::Piece<Owner>;

/// The item a piece was made from; a piece made from none is refused.
pub(super) fn item_of<'a>(def: &'a PageDef, piece: &Piece) -> Result<&'a Item, BadInput> {
    piece
        .item
        .map(|owner| def.owner_item(owner))
        .ok_or(BadInput("a piece made from no item"))
}

pub(super) fn lay_out_pieces(
    install: &'static crate::install::owner::Install,
    def: &PageDef,
) -> Result<Vec<Piece>, Refused> {
    let made = match def.lay_out(|font| install.pages.kind_fonts.get(4 + 8 * font as usize)) {
        Ok(made) => made,
        Err(LayoutError::UnknownKind) => return Err(fatal(pieces::UNKNOWN_KIND)),
        Err(LayoutError::TooMany) => {
            return Err(BadInput("more pieces than the page's table holds").into());
        }
    };
    Ok(made
        .iter()
        .map(|new| Piece {
            marks: new.marks,
            kind: new.kind,
            // A piece made with no text keeps no string.
            text: new.text.unwrap_or(0),
            item: Some(new.owner),
            ..Piece::default()
        })
        .collect())
}
