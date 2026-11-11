//! The load-game screen's preview of the highlighted saved game:
//! `screens::draw_empty_savegame` 0x46E2DE and `draw_savegame_details` 0x46D907, and the
//! frame both start with (`screens::savegame_preview`): three headings, the row numbers
//! 1 .. 4, a title line and a row per car.

use crate::screens::best_scores::{car_glyph, flag_glyph};
use crate::screens::screen::{Drawn, Screen};
use crate::strings::format::{Pad, decimal, fraction};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `FontObjects.objects[15].first_char`: the headings and the rows.
    Rows,
    /// `FontObjects.objects[18].first_char`: the title line.
    Title,
    /// `FontObjects.objects[16].first_char`: the row numbers.
    RowNumber,
}

/// The preview keeps nothing in work variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

const HEADINGS: [(u32, i32); 3] = [(0x4E5, 0x8F), (0x4E6, 0x1D0), (0x4E7, 0x271)];
const HEADINGS_Y: i32 = 0x17A;
const TITLE_Y: i32 = 0x161;
const LEFT_X: i32 = 0x8F;
/// The four rows.
const ROWS: [i32; 4] = [0x193, 0x1AC, 0x1C5, 0x1DE];
/// The four number columns of a row.
const NUMBER_COLUMNS: [i32; 4] = [0x1D0, 0x215, 0x271, 0x2B6];
const NAME_X: i32 = 0xD6;
const FLAG_X: i32 = 0xAF;
const CAR_X: i32 = 0x18B;
/// Where "nothing" is written in place of the two sprites.
const NO_FLAG_X: i32 = 0xB8;
const NO_CAR_X: i32 = 0x1A1;
/// "-": where there is no value.
const NOTHING: u32 = 0x57A;
/// "n/d": where there is no name.
const NO_ENTRY: u32 = 0x578;

/// The headings and the row numbers, which both panels start with.
fn frame<S: Screen<Font = Font, Work = Work>>(s: &mut S) -> Result<(), S::Error> {
    for (id, x) in HEADINGS {
        s.string(Font::Rows, id, x, HEADINGS_Y)?;
    }
    for (n, y) in ROWS.into_iter().enumerate() {
        s.line(
            Font::RowNumber,
            &decimal(n as u32 + 1, Pad::None),
            LEFT_X,
            y,
        )?;
    }
    Ok(())
}

/// The panel for a slot with no saved game: "n/d" as the title and every row's name, "-"
/// in the six other places of every row. The carry of the last text.
pub fn empty<S: Screen<Font = Font, Work = Work>>(s: &mut S) -> Result<bool, S::Error> {
    frame(s)?;
    let mut last = s.string(Font::Title, NO_ENTRY, LEFT_X, TITLE_Y)?;
    for y in ROWS {
        s.string(Font::Rows, NO_ENTRY, NAME_X, y)?;
        for x in [NO_FLAG_X, NO_CAR_X].into_iter().chain(NUMBER_COLUMNS) {
            last = s.string(Font::Rows, NOTHING, x, y)?;
        }
    }
    Ok(last.too_wide)
}

/// A saved car as the preview shows it.
#[derive(Clone, Copy, Debug)]
pub struct Car<T> {
    /// Its name (car +0x124).
    pub name: T,
    pub nation: u32,
    pub car: u32,
    /// The four numbers: +0x14C, +0x150, +0x144, +0x148.
    pub numbers: [u32; 4],
    /// Whether the last two are shown (car +0xB0, +0xCC not 0); "-" in their place if not.
    pub shown: [bool; 2],
}

/// The title line's parts.
#[derive(Clone, Copy, Debug)]
pub struct Title {
    /// The saved event mode is 0x13, the A8 rally championship's (string 0x1DA, else 0x1D9: the Mobil One championship).
    pub a8_mode: bool,
    /// The saved section's title string.
    pub section: u32,
    /// The saved stage's number (its record's +0x40): "n/6" after the title unless 0.
    pub stage: u32,
}

/// A number as the rows show it: "-" for a negative one.
fn number<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    value: u32,
    x: i32,
    y: i32,
) -> Result<Drawn, S::Error> {
    let text = if (value as i32) < 0 {
        vec![b'-']
    } else {
        decimal(value, Pad::None)
    };
    s.line(Font::Rows, &text, x, y)
}

