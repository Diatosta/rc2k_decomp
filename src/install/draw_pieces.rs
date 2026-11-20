//! Each of a page's pieces drawn at its place: a plain text piece as its string in the font of its
//! marks; else by the item's kind: an edit its text (with the caret after it while it is edited), a
//! text its string and the item's appended text, an action its drawing action. Refused: no pieces.
//! The caller has locked the surface.

use crate::errors::{BadInput, Refused};
use crate::install::installer::{draw_string, draw_text, push_string_by_id, store_text};
use crate::install::lay_out_pieces::{Piece, item_of};
use crate::install::vars::InstallerPages;
use crate::strings::owner::Strings;
use rc2k_core::install::model::{Kind, PageDef};
use rc2k_core::install::pieces::{self, Way};

fn font(
    install: &crate::install::owner::Install,
    index: u32,
) -> Result<crate::gfx::typefaces::FontId, BadInput> {
    crate::install::vars::InstallerPages::font_of(install, index)
}

fn colour(install: &crate::install::owner::Install, marks: u8, kind: u32) -> u32 {
    InstallerPages::kind_font(install, pieces::colour_at(marks, kind)) as u32
}

/// `path` kept whole when it and the `caret` after it fit in `widest` pixels (`width` measures a
/// text); else its end with "..." in place of the start, as much of the end as fits, so the caret
/// stays in view.
/// A path is cut to its end where the folder field's text starts at its first byte (no drive
/// letters); with drive letters the text is shown as it is typed.
const SHOWN_BY_ITS_END: bool = crate::install::page_state::TEXT_AT == 0;

fn fit_path_end(path: &[u8], caret: &[u8], widest: u32, width: impl Fn(&[u8]) -> u32) -> Vec<u8> {
    let with = |cut: usize| -> Vec<u8> {
        let mut t = if cut == 0 {
            Vec::new()
        } else {
            b"...".to_vec()
        };
        t.extend(&path[cut..]);
        t.extend(caret);
        t
    };
    if width(&with(0)) <= widest {
        return path.to_vec();
    }
    // The least start cut that fits (the width falls as the cut grows).
    let (mut low, mut high) = (1, path.len());
    while low < high {
        let mid = (low + high) / 2;
        if width(&with(mid)) <= widest {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    with(low)[..with(low).len() - caret.len()].to_vec()
}

pub(super) fn draw_pieces(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    install: &'static crate::install::owner::Install,
    def: &PageDef,
    laid: &[Piece],
    strings: &'static Strings,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    if laid.is_empty() {
        return Err(BadInput("a page of no pieces").into());
    }
    for piece in laid {
        let first = piece.marks;
        let (x, y) = (piece.x, piece.y);
        match Way::of(first, piece.kind) {
            Way::String { font_kind } => {
                draw_string(
                    so,
                    piece.text as u32,
                    font(install, colour(install, first, font_kind))?,
                    x,
                    y,
                    gfx,
                )?;
            }
            Way::Edit => {
                let Kind::Edit(edit) = &item_of(def, piece)?.kind else {
                    return Err(BadInput("an edit piece of an item that is no edit").into());
                };
                let edit_font = font(install, colour(install, first, u32::from(edit.font_kind)))?;
                let mut text = edit.text.clone();
                let mut caret = Vec::new();
                let with_caret = pieces::caret(first, edit.flags());
                if with_caret {
                    push_string_by_id(so, &mut caret, 4);
                }
                if SHOWN_BY_ITS_END {
                    // The folder's path is shown by its end when it is longer than the field,
                    // measured with the measuring font of the field's kind.
                    let measuring = crate::install::page_state::edit_font(install, edit.font_kind)?;
                    text = fit_path_end(&text, &caret, u32::from(edit.width), |t| {
                        let mut t = t.to_vec();
                        t.push(0);
                        crate::install::installer::measure_text(measuring, &t, 0, gfx)
                            .map_or(0, |(width, _)| width)
                    });
                    text.extend(&caret);
                    text.push(0);
                    store_text(&strings.text.buffer.text, &text);
                } else if !with_caret {
                    // The drawer reads to the NUL.
                    text.push(0);
                } else {
                    // The text and the caret, kept in the text buffer (anything reads it
                    // afterwards only after another page text overwrote it).
                    text.extend(&caret);
                    text.push(0);
                    store_text(&strings.text.buffer.text, &text);
                }
                draw_text(edit_font, &text, x, y, gfx)?;
            }
            Way::Text => {
                let Kind::Text {
                    font_kind,
                    ends_line,
                } = item_of(def, piece)?.kind
                else {
                    return Err(BadInput("a text piece of an item that is no text").into());
                };
                let mut text = Vec::new();
                push_string_by_id(so, &mut text, piece.text as u32);
                if ends_line {
                    crate::install::end_text_line::end_text_line(&mut text);
                }
                text.push(0);
                store_text(&strings.text.buffer.text, &text);
                draw_text(
                    font(install, colour(install, first, u32::from(font_kind)))?,
                    &text,
                    x,
                    y,
                    gfx,
                )?;
            }
            Way::Action => return Err(BadInput("an action item (the pages hold none)").into()),
            Way::Other => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::fit_path_end;

    /// Ten pixels a byte.
    fn width(t: &[u8]) -> u32 {
        t.len() as u32 * 10
    }

    #[test]
    fn a_long_path_is_shown_by_its_end() {
        assert_eq!(fit_path_end(b"/a/b", b"_", 100, width), b"/a/b");
        let shown = fit_path_end(
            b"/Users/someone/Library/Application Support/Rally",
            b"_",
            150,
            width,
        );
        assert_eq!(shown, b"...pport/Rally");
        assert!(width(&shown) + 10 <= 150);
    }
}
