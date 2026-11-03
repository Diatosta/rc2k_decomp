//! The menus (converted 2026-10-20): `screens::draw_menu_labels` 0x471EF0 (a menu's fixed
//! labels), `screens::draw_menu_item_text` 0x471E99 (an item's text, spaced out or not) and
//! `screens::draw_menu_items` 0x471D0A (the items: what is behind each, then its text). The
//! fonts are the menu tables' own (the screen's font type).

use crate::Invalid;
use crate::screens::screen::{Buffer, Screen};

/// The work variable of the spaced-out text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `dword_7559EC`: the spaced-out text's font.
    SpacedFont,
    /// `g_markerX`, `g_markerY`: where the item's text went.
    MarkerX,
    MarkerY,
    /// `dword_7559D0`: 1 while the highlighted item is drawn.
    Highlighted,
}

/// A label: x, y, font, string.
#[derive(Clone, Copy, Debug)]
pub struct Label<F> {
    pub x: i32,
    pub y: i32,
    pub font: F,
    pub id: u32,
}

/// The labels, each a string.
pub fn labels<S: Screen>(s: &mut S, labels: &[Label<S::Font>]) -> Result<(), S::Error> {
    for l in labels {
        s.string(l.font, l.id, l.x, l.y)?;
    }
    Ok(())
}

/// The line buffer's length: `byte_A8BC54` (the one-character text) follows it.
const LINE_LEN: usize = 0x100;

/// An item's text: one string, or with `spaced` one character at a time 18 pixels apart
/// from x - 26, each through a one-character text in `byte_A8BC54`, the font kept
/// (`font_value`, as the game keeps it). The string is put in the line buffer with its NUL
/// and read from there: the first character is drawn even when it is an empty string's
/// NUL, and then the loop goes on into what the line buffer held after it (`after(n)`,
/// asked for each byte past the string the original reads, n < 0x100). At the buffer's end
/// it reads the one-character text: its first byte the character drawn last, then the 0
/// written over it. Refused: a string of 0x100 bytes or more with its NUL.
#[allow(clippy::too_many_arguments)] // the original's registers: eax .. ebp
pub fn item_text<S: Screen<Work = Work>>(
    s: &mut S,
    id: u32,
    font: S::Font,
    font_value: u32,
    x: i32,
    y: i32,
    spaced: bool,
    after: &dyn Fn(usize) -> u8,
) -> Result<(), S::Error> {
    if !spaced {
        s.string(font, id, x, y)?;
        return Ok(());
    }
    let mut line = s.string_bytes(id);
    line.push(0);
    if line.len() > LINE_LEN {
        return Err(Invalid("a string longer than the line buffer").into());
    }
    s.keep(Work::SpacedFont, font_value);
    s.store(Buffer::Line, &line)?;
    let byte = |n: usize| if n < line.len() { line[n] } else { after(n) };
    let mut x = x.wrapping_sub(0x1A);
    let mut place = 0usize;
    loop {
        // The one-character text is cleared before the byte is read: past the buffer the
        // byte is that text's own, so 0.
        let character = if place < LINE_LEN { byte(place) } else { 0 };
        s.store(Buffer::Cells, &[character, 0, 0, 0])?;
        s.draw(font, Buffer::Cells, x, y)?;
        x = x.wrapping_add(0x12);
        place += 1;
        let next = match place {
            n if n < LINE_LEN => byte(n),
            LINE_LEN => character,
            _ => 0,
        };
        if next == 0 {
            return Ok(());
        }
    }
}

/// What is behind an item's text (its +0x1C).
pub const NOTHING: u32 = 0xFFFF_FFA9;
pub const OWN_BUTTON: u32 = 0xFFFF_FFA5;
pub const RIGHT_BUTTON: u32 = 0xFFFF_FFA7;
pub const CENTRED_BUTTON: u32 = 0xFFFF_FFA6;
/// Nothing either, but only looked at when the item is not highlighted; as a highlighted
/// item's sprite (+0x20): nothing at all behind it.
pub const NOTHING_EITHER: u32 = 0xFFFF_FFAA;

/// A menu item (0x5C bytes): the fields the drawing reads.
#[derive(Clone, Copy, Debug)]
pub struct Item<F> {
    /// +0, +4: the sprite's place, and the text's for the button kinds.
    pub x: u32,
    pub y: u32,
    /// +8, +0xC: the text's place otherwise.
    pub text_x: u32,
    pub text_y: u32,
    /// +0x10: highlighted when it is the current item.
    pub kind: u32,
    /// +0x14.
    pub id: u32,
    /// +0x18, and +0x50 the highlighted item's second font.
    pub font: F,
    pub pulse_font: F,
    /// +0x1C, +0x20.
    pub behind: u32,
    pub sprite: u32,
    /// +0x58 is 1.
    pub spaced: bool,
}