/// The panel of a saved game of `players` players: the title line and a row per car. The
/// first row draws its sprites after its numbers (their palette what the last number left);
/// the others after the name (its leftover), or "n/d" and "-" for a row past the players.
/// The carry of the last text.
pub fn details<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    title: Title,
    players: i32,
    cars: &[Car<S::Text>; 4],
) -> Result<bool, S::Error> {
    frame(s)?;
    let mut line = s.string_bytes(if title.a8_mode { 0x1DA } else { 0x1D9 });
    line.extend_from_slice(b" - ");
    line.extend(s.string_bytes(title.section));
    line.push(b' ');
    if title.stage != 0 {
        line.extend(fraction(title.stage, 6));
    }
    s.line(Font::Title, &line, LEFT_X, TITLE_Y)?;
    let mut last = Drawn::default();
    for (row, (y, car)) in ROWS.into_iter().zip(cars).enumerate() {
        let sprites = |s: &mut S| -> Result<(), S::Error> {
            s.sprite(flag_glyph(car.nation), FLAG_X, y)?;
            s.sprite(car_glyph(car.car), CAR_X, y)
        };
        let numbers = |s: &mut S| -> Result<Drawn, S::Error> {
            number(s, car.numbers[0], NUMBER_COLUMNS[0], y)?;
            number(s, car.numbers[1], NUMBER_COLUMNS[1], y)?;
            let mut out = Drawn::default();
            for n in 0..2 {
                let x = NUMBER_COLUMNS[2 + n];
                out = if car.shown[n] {
                    number(s, car.numbers[2 + n], x, y)?
                } else {
                    s.string(Font::Rows, NOTHING, x, y)?
                };
            }
            Ok(out)
        };
        if row == 0 {
            s.text(Font::Rows, car.name, NAME_X, y)?;
            last = numbers(s)?;
            sprites(s)?;
        } else {
            if players > row as i32 {
                s.text(Font::Rows, car.name, NAME_X, y)?;
                sprites(s)?;
            } else {
                s.string(Font::Rows, NO_ENTRY, NAME_X, y)?;
                s.string(Font::Rows, NOTHING, NO_FLAG_X, y)?;
                s.string(Font::Rows, NOTHING, NO_CAR_X, y)?;
            }
            last = numbers(s)?;
        }
    }
    Ok(last.too_wide)
}

/// What the load page's preview (`screens::draw_savegame_preview` 0x46D88F) shows for the
/// highlighted item: above 10 no slot (the empty panel; the slot remembered as 999), else
/// slot page x 10 + item - 1, read when it is not the one remembered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preview {
    NoSlot,
    Slot { slot: u32, read: bool },
}

/// The slot remembered when there is none.
pub const NO_SLOT: u32 = 999;

pub fn preview(item: u32, page: u32, shown: u32) -> Preview {
    if (item as i32) > 10 {
        return Preview::NoSlot;
    }
    let slot = page.wrapping_mul(10).wrapping_add(item).wrapping_sub(1);
    Preview::Slot {
        slot,
        read: shown != slot,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Invalid;
    use crate::screens::screen::{Bar, Buffer};

    #[derive(Default)]
    struct Record(Vec<String>, Vec<u8>);

    impl Screen for Record {
        type Font = Font;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, f: Font, t: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} n{t} {x:#x} {y:#x}"));
            Ok(Drawn { ..Drawn::default() })
        }
        fn store(&mut self, _: Buffer, b: &[u8]) -> Result<(), Invalid> {
            self.1 = b.to_vec();
            Ok(())
        }
        fn draw(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!(
                "{f:?} {:?} {x:#x} {y:#x}",
                String::from_utf8_lossy(&self.1)
            ));
            Ok(Drawn { ..Drawn::default() })
        }
        fn draw_time(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn draw_clock(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn string(&mut self, f: Font, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} s{id:x} {x:#x} {y:#x}"));
            Ok(Drawn {
                too_wide: id == NOTHING,
                ..Drawn::default()
            })
        }
        fn string_bytes(&mut self, id: u32) -> Vec<u8> {
            format!("<{id:x}>").into_bytes()
        }
        fn strings(&mut self, _: u32, _: u32) -> Result<Vec<u8>, Invalid> {
            unreachable!()
        }
        fn sprite(&mut self, g: u32, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("sprite {g:#x} {x:#x} {y:#x}"));
            Ok(())
        }
        fn keep(&mut self, _: Work, _: u32) {
            unreachable!()
        }
        fn bar(&mut self, _: Bar) -> Result<(), Invalid> {
            unreachable!()
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
    }

    #[test]
    fn panels() {
        let mut s = Record::default();
        assert!(empty(&mut s).unwrap());
        assert_eq!(s.0.len(), 3 + 4 + 1 + 4 * 7);
        assert_eq!(s.0[3], "RowNumber \"1\\0\" 0x8f 0x193");
        let car = |n: u32| Car {
            name: n,
            nation: n,
            car: 0x18,
            numbers: [1, u32::MAX, 3, 4],
            shown: [true, n != 1],
        };
        let cars = [car(0), car(1), car(2), car(3)];
        let mut s = Record::default();
        let title = Title {
            a8_mode: true,
            section: 0x300,
            stage: 2,
        };
        assert!(!details(&mut s, title, 2, &cars).unwrap());
        assert_eq!(s.0[7], "Title \"<1da> - <300> 2/6\\0\" 0x8f 0x161");
        assert!(s.0.contains(&"sprite 0x1c 0xaf 0x193".to_string()));
        assert!(s.0.contains(&"sprite 0x5d 0x18b 0x193".to_string()));
        assert!(s.0.contains(&"sprite 0x1d 0xaf 0x1ac".to_string()));
        assert!(s.0.contains(&"Rows \"-\\0\" 0x215 0x1ac".to_string()));
        assert!(s.0.contains(&"Rows s57a 0x2b6 0x1ac".to_string()));
        assert!(s.0.contains(&"Rows s578 0xd6 0x1c5".to_string()));
    }
}
