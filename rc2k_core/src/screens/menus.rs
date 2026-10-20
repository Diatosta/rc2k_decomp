//! The menus' texts (converted 2026-10-20): `screens::draw_menu_labels` 0x471EF0 (a menu's
//! fixed labels) and `screens::draw_menu_item_text` 0x471E99 (an item's text, spaced out or
//! not). The fonts are the menu tables' own (the screen's font type).

use crate::Invalid;
use crate::screens::screen::{Buffer, Screen};

/// The work variable of the spaced-out text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `dword_7559EC`: the spaced-out text's font.
    SpacedFont,
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
        fn sprite(&mut self, _: u32, _: i32, _: i32, _: u32) -> Result<(), Invalid> {
            unreachable!()
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
}