/// The buttons behind the texts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextButton {
    /// Right-aligned, as wide as the text (`ALT`).
    Right,
    /// Centred (`PLAIN`).
    Centred,
    /// The highlighted item's, right-aligned or centred.
    HighlightedRight,
    HighlightedCentred,
}

/// What the items' drawing asks of the game beside the screen.
pub trait MenuScreen: Screen<Work = Work> {
    /// Item `n`'s own button (`screens::draw_item_button`).
    fn own_button(&mut self, n: usize) -> Result<(), Self::Error>;
    /// A button as wide as item `n`'s text (`screens::draw_text_button`).
    fn text_button(&mut self, button: TextButton, n: usize) -> Result<(), Self::Error>;
    /// The font blended between two by the time since the last key (`text::blend_font`).
    fn blended(
        &mut self,
        since_key: u32,
        a: Self::Font,
        b: Self::Font,
    ) -> Result<Self::Font, Self::Error>;
    /// A font as the registers hold it (its address in the game).
    fn font_value(&self, f: Self::Font) -> u32;
}

/// The items. `current` is the highlighted item's kind (`dword_755525`), `since_key` the
/// time since the last key (0 while the blink is off). `after` as for `item_text`.
///
/// The original hands each sprite a palette (its own number, or for the highlighted item
/// the font of the item before, or for the first the caller's ebx) and each text button's
/// measuring an x (the text x of the item before, or the caller's ecx). Neither is looked
/// at: the menu sprites are direct colour (`gfx::build_menu_sprites` checks) and a measured
/// width does not depend on where the measuring starts (`screens::buttons` checks the
/// measuring font only measures).
pub fn items<S: MenuScreen>(
    s: &mut S,
    items: &[Item<S::Font>],
    current: u32,
    since_key: u32,
    after: &dyn Fn(usize) -> u8,
) -> Result<(), S::Error> {
    for (n, it) in items.iter().enumerate() {
        let highlighted = current == it.kind;
        s.keep(Work::Highlighted, highlighted as u32);
        if !highlighted {
            if it.behind != NOTHING {
                s.keep(Work::MarkerX, it.x);
                s.keep(Work::MarkerY, it.y);
            }
            match it.behind {
                NOTHING | NOTHING_EITHER => {}
                OWN_BUTTON => s.own_button(n)?,
                RIGHT_BUTTON => s.text_button(TextButton::Right, n)?,
                CENTRED_BUTTON => s.text_button(TextButton::Centred, n)?,
                _ => s.sprite(it.sprite, it.x as i32, it.y as i32)?,
            }
        } else if it.sprite != NOTHING_EITHER {
            match it.behind {
                NOTHING => {}
                OWN_BUTTON => s.own_button(n)?,
                RIGHT_BUTTON => s.text_button(TextButton::HighlightedRight, n)?,
                CENTRED_BUTTON => s.text_button(TextButton::HighlightedCentred, n)?,
                _ => s.sprite(it.sprite, it.x as i32, it.y as i32)?,
            }
        }
        let is_button = matches!(it.behind, OWN_BUTTON | RIGHT_BUTTON | CENTRED_BUTTON);
        let (mut x, y) = if is_button {
            (it.x, it.y.wrapping_sub(3))
        } else {
            (it.text_x, it.text_y)
        };
        if it.behind == RIGHT_BUTTON {
            x = x.wrapping_sub(4);
        }
        s.keep(Work::MarkerX, x);
        s.keep(Work::MarkerY, y);
        let font = if highlighted {
            s.blended(since_key, it.font, it.pulse_font)?
        } else {
            it.font
        };
        let value = s.font_value(font);
        item_text(s, it.id, font, value, x as i32, y as i32, it.spaced, after)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::screen::{Bar, Drawn};

    #[derive(Default)]
    struct Record(Vec<String>, Vec<u8>);

    impl Screen for Record {
        type Font = u32;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, _: u32, _: u32, _: i32, _: i32) -> Result<Drawn, Invalid> {
            unreachable!()
        }
        fn store(&mut self, b: Buffer, bytes: &[u8]) -> Result<(), Invalid> {
            if b == Buffer::Cells {
                self.1 = bytes.to_vec();
            }
            Ok(())
        }
        fn draw(&mut self, f: u32, _: Buffer, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f} {} {x} {y}", self.1[0]));
            Ok(Drawn::default())
        }
        fn draw_time(&mut self, _: u32, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn draw_clock(&mut self, _: u32, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn string(&mut self, f: u32, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f} s{id:x} {x} {y}"));
            Ok(Drawn::default())
        }
        fn string_bytes(&mut self, id: u32) -> Vec<u8> {
            match id {
                1 => b"AB".to_vec(),
                2 => Vec::new(),
                _ => vec![b'x'; 0x100],
            }
        }
        fn strings(&mut self, _: u32, _: u32) -> Result<Vec<u8>, Invalid> {
            unreachable!()
        }
        fn sprite(&mut self, g: u32, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("sprite {g:#x} {x:#x} {y:#x}"));
            Ok(())
        }
        fn keep(&mut self, _: Work, v: u32) {
            self.0.push(format!("keep {v}"));
        }
        fn bar(&mut self, _: Bar) -> Result<(), Invalid> {
            unreachable!()
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
    }

    #[test]
    fn texts() {
        let mut s = Record::default();
        labels(
            &mut s,
            &[Label {
                x: 1,
                y: 2,
                font: 7,
                id: 0x30,
            }],
        )
        .unwrap();
        assert_eq!(s.0, ["7 s30 1 2"]);
        let mut s = Record::default();
        item_text(&mut s, 1, 7, 9, 100, 50, true, &|_| unreachable!()).unwrap();
        assert_eq!(s.0, ["keep 9", "7 65 74 50", "7 66 92 50"]);
        // An empty string: its NUL, then what the buffer held after it until a 0.
        let mut s = Record::default();
        item_text(&mut s, 2, 7, 9, 100, 50, true, &|n| {
            if n < 3 { b'q' } else { 0 }
        })
        .unwrap();
        assert_eq!(s.0, ["keep 9", "7 0 74 50", "7 113 92 50", "7 113 110 50"]);
        // A buffer with no 0 after the NUL: on to its end, then the last character again
        // and the cleared one-character text.
        let mut s = Record::default();
        item_text(&mut s, 2, 7, 9, 100, 50, true, &|_| b'q').unwrap();
        assert_eq!(s.0.len(), 1 + 0x101);
        assert_eq!(
            s.0.last().unwrap(),
            &format!("7 0 {} 50", 74 + 0x12 * 0x100)
        );
        assert!(item_text(&mut Record::default(), 3, 7, 9, 0, 0, true, &|_| 0).is_err());
    }

    impl MenuScreen for Record {
        fn own_button(&mut self, n: usize) -> Result<(), Invalid> {
            self.0.push(format!("own {n}"));
            Ok(())
        }
        fn text_button(&mut self, b: TextButton, n: usize) -> Result<(), Invalid> {
            self.0.push(format!("{b:?} {n}"));
            Ok(())
        }
        fn blended(&mut self, since: u32, a: u32, b: u32) -> Result<u32, Invalid> {
            self.0.push(format!("blend {since} {a} {b}"));
            Ok(99)
        }
        fn font_value(&self, f: u32) -> u32 {
            f + 1000
        }
    }

    #[test]
    fn menu_items() {
        let item = |kind, behind, sprite| Item {
            x: 10,
            y: 20,
            text_x: 30,
            text_y: 40,
            kind,
            id: 1,
            font: 5,
            pulse_font: 6,
            behind,
            sprite,
            spaced: false,
        };
        let list = [
            item(1, 0x40, 0x40),
            item(2, RIGHT_BUTTON, 0),
            item(3, 0x41, 0x41),
            item(4, CENTRED_BUTTON, NOTHING_EITHER),
        ];
        let mut s = Record::default();
        items(&mut s, &list, 3, 7, &|_| 0).unwrap();
        let keep: Vec<_> =
            s.0.iter()
                .filter(|l| !l.starts_with("keep"))
                .cloned()
                .collect();
        assert_eq!(
            keep,
            [
                "sprite 0x40 0xa 0x14",
                "5 s1 30 40",
                "Right 1",
                "5 s1 6 17",
                // Highlighted: the font blended.
                "sprite 0x41 0xa 0x14",
                "blend 7 5 6",
                "99 s1 30 40",
                "Centred 3",
                "5 s1 10 17",
            ]
        );
    }
}
