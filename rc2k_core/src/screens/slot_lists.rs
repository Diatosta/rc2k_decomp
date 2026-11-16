//! The name lists of the load and save pages (saved games, setups, replays) and the setup
//! pages around their list. Each list is a page of names of 0x20 bytes in two columns.

use crate::Invalid;
use crate::screens::screen::Screen;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// The saved games' and replays' names.
    Names,
    /// The setup pages.
    Setups,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// Where a list's names go.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub font: Font,
    pub columns: [i32; 2],
    pub first_y: i32,
    pub step: i32,
    /// Rows in a column.
    pub rows: u32,
    /// Pages in the table (100 names or more).
    pub pages: u32,
    /// What a page past the table is refused as.
    pub past: &'static str,
}

pub const SAVED_GAMES: Layout = Layout {
    font: Font::Names,
    columns: [0xCF, 0x21C],
    first_y: 0xB5,
    step: 0x1D,
    rows: 5,
    pages: 10,
    past: "a page past the saved games' names",
};

pub const SETUPS: Layout = Layout {
    font: Font::Setups,
    columns: [0xE9, 0x238],
    first_y: 0x44,
    step: 0x1D,
    rows: 15,
    pages: 3,
    past: "slot page past the name table",
};

pub const REPLAYS: Layout = Layout {
    font: Font::Names,
    columns: [0xC6, 0x213],
    first_y: 0xB5,
    step: 0x1B,
    rows: 10,
    pages: 5,
    past: "slot page past the name table",
};

/// Page `page` of a list, `name` giving slot n's text: returns the last name's `too_wide`.
pub fn names<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    layout: &Layout,
    page: u32,
    name: impl Fn(u32) -> S::Text,
) -> Result<bool, S::Error> {
    if page >= layout.pages {
        return Err(Invalid(layout.past).into());
    }
    let mut slot = page * layout.rows * 2;
    let mut too_wide = false;
    for x in layout.columns {
        for row in 0..layout.rows {
            let y = layout.first_y + layout.step * row as i32;
            too_wide = s.text(layout.font, name(slot), x, y)?.too_wide;
            slot += 1;
        }
    }
    Ok(too_wide)
}

/// The save setup page's title.
pub const SAVE_SETUP_TITLE: u32 = 0x4DC;
/// The load setup page's title.
pub const LOAD_SETUP_TITLE: u32 = 0x4DD;

/// A setup page: its title (string `title`) at (0x190, 0xE), the columns' labels (0x4E3,
/// 0x4E4) at y 0x1F7, then the setups' names.
pub fn setup_page<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    title: u32,
    page: u32,
    name: impl Fn(u32) -> S::Text,
) -> Result<(), S::Error> {
    s.string(Font::Setups, title, 0x190, 0xE)?;
    for (id, x) in [0x4E3, 0x4E4].into_iter().zip(SETUPS.columns) {
        s.string(Font::Setups, id, x, 0x1F7)?;
    }
    names(s, &SETUPS, page, name).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::screen::{Bar, Buffer, Drawn};

    #[derive(Default)]
    struct Record(Vec<String>);

    impl Screen for Record {
        type Font = Font;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, f: Font, t: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} n{t} {x:#x} {y:#x}"));
            Ok(Drawn {
                too_wide: t == 59,
                ..Drawn::default()
            })
        }
        fn store(&mut self, _: Buffer, _: &[u8]) -> Result<(), Invalid> {
            unreachable!()
        }
        fn draw(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<Drawn, Invalid> {
            unreachable!()
        }
        fn draw_time(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn draw_clock(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn string(&mut self, f: Font, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} s{id:x} {x:#x} {y:#x}"));
            Ok(Drawn::default())
        }
        fn string_bytes(&mut self, _: u32) -> Vec<u8> {
            unreachable!()
        }
        fn strings(&mut self, _: u32, _: u32) -> Result<Vec<u8>, Invalid> {
            unreachable!()
        }
        fn sprite(&mut self, _: u32, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn keep(&mut self, w: Work, v: u32) {
            self.0.push(format!("keep {w:?} {v:#x}"));
        }
        fn bar(&mut self, _: Bar) -> Result<(), Invalid> {
            unreachable!()
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
    }

    #[test]
    fn lists() {
        let mut s = Record::default();
        assert!(names(&mut s, &SAVED_GAMES, 5, |n| n).unwrap());
        assert_eq!(s.0.len(), 10);
        assert_eq!(s.0[0], "Names n50 0xcf 0xb5");
        assert_eq!(s.0[9], "Names n59 0x21c 0x129");
        assert!(names(&mut Record::default(), &SAVED_GAMES, 10, |n| n).is_err());
        let mut s = Record::default();
        setup_page(&mut s, LOAD_SETUP_TITLE, 2, |n| n).unwrap();
        assert_eq!(s.0[0], "Setups s4dd 0x190 0xe");
        assert_eq!(s.0[3], "Setups n60 0xe9 0x44");
        let mut s = Record::default();
        names(&mut s, &REPLAYS, 4, |n| n).unwrap();
        assert_eq!(s.0[0], "Names n80 0xc6 0xb5");
        assert!(names(&mut Record::default(), &REPLAYS, 5, |n| n).is_err());
    }
}
