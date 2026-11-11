//! A page's pieces made from its list, measured and drawn (from
//! `install::lay_out_pieces` 0x414C7E, `install::measure_pieces` 0x414DF5 and
//! `install::draw_pieces` 0x415058). A list: +0 its flags (hidden: 0x40 of 0xC0), +1 its
//! items (0: 0x100), +2 its title's string, from +0x18 its items, 0x10 bytes each: +0 the
//! kind (low four bits: 1 a link, 2 a choice, 3 a value, 4 an edit, 5 a text, 6 an action)
//! and marks, +1 a count or a font, +2 its string, +0xC a choice's table (a word a choice,
//! its high bits marks) or an edit's record. A piece (`crate::install::layout::PIECE`
//! bytes): +0 its item's kind and marks, +1 its own kind (0x10 and up: drawn by its item),
//! +2 its string, +4 its item, then its place and size (zeroed here).

use crate::install::layout::PIECE;

/// The bytes the page's pieces have.
pub const ROOM: u32 = 0xFFC;
/// The most pieces a page has room for (85).
pub const MOST_PIECES: u32 = ROOM / PIECE;
/// The fatal error of an item of a kind the installer does not know.
pub const UNKNOWN_KIND: u32 = 0x80A9_7451;

/// A list as the layout reads it, each part only when the original reads it: its header,
/// its items by number, a choice item's table and a text item's font (`byte_51D52C`).
pub trait ListSource {
    /// +0: the list's flags.
    fn flags(&self) -> u8;
    /// +1: its number of items (0: 0x100).
    fn count(&self) -> u8;
    /// +2: its title's string.
    fn title_text(&self) -> u16;
    /// Item `k`'s first four bytes: its kind and marks, its count or font, its string.
    fn item(&self, k: u32) -> Item;
    /// Choice `c` of item `k`'s table (a choice item): a string, its high bits marks.
    fn choice(&self, k: u32, c: u32) -> u16;
    /// The font of a text item's font number.
    fn text_font(&self, font: u8) -> u8;
}

/// An item's first four bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    /// +0: the kind (low four bits) and marks.
    pub first: u8,
    /// +1: a choice's number of choices, a text's font number.
    pub count: u8,
    /// +2: its string.
    pub text: u16,
}

/// Hidden: 0x40 of the marks 0xC0.
pub fn hidden(marks: u8) -> bool {
    marks & 0xC0 == 0x40
}

fn count_of(b: u8) -> u32 {
    if b == 0 { 0x100 } else { b as u32 }
}

/// Why a page cannot be laid out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    /// An item of an unknown kind: fatal error `UNKNOWN_KIND`.
    UnknownKind,
    /// More pieces than the page has room for (the original writes over what follows).
    TooMany,
}

/// The pieces a list makes (the original's first pass), or its unknown kind.
pub fn count(list: &impl ListSource) -> Result<u32, LayoutError> {
    let mut n = if hidden(list.flags()) { 0 } else { 1 };
    for k in 0..count_of(list.count()) {
        let item = list.item(k);
        let b = item.first;
        if hidden(b) {
            continue;
        }
        n += match b & 0xF {
            1 | 5 | 6 => 1,
            3 => 2,
            4 => 1 + (b & 0x10 == 0) as u32,
            2 => {
                1 + (0..count_of(item.count))
                    .filter(|&c| !hidden((list.choice(k, c) >> 8) as u8))
                    .count() as u32
            }
            _ => return Err(LayoutError::UnknownKind),
        };
    }
    Ok(n)
}

/// Whose piece a piece is: the page's title's, or item `n`'s of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Title,
    Item(u32),
}

/// A piece made: its first two bytes, its string (`None`: the table's word left as it
/// was) and its item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewPiece {
    pub marks: u8,
    pub kind: u8,
    pub text: Option<u16>,
    pub owner: Owner,
}

