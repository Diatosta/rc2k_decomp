//! The players' stage results pages, with and without the points column, drawn by one loop over
//! the rows each; their differences are the `Layout`. Slips kept as is: the
//! points version draws four rows for two players, tests the first row's time with a signed
//! comparison (the others with equality) and puts the third row's time at 0x1B2.

use crate::screens::best_scores::{car_glyph, flag_glyph};
use crate::screens::screen::{Buffer, Screen};
use crate::strings::format::{Pad, decimal, time};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Rows,
    /// Speeds and times.
    Small,
    /// The stage.
    Stage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// A time of 0x36EE80 hundredths (ten hours) or more: no time.
pub const NO_TIME: u32 = 0x36EE80;

/// What differs between the two pages.
pub struct Layout {
    /// Added to every column's x but the stage's and the "no time" string's.
    pub dx: i32,
    /// Two players give two rows or four (the points version).
    pub two_rows_for_two: bool,
    /// The first row's "no time" test: signed `>= NO_TIME` or `== NO_TIME`.
    pub first_time_signed: bool,
    /// The third row's time's x (0x1B2 in the points version, the column's 0x1E4 + dx in the
    /// other).
    pub third_time_x: i32,
    /// The head string 0x21A at 0x2BB and the points column.
    pub points: bool,
    /// After the third row's flag, the extra sprite there too (the version without points).
    pub third_row_extra: bool,
}

pub const WITH_POINTS: Layout = Layout {
    dx: 0,
    two_rows_for_two: false,
    first_time_signed: true,
    third_time_x: 0x1B2,
    points: true,
    third_row_extra: false,
};

pub const WITHOUT_POINTS: Layout = Layout {
    dx: 0x1E,
    two_rows_for_two: true,
    first_time_signed: false,
    third_time_x: 0x202,
    points: false,
    third_row_extra: true,
};

/// How many rows a page shows for `players` players.
pub fn rows(layout: &Layout, players: u32) -> usize {
    match players {
        3 => 3,
        2 if layout.two_rows_for_two => 2,
        _ => 4,
    }
}

/// A row's player.
#[derive(Clone, Debug)]
pub struct Row<T> {
    pub name: T,
    pub nation: u32,
    pub car: u32,
    /// Hundredths.
    pub time: u32,
    /// The top speed as formatted (no NUL).
    pub speed: Vec<u8>,
    /// The points (the page with points only).
    pub points: u32,
}

/// The page's other parts.
#[derive(Clone, Debug)]
pub struct Page {
    /// The section's title string (its first record's +0x38).
    pub section: u32,
    /// The stage line, with its NUL.
    pub stage: Vec<u8>,
    /// The units' head: string 0x574 when true, else 0x573.
    pub units_set: bool,
    pub show_hours: bool,
    /// The third row's extra sprite.
    pub extra_glyph: u32,
}

/// The page. Every sprite's palette is taken from where the previous drawing left off.
pub fn players_results<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    layout: &Layout,
    page: &Page,
    rows: &[Row<S::Text>],
) -> Result<(), S::Error> {
    let dx = layout.dx;
    let y = |row: usize| 0xFE + 0x17 * row as i32;
    let mut heads = vec![
        (0x218, 0x55 + dx),
        (0x219, 0x9C + dx),
        (0x21C, 0x176 + dx),
        (if page.units_set { 0x574 } else { 0x573 }, 0x25A + dx),
    ];
    if layout.points {
        heads.push((0x21A, 0x2BB));
    }
    for (id, x) in heads {
        s.string(Font::Rows, id, x, 0xE7)?;
    }
    s.string(Font::Stage, page.section, 0x2EB, 0x77)?;
    s.store(Buffer::Line, &page.stage)?;
    s.draw(Font::Stage, Buffer::Line, 0x2EB, 0x91)?;
    for (row, r) in rows.iter().enumerate() {
        s.text(Font::Rows, r.name, 0x9C + dx, y(row))?;
    }
    for (row, r) in rows.iter().enumerate() {
        s.spaced(Font::Small, &r.speed, 0x25A + dx, y(row))?;
    }
    for (row, r) in rows.iter().enumerate() {
        s.sprite(flag_glyph(r.nation), 0x7F + dx, y(row))?;
    }
    for (row, r) in rows.iter().enumerate() {
        let none = if row == 0 && layout.first_time_signed {
            r.time as i32 >= NO_TIME as i32
        } else {
            r.time == NO_TIME
        };
        if none {
            s.string(Font::Small, 0x4FA, 0x1B8, y(row))?;
        } else {
            let x = if row == 2 {
                layout.third_time_x
            } else {
                0x1E4 + dx
            };
            s.time_line(Font::Small, &time(r.time, page.show_hours), x, y(row))?;
        }
    }
    for (row, r) in rows.iter().enumerate() {
        s.sprite(car_glyph(r.car), 0x141 + dx, y(row))?;
        if row == 2 && layout.third_row_extra {
            s.sprite(page.extra_glyph, 0x141 + dx, y(row))?;
        }
    }
    let first = rows[0].time;
    for (row, r) in rows.iter().enumerate().skip(1) {
        if r.time == NO_TIME {
            continue;
        }
        let gap = r.time.wrapping_sub(first);
        s.string(Font::Rows, 0x579, 0x1DF + dx, y(row))?;
        s.time_line(Font::Small, &time(gap, page.show_hours), 0x257 + dx, y(row))?;
    }
    for row in 0..rows.len() {
        s.string(Font::Rows, 0x4FE + row as u32, 0x55 + dx, y(row))?;
    }
    if layout.points {
        for (row, r) in rows.iter().enumerate() {
            s.line(Font::Rows, &decimal(r.points, Pad::None), 0x2BB, y(row))?;
        }
    }
    Ok(())
}

