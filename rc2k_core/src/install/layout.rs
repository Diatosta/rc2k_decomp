//! A page's pieces placed in their boxes (converted 2026-10-21 from `install::place_pieces`
//! 0x414ED1 and `install::place_row` 0x414F11). A piece (0x30 bytes in the page's table):
//! +0 its item's kind and marks, +1 its own kind (bit 0 a label, bit 1 a choice; for the
//! rest bit 2 at the left, bit 3 at the right, else centred), +8 x, +0xC y, +0x10 width,
//! +0x14 height. A box: x, y, width, height. Compares are signed, as the original's. And
//! the drawing lists of the installer's frames (converted the same day from
//! `install::draw_string_list` 0x402370 and `install::blit_glyph_list` 0x402628).

use crate::Invalid;

/// A piece's bytes in the page's table.
pub const PIECE: u32 = 0x30;

/// What placing reads of a piece and changes (x and y).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Piece {
    pub marks: u8,
    pub kind: u8,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// A box of a page: `{x, y, width, height}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Area {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// What `place_row` works out: the box's x, how far the whole was moved down in the box,
/// the height of the pieces, and how far the labels and choices were centred across.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    pub x: u32,
    pub down: u32,
    pub height: u32,
    pub across: u32,
}

fn i(v: u32) -> i32 {
    v as i32
}

/// A label's mark in a piece's kind, and a choice's.
const LABEL: u8 = 1;
const CHOICE: u8 = 2;
/// With neither, bit 2 keeps a piece at the left and bit 3 puts it at the right edge.
const AT_LEFT: u8 = 4;
const AT_RIGHT: u8 = 8;
/// The pixels between a choice and what follows it.
const GAP: u32 = 10;

/// The pieces of one box placed in it: labels in a column as wide as the widest, choices
/// after their label and each other 10 pixels apart, wrapped to a new line where they would
/// pass the box's width, the rest centred (or at the left, or at the right edge); then the
/// whole centred across and down in the box (by an even number of pixels down in the
/// title's box) and moved to its place. Refused: no pieces (the original goes round 2^32
/// times).
pub fn place_row(pieces: &mut [Piece], area: Area, title: bool) -> Result<Placed, Invalid> {
    if pieces.is_empty() {
        return Err(Invalid("a box of no pieces"));
    }
    let labels = widest_label_and_push_choices(pieces);
    wrap_lines(pieces, area.width.wrapping_sub(labels));
    let right_edge = place_across(pieces, area.width, labels);
    // The labels and choices centred across, the height found.
    let across = (i(area.width.wrapping_sub(right_edge)) >> 1) as u32;
    let mut height = 0u32;
    for p in pieces.iter_mut() {
        if p.kind & (LABEL | CHOICE) != 0 {
            p.x = p.x.wrapping_add(across);
        }
        let bottom = p.y.wrapping_add(p.height);
        if i(height) < i(bottom) {
            height = bottom;
        }
    }
    // The whole moved to the box.
    let mut down = (i(area.height.wrapping_sub(height)) >> 1) as u32;
    if title {
        down &= !1;
    }
    let down = down.wrapping_add(area.y);
    for p in pieces.iter_mut() {
        p.x = p.x.wrapping_add(area.x);
        p.y = p.y.wrapping_add(down);
    }
    Ok(Placed {
        x: area.x,
        down,
        height,
        across,
    })
}

/// The widest label's width; a choice followed by a choice pushes the next one to the right
/// of its own end (and the gap).
fn widest_label_and_push_choices(pieces: &mut [Piece]) -> u32 {
    let mut widest = 0u32;
    for k in 0..pieces.len() {
        let kind = pieces[k].kind;
        if kind & LABEL != 0 {
            if i(widest) < i(pieces[k].width) {
                widest = pieces[k].width;
            }
        } else if kind & CHOICE != 0 && k + 1 < pieces.len() && pieces[k + 1].kind & CHOICE != 0 {
            let after = pieces[k].x.wrapping_add(pieces[k].width).wrapping_add(GAP);
            if i(after) > i(pieces[k + 1].x) {
                pieces[k + 1].x = after;
            }
        }
    }
    widest
}

/// Breaks the pieces into lines. A choice stays on its line while its end is within `limit`
/// (the box's width less the label column); any other piece, and a choice past the limit,
/// starts a new line: the pieces from it on move down by the line's height (and back left by
/// the new line's first choice's x, which choices after a plain piece do not follow), and the
/// pieces of the line just ended move down to its bottom.
fn wrap_lines(pieces: &mut [Piece], limit: u32) {
    let mut line_start = 0usize;
    let mut line_height = pieces[0].height;
    for k in 1..pieces.len() {
        let piece = pieces[k];
        let back = if piece.kind & CHOICE == 0 {
            0
        } else if i(piece.x.wrapping_add(piece.width)) <= i(limit) {
            // It fits: the line is as high as its highest piece.
            if i(line_height) < i(piece.height) {
                line_height = piece.height;
            }
            continue;
        } else {
            piece.x
        };
        let mut back = back;
        for p in &mut pieces[k..] {
            if p.kind & CHOICE == 0 {
                back = 0;
            }
            p.x = p.x.wrapping_sub(back);
            p.y = p.y.wrapping_add(line_height);
        }
        for p in &mut pieces[line_start..k] {
            p.y = p.y.wrapping_add(line_height.wrapping_sub(p.height));
        }
        line_start = k;
        line_height = piece.height;
    }
}

