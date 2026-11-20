//! The menus: a menu's fixed labels, an item's text (spaced out or not) and the items (what is
//! behind each, then its text).

use crate::Invalid;
use crate::screens::screen::{Buffer, Screen};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// A label: x, y, font, string.
#[derive(Clone, Copy, Debug)]
pub struct Label<F> {
    pub x: i32,
    pub y: i32,
    pub font: F,
    pub id: u32,
}

/// Draws the labels, each a string.
pub fn labels<S: Screen>(s: &mut S, labels: &[Label<S::Font>]) -> Result<(), S::Error> {
    for l in labels {
        s.string(l.font, l.id, l.x, l.y)?;
    }
    Ok(())
}

/// The line buffer's length.
const LINE_LEN: usize = 0x100;

/// An item's text: one string, or with `spaced` one character at a time 18 pixels apart from
/// x - 26. An empty spaced text draws nothing. Refused: a string of 0x100 bytes or more with
/// its NUL.
pub(crate) fn item_text<S: Screen<Work = Work>>(
    s: &mut S,
    id: u32,
    font: S::Font,
    x: i32,
    y: i32,
    spaced: bool,
) -> Result<(), S::Error> {
    if !spaced {
        s.string(font, id, x, y)?;
        return Ok(());
    }
    let line = s.string_bytes(id);
    if line.len() >= LINE_LEN {
        return Err(Invalid("a string longer than the line buffer").into());
    }
    let mut at = x.wrapping_sub(0x1A);
    for &character in &line {
        s.store(Buffer::Cells, &[character, 0, 0, 0])?;
        s.draw(font, Buffer::Cells, at, y)?;
        at = at.wrapping_add(0x12);
    }
    Ok(())
}

/// What is behind an item's text: nothing, or one of the buttons below.
pub const NOTHING: u32 = 0xFFFF_FFA9;
pub(crate) const OWN_BUTTON: u32 = 0xFFFF_FFA5;
pub(crate) const RIGHT_BUTTON: u32 = 0xFFFF_FFA7;
pub(crate) const CENTRED_BUTTON: u32 = 0xFFFF_FFA6;
/// Also nothing, looked at only when the item is not highlighted; as a highlighted item's sprite
/// it means nothing at all behind it.
pub const NOTHING_EITHER: u32 = 0xFFFF_FFAA;

/// What an item's +0x20 holds: a sprite number, or for a button one of the menus' lists of
/// five sprite numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemSprite {
    Number(u32),
    List(usize),
}

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
    /// The item's text string.
    pub id: u32,
    /// The item's font; `pulse_font` is the highlighted item's second font.
    pub font: F,
    pub pulse_font: F,
    /// What is behind the text (`NOTHING` and the buttons).
    pub behind: u32,
    pub sprite: ItemSprite,
    /// The text is drawn spaced out.
    pub spaced: bool,
}

/// The buttons behind the texts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextButton {
    /// Right-aligned, as wide as the text.
    Right,
    /// Centred.
    Centred,
    /// The highlighted item's, right-aligned or centred.
    HighlightedRight,
    HighlightedCentred,
}

/// What the items' drawing asks of the game beside the screen.
pub trait MenuScreen: Screen<Work = Work> {
    /// Item `n`'s own button.
    fn own_button(&mut self, n: usize) -> Result<(), Self::Error>;
    /// A button as wide as item `n`'s text.
    fn text_button(&mut self, button: TextButton, n: usize) -> Result<(), Self::Error>;
    /// The font blended between two by the time since the last key.
    fn blended(
        &mut self,
        since_key: u32,
        a: Self::Font,
        b: Self::Font,
    ) -> Result<Self::Font, Self::Error>;
    /// A sprite list drawn where a sprite number is drawn.
    fn list_sprite(&mut self, list: usize, x: i32, y: i32) -> Result<(), Self::Error>;
}

/// The sprite an item's +0x20 names, at (`x`, `y`).
fn draw_sprite<S: MenuScreen>(
    s: &mut S,
    sprite: ItemSprite,
    x: u32,
    y: u32,
) -> Result<(), S::Error> {
    match sprite {
        ItemSprite::Number(glyph) => s.sprite(glyph, x as i32, y as i32),
        ItemSprite::List(list) => s.list_sprite(list, x as i32, y as i32),
    }
}

/// The items. `current` is the highlighted item's kind, `since_key` the time since the last key
/// (0 while the blink is off).
///
/// Sprites take no palette and text buttons' measuring no x: menu sprites are direct colour and
/// a measured width does not depend on where measuring starts.
pub fn items<S: MenuScreen>(
    s: &mut S,
    items: &[Item<S::Font>],
    current: u32,
    since_key: u32,
) -> Result<(), S::Error> {
    for (n, it) in items.iter().enumerate() {
        let highlighted = current == it.kind;
        if !highlighted {
            match it.behind {
                NOTHING | NOTHING_EITHER => {}
                OWN_BUTTON => s.own_button(n)?,
                RIGHT_BUTTON => s.text_button(TextButton::Right, n)?,
                CENTRED_BUTTON => s.text_button(TextButton::Centred, n)?,
                _ => draw_sprite(s, it.sprite, it.x, it.y)?,
            }
        } else if it.sprite != ItemSprite::Number(NOTHING_EITHER) {
            match it.behind {
                NOTHING => {}
                OWN_BUTTON => s.own_button(n)?,
                RIGHT_BUTTON => s.text_button(TextButton::HighlightedRight, n)?,
                CENTRED_BUTTON => s.text_button(TextButton::HighlightedCentred, n)?,
                _ => draw_sprite(s, it.sprite, it.x, it.y)?,
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
        let font = if highlighted {
            s.blended(since_key, it.font, it.pulse_font)?
        } else {
            it.font
        };
        item_text(s, it.id, font, x as i32, y as i32, it.spaced)?;
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
        item_text(&mut s, 1, 7, 100, 50, true).unwrap();
        assert_eq!(s.0, ["7 65 74 50", "7 66 92 50"]);
        // An empty string draws nothing.
        let mut s = Record::default();
        item_text(&mut s, 2, 7, 100, 50, true).unwrap();
        assert!(s.0.is_empty());
        assert!(item_text(&mut Record::default(), 3, 7, 0, 0, true).is_err());
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
        fn list_sprite(&mut self, list: usize, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("list {list} {x:#x} {y:#x}"));
            Ok(())
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
            item(1, 0x40, ItemSprite::Number(0x40)),
            item(2, RIGHT_BUTTON, ItemSprite::Number(0)),
            item(3, 0x41, ItemSprite::Number(0x41)),
            item(4, CENTRED_BUTTON, ItemSprite::Number(NOTHING_EITHER)),
        ];
        let mut s = Record::default();
        items(&mut s, &list, 3, 7).unwrap();
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