/// The pieces of `list`, in order: the title (kind 0x10) unless the list is hidden; for
/// each item not hidden, a link a piece of kind 0, a choice a label (1) and a piece (2) for
/// each choice not hidden (its marks from the choice's high bits), a value a label and a
/// piece (0x12), an edit the same but with 0x10 one piece (0x12, the label's string), a
/// text kind 0x10 with its font's bits, an action 0x14. Refused, before any: an unknown
/// kind, more than `MOST_PIECES`.
pub fn lay_out(list: &impl ListSource) -> Result<Vec<NewPiece>, LayoutError> {
    if count(list)? > MOST_PIECES {
        return Err(LayoutError::TooMany);
    }
    let mut out = vec![];
    let piece = |marks, kind, text, owner| NewPiece {
        marks,
        kind,
        text: Some(text),
        owner,
    };
    if !hidden(list.flags()) {
        out.push(piece(5, 0x10, list.title_text(), Owner::Title));
    }
    for k in 0..count_of(list.count()) {
        let item = list.item(k);
        let b = item.first;
        if hidden(b) {
            continue;
        }
        let (text, owner) = (item.text, Owner::Item(k));
        match b & 0xF {
            1 => out.push(piece(b, 0, text, owner)),
            2 => {
                out.push(piece(b, 1, text, owner));
                for c in 0..count_of(item.count) {
                    let word = list.choice(k, c);
                    let high = (word >> 8) as u8 & 0xC0;
                    if high == 0x40 {
                        continue;
                    }
                    let text = word.wrapping_sub((high as u16) << 8);
                    out.push(piece(high | (b & 0x3F), 2, text, owner));
                }
            }
            3 => {
                out.push(piece(b, 1, text, owner));
                out.push(NewPiece {
                    marks: b,
                    kind: 0x12,
                    text: None,
                    owner,
                });
            }
            4 => {
                // With 0x10 the value's piece goes over the label's, keeping its string.
                let label = b & 0x10 != 0;
                if !label {
                    out.push(piece(b, 1, text, owner));
                }
                out.push(NewPiece {
                    marks: b & 0xEF,
                    kind: 0x12,
                    text: label.then_some(text),
                    owner,
                });
            }
            5 => {
                let font = list.text_font(item.count);
                out.push(piece(b, font | 0x10, text, owner));
            }
            _ => out.push(piece(b, 0x14, text, owner)),
        }
    }
    Ok(out)
}

/// A piece's kind once measured: with its item's 0x20, bits 0, 1 moved to bits 2, 3.
pub fn measured_kind(marks: u8, kind: u8) -> u8 {
    if marks & 0x20 == 0 {
        return kind;
    }
    let low = kind & 3;
    (low << 2) | (kind - low)
}

/// How a piece is measured or drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// A plain text kind (no 0x10): its string, in the font of `font_kind` (its item's
    /// 0x10 and its own kind's low four bits).
    String { font_kind: u32 },
    /// By its item's kind: an edit.
    Edit,
    /// A text.
    Text,
    /// An action.
    Action,
    /// Any other: nothing.
    Other,
}

impl Way {
    pub fn of(marks: u8, kind: u8) -> Way {
        if kind & 0x10 == 0 {
            return Way::String {
                font_kind: (marks & 0x10) as u32 | (kind & 0xF) as u32,
            };
        }
        match marks & 0xF {
            4 => Way::Edit,
            5 => Way::Text,
            6 => Way::Action,
            _ => Way::Other,
        }
    }
}

/// The colour byte (`byte_51D528`, 8 a kind) of a piece's marks (bits 6, 7) and a kind.
pub fn colour_at(marks: u8, kind: u32) -> u32 {
    (marks >> 6) as u32 + 8 * kind
}

/// The colour byte a piece is measured with: the font's at 5 in the kind's eight.
pub fn measure_colour_at(kind: u32) -> u32 {
    5 + 8 * kind
}