/// A standings row's player.
#[derive(Clone, Copy, Debug)]
pub struct Standing<T> {
    pub name: T,
    pub nation: u32,
    pub car: u32,
    pub points: u32,
}

/// The players' standings: the heads, then per row the name, the nationality's and the car's
/// sprites, the place (strings 0x4FE ..) and the points; rows 0x17 apart from y 0xFE.
pub fn standings<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    rows: &[Standing<S::Text>],
) -> Result<(), S::Error> {
    for (id, x) in [(0x218u32, 0xEA), (0x219, 0x132), (0x21B, 0x217)] {
        s.string(Font::Rows, id, x, 0xE7)?;
    }
    let y = |row: usize| 0xFE + 0x17 * row as i32;
    for (row, r) in rows.iter().enumerate() {
        s.text(Font::Rows, r.name, 0x132, y(row))?;
    }
    for (row, r) in rows.iter().enumerate() {
        s.sprite(flag_glyph(r.nation), 0x114, y(row))?;
    }
    for (row, r) in rows.iter().enumerate() {
        s.sprite(car_glyph(r.car), 0x1DA, y(row))?;
    }
    for row in 0..rows.len() {
        s.string(Font::Rows, 0x4FE + row as u32, 0xEA, y(row))?;
    }
    for (row, r) in rows.iter().enumerate() {
        s.line(Font::Rows, &decimal(r.points, Pad::None), 0x217, y(row))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Invalid;
    use crate::screens::screen::{Bar, Drawn};

    #[derive(Default)]
    struct Record(Vec<String>, Vec<u8>);

    impl Screen for Record {
        type Font = Font;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, f: Font, t: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} n{t} {x:#x} {y:#x}"));
            Ok(Drawn::default())
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
            Ok(Drawn::default())
        }
        fn draw_time(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!(
                "time {f:?} {:?} {x:#x} {y:#x}",
                String::from_utf8_lossy(&self.1)
            ));
            Ok(())
        }
        fn draw_clock(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn string(&mut self, f: Font, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} s{id:x} {x:#x} {y:#x}"));
            Ok(Drawn { ..Drawn::default() })
        }
        fn string_bytes(&mut self, _: u32) -> Vec<u8> {
            unreachable!()
        }
        fn strings(&mut self, _: u32, _: u32) -> Result<Vec<u8>, Invalid> {
            unreachable!()
        }
        fn sprite(&mut self, g: u32, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("sprite {g:#x} {x:#x} {y:#x}"));
            Ok(())
        }
        fn keep(&mut self, w: Work, v: u32) {
            self.0.push(format!("keep {w:?} {v}"));
        }
        fn bar(&mut self, _: Bar) -> Result<(), Invalid> {
            unreachable!()
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
        fn spaced(&mut self, f: Font, b: &[u8], x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!(
                "spaced {f:?} {:?} {x:#x} {y:#x}",
                String::from_utf8_lossy(b)
            ));
            Ok(())
        }
    }

    #[test]
    fn pages() {
        assert_eq!(rows(&WITH_POINTS, 2), 4);
        assert_eq!(rows(&WITHOUT_POINTS, 2), 2);
        let row = |n: u32, time: u32| Row {
            name: n,
            nation: n,
            car: n,
            time,
            speed: b"123".to_vec(),
            points: 10 - n,
        };
        let rows = [row(0, 1000), row(1, NO_TIME), row(2, 1500)];
        let page = Page {
            section: 0x300,
            stage: b"stage\0".to_vec(),
            units_set: false,
            show_hours: false,
            extra_glyph: 0x77,
        };
        let mut s = Record::default();
        players_results(&mut s, &WITHOUT_POINTS, &page, &rows).unwrap();
        assert!(s.0.contains(&"sprite 0x1c 0x9d 0xfe".to_string()));
        assert!(s.0.contains(&"Small s4fa 0x1b8 0x115".to_string()));
        // The third row's time last: its palette is the line buffer.
        assert!(s.0.contains(&"sprite 0x46 0x15f 0x12c".to_string()));
        assert!(s.0.contains(&"sprite 0x77 0x15f 0x12c".to_string()));
        assert!(!s.0.iter().any(|l| l.contains("0x2bb")));
        let mut s = Record::default();
        players_results(&mut s, &WITH_POINTS, &page, &rows).unwrap();
        assert!(
            s.0.contains(&"time Small \"00:15.00\\0\" 0x1b2 0x12c".to_string()),
            "{:?}",
            s.0
        );
        assert_eq!(s.0.last().unwrap(), "Rows \"8\\0\" 0x2bb 0x12c");
    }
}