/// Each piece's place across: labels at the label column's end, choices after it and the
/// gap, the rest centred, at the left or at the right edge. The right-most end of the
/// choices.
fn place_across(pieces: &mut [Piece], width: u32, labels: u32) -> u32 {
    let mut right_edge = 0u32;
    for p in pieces.iter_mut() {
        if p.kind & LABEL != 0 {
            p.x = p.x.wrapping_add(labels);
        } else if p.kind & CHOICE != 0 {
            p.x = p.x.wrapping_add(labels).wrapping_add(GAP);
            let right = p.x.wrapping_add(p.width);
            if i(right_edge) < i(right) {
                right_edge = right;
            }
        } else if p.marks & 0x20 != 0 || p.kind & (AT_LEFT | AT_RIGHT) == 0 {
            p.x = p.x.wrapping_add((i(width) >> 1) as u32);
        } else if p.kind & AT_LEFT == 0 {
            p.x = p.x.wrapping_add(width.wrapping_sub(1));
        }
    }
    right_edge
}

/// A page's pieces in rows of the same box (`box_of`, asked of each piece in turn, the next
/// only while pieces are left): each row's first piece, its pieces and its box. Refused: a
/// page of no pieces (the original goes round 2^32 times).
pub fn rows(count: u32, box_of: impl Fn(u32) -> u32) -> Result<Vec<(u32, u32, u32)>, Invalid> {
    if count == 0 {
        return Err(Invalid("a page of no pieces"));
    }
    let mut rows = Vec::new();
    let mut at = 0u32;
    while at < count {
        let the_box = box_of(at);
        let first = at;
        loop {
            at += 1;
            if at == count || box_of(at) != the_box {
                break;
            }
        }
        rows.push((first, at - first, the_box));
    }
    Ok(rows)
}

/// A drawing list's record's bytes: dx, dy and two dwords (a font and a string, or a
/// glyph and a palette).
pub const LIST_RECORD: u32 = 0x10;

/// A drawing list of `count` records: each record's offset in the table and what ebp holds
/// while it is drawn (the records left, this one counted), in order. Refused: a count of 0
/// (the original's do-while goes round 2^32 times).
pub fn list_records(count: u32) -> Result<Vec<(u32, u32)>, Invalid> {
    if count == 0 {
        return Err(Invalid(
            "a count of 0 (the original's do-while goes round 2^32 times)",
        ));
    }
    Ok((0..count)
        .map(|n| (n.wrapping_mul(LIST_RECORD), count - n))
        .collect())
}

/// Where a list's record is drawn: its offsets from the list's place (wrapping).
pub fn list_place(dx: u32, dy: u32, x: u32, y: u32) -> (u32, u32) {
    (dx.wrapping_add(x), dy.wrapping_add(y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists() {
        assert!(list_records(0).is_err());
        assert_eq!(list_records(3), Ok(vec![(0, 3), (0x10, 2), (0x20, 1)]));
        assert_eq!(list_place(5, u32::MAX, 10, 2), (15, 1));
    }

    fn piece(kind: u8, width: u32, height: u32) -> Piece {
        Piece {
            kind,
            width,
            height,
            ..Piece::default()
        }
    }

    #[test]
    fn a_row() {
        let area = Area {
            x: 100,
            y: 50,
            width: 200,
            height: 40,
        };
        let mut pieces = [piece(1, 30, 10), piece(2, 50, 10), piece(2, 60, 12)];
        let placed = place_row(&mut pieces, area, false).unwrap();
        // Label 30 wide; the choices at 0 and 60, after the label and 10: 40, 100; their
        // right edge 160: centred across by 20; height 12, down by 14.
        assert_eq!(
            placed,
            Placed {
                x: 100,
                down: 64,
                height: 12,
                across: 20
            }
        );
        assert_eq!((pieces[0].x, pieces[1].x, pieces[2].x), (150, 160, 220));
        assert_eq!((pieces[0].y, pieces[1].y, pieces[2].y), (64, 64, 64));
        assert!(place_row(&mut [], area, false).is_err());
    }

    #[test]
    fn a_wrap_and_the_rest() {
        let area = Area {
            x: 0,
            y: 0,
            width: 100,
            height: 101,
        };
        let mut pieces = [
            piece(2, 60, 10),
            piece(2, 60, 20),
            piece(0, 40, 5),
            piece(4, 8, 5),
        ];
        let placed = place_row(&mut pieces, area, true).unwrap();
        // The second choice wraps under the first (y 10, x back to 0); the plain pieces go
        // on lines of their own (y 30, 35), one centred, the one with bit 2 at 0; the
        // choices centred across by 15; 40 high, down by 30 (61 / 2, made even).
        assert_eq!(pieces.map(|p| p.y), [30, 40, 60, 65]);
        assert_eq!(pieces.map(|p| p.x), [25, 25, 50, 0]);
        assert_eq!(
            placed,
            Placed {
                x: 0,
                down: 30,
                height: 40,
                across: 15
            }
        );
    }

    #[test]
    fn row_grouping() {
        let boxes = [0u32, 0, 7, 7, 0];
        assert_eq!(
            rows(5, |n| boxes[n as usize]),
            Ok(vec![(0, 2, 0), (2, 2, 7), (4, 1, 0)])
        );
        assert!(rows(0, |_| 0).is_err());
    }
}