/// Whether an edit is drawn with the caret after its text: lit (marks 3) and its record
/// being edited and showing it (bits 0 and 1).
pub fn caret(marks: u8, record_flags: u8) -> bool {
    (marks >> 6) & 3 == 3 && record_flags & 1 != 0 && record_flags & 2 != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    struct List {
        flags: u8,
        items: Vec<(u8, u8, u16)>,
        choices: Vec<u16>,
    }

    impl ListSource for List {
        fn flags(&self) -> u8 {
            self.flags
        }
        fn count(&self) -> u8 {
            self.items.len() as u8
        }
        fn title_text(&self) -> u16 {
            0x77
        }
        fn item(&self, k: u32) -> Item {
            let (first, count, text) = self.items[k as usize];
            Item { first, count, text }
        }
        fn choice(&self, _k: u32, c: u32) -> u16 {
            self.choices[c as usize]
        }
        fn text_font(&self, font: u8) -> u8 {
            font + 0x20
        }
    }

    fn list(flags: u8, items: &[(u8, u8, u16)], choices: &[u16]) -> List {
        List {
            flags,
            items: items.to_vec(),
            choices: choices.to_vec(),
        }
    }

    #[test]
    fn laying_out() {
        let m = list(
            0,
            &[(1, 0, 0x10), (2, 3, 0x11), (4, 0, 0x12)],
            &[0x30, 0x4031, 0x8032],
        );
        let p = lay_out(&m).unwrap();
        assert_eq!(count(&m), Ok(p.len() as u32));
        assert_eq!(p[0].owner, Owner::Title);
        assert_eq!(p[4].owner, Owner::Item(1));
        let kinds: Vec<_> = p.iter().map(|p| (p.marks, p.kind, p.text)).collect();
        assert_eq!(
            kinds,
            [
                (5, 0x10, Some(0x77)),
                (1, 0, Some(0x10)),
                (2, 1, Some(0x11)),
                (2, 2, Some(0x30)),
                (0x82, 2, Some(0x32)),
                (4, 1, Some(0x12)),
                (4, 0x12, None),
            ]
        );
        let m = list(
            0x40,
            &[(0x14, 0, 0x12), (5, 2, 0x13), (0x43, 0, 0), (6, 0, 0x14)],
            &[],
        );
        let p = lay_out(&m).unwrap();
        let kinds: Vec<_> = p.iter().map(|p| (p.marks, p.kind, p.text)).collect();
        assert_eq!(
            kinds,
            [
                (4, 0x12, Some(0x12)),
                (5, 0x32, Some(0x13)),
                (6, 0x14, Some(0x14))
            ]
        );
        assert_eq!(
            lay_out(&list(0, &[(7, 0, 0)], &[])),
            Err(LayoutError::UnknownKind)
        );
        let many = vec![(3u8, 0u8, 0u16); 43];
        assert_eq!(lay_out(&list(0, &many, &[])), Err(LayoutError::TooMany));
        assert_eq!(
            lay_out(&list(0x40, &many[..42], &[])).map(|p| p.len()),
            Ok(84)
        );
    }

    #[test]
    fn measuring_and_drawing() {
        assert_eq!(measured_kind(0, 0x12), 0x12);
        assert_eq!(measured_kind(0x20, 0x12), 0x18);
        assert_eq!(measured_kind(0x20, 0x13), 0x1C);
        assert_eq!(Way::of(0x13, 2), Way::String { font_kind: 0x12 });
        assert_eq!(Way::of(4, 0x12), Way::Edit);
        assert_eq!(Way::of(5, 0x30), Way::Text);
        assert_eq!(Way::of(6, 0x14), Way::Action);
        assert_eq!(Way::of(1, 0x14), Way::Other);
        assert_eq!(colour_at(0xC4, 2), 3 + 16);
        assert_eq!(measure_colour_at(2), 21);
        assert!(caret(0xC4, 3));
        assert!(!caret(0x84, 3));
        assert!(!caret(0xC4, 1));
    }
}
