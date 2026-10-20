//! The championship's four results pages (converted 2026-10-20):
//! `screens::draw_class_rally_results` 0x4A3376, `draw_overall_rally_results` 0x4A37E2,
//! `draw_class_stage_results` 0x4A2111, `draw_overall_stage_results` 0x4A2F46: a heading,
//! the column headings and ten rows of the ranked times.

use crate::best_scores::{car_glyph, flag_glyph};
use crate::format::{Pad, decimal, time};
use crate::screen::{Buffer, Screen};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E944`: the title.
    Title,
    /// `stru_51E9A4`: the stage's name.
    StageName,
    /// `stru_51E5A4`, `stru_51E584`, `stru_51E564`: position, name, time.
    Place,
    Name,
    Time,
    /// `stru_51E544`, `stru_51E524`, `stru_51E504`: a player's row's.
    PlayerPlace,
    PlayerName,
    PlayerTime,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `g_firstPlayerTime`.
    FirstTime,
    /// `dword_7559D4`: the overall stage page's first time.
    OverallFirst,
    /// `hundredths`: a row's gap.
    Gap,
    /// `g_champRow`.
    Row,
    /// `g_champRowEnd`.
    RowEnd,
}

/// A time of 0x36EE8A: not timed (string 0x4FB in place of 0x4FA).
pub const NOT_TIMED: u32 = 0x36EE8A;

/// The heading.
#[derive(Clone, Copy, Debug)]
pub enum Heading {
    /// The title string at (0x310, 0x30) and the stage's name at (0x2EB, 0x77).
    Titles { title: u32, name: u32 },
    /// Drawn by the caller already (`screens::draw_record_heading`).
    Drawn,
}

/// A row: an entry of the ranked times.
#[derive(Clone, Copy, Debug)]
pub struct Row<T> {
    /// A player's (+4 above 0): the highlight fonts.
    pub player: bool,
    /// +0x44.
    pub position: u32,
    /// +0x20.
    pub name: T,
    /// +0x4C is 1.
    pub finished: bool,
    /// The page's "not timed" field is `NOT_TIMED`.
    pub not_timed: bool,
    /// The page's time (+0x18 the rally's, +0x14 the stage's).
    pub time: u32,
    /// +8.
    pub nation: u32,
    /// +0x10.
    pub car: u32,
}

/// One page: `first` the time the gaps are from, `first_row` the list's row the page starts
/// at, `keep_overall` the first time kept in `dword_7559D4` too. The sprites' palette is
/// where the last text left ebx: the line buffer after a time, the string's end else.
#[allow(clippy::too_many_arguments)] // one page's parts, as the original reads them
pub fn results_page<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    heading: Heading,
    first: u32,
    first_row: u32,
    keep_overall: bool,
    show_hours: bool,
    rows: &[Row<S::Text>],
) -> Result<(), S::Error> {
    if let Heading::Titles { title, name } = heading {
        s.string(Font::Title, title, 0x310, 0x30)?;
        s.string(Font::StageName, name, 0x2EB, 0x77)?;
    }
    s.string(Font::Place, 0x484, 0xC8, 0xDC)?;
    s.string(Font::Name, 0x485, 0xFA, 0xDC)?;
    s.string(Font::Name, 0x47F, 0x1F4, 0xDC)?;
    s.keep(Work::FirstTime, first);
    if keep_overall {
        s.keep(Work::OverallFirst, first);
    }
    s.keep(Work::Row, first_row);
    s.keep(Work::RowEnd, first_row.wrapping_add(10));
    let line = s.buffer_address(Buffer::Line);
    let mut y = 0xF0i32;
    for (n, r) in rows.iter().enumerate() {
        let (place, name, time_font) = if r.player {
            (Font::PlayerPlace, Font::PlayerName, Font::PlayerTime)
        } else {
            (Font::Place, Font::Name, Font::Time)
        };
        y = s.line(place, &decimal(r.position, Pad::None), 0xBE, y)?.y;
        y = s.text(name, r.name, 0xFA, y)?.y;
        let palette = if !r.finished {
            let id = if r.not_timed { 0x4FB } else { 0x4FA };
            let o = s.string(name, id, 0x1EF, y)?;
            y = o.y;
            o.after
        } else {
            s.time_line(time_font, &time(r.time, show_hours), 0x258, y)?;
            if r.position != 1 {
                let gap = r.time.wrapping_sub(first);
                s.keep(Work::Gap, gap);
                y = s.string(place, 0x579, 0x276, y)?.y;
                s.time_line(time_font, &time(gap, show_hours), 0x2E4, y)?;
            }
            line
        };
        s.sprite(flag_glyph(r.nation), 0xD2, y, palette)?;
        s.sprite(car_glyph(r.car), 0x1AE, y, palette)?;
        y = y.wrapping_add(0x19);
        s.keep(Work::Row, first_row.wrapping_add(n as u32 + 1));
    }
    Ok(())
}

/// The work variables of `screens::draw_class_ranking` 0x47C16A.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RankingWork {
    /// `g_firstPlayerTime`.
    FirstTime,
    /// `seconds` (0x7559AC): the row's place.
    Place,
    /// `glyph` (0x7559C0): the row's gap.
    Gap,
    /// `dword_7559B0`: the sprites' y.
    SpriteY,
}

/// A class ranking row's name: a string (a driver of the game's, kind 0) or a text.
#[derive(Clone, Copy, Debug)]
pub enum Name<T> {
    String(u32),
    Text(T),
}

/// A class ranking row.
#[derive(Clone, Copy, Debug)]
pub struct RankedRow<T> {
    pub place: u32,
    /// The player `dword_7556C0` (the highlight fonts).
    pub mine: bool,
    pub name: Name<T>,
    /// +0x18.
    pub time: u32,
    pub nation: u32,
    pub car: u32,
}

/// `screens::draw_class_ranking`: a page of one class's times, the rows found by the
/// caller; `after` the place `seconds` is left at when the list ended while the next row
/// was looked for. The sprites' palette is where the last text left ebx (the place's text,
/// or the line buffer after a gap), the sprites 4 below the row.
pub fn class_ranking<S: Screen<Font = Font, Work = RankingWork>>(
    s: &mut S,
    first: u32,
    show_hours: bool,
    rows: &[RankedRow<S::Text>],
    after: Option<u32>,
) -> Result<(), S::Error> {
    s.keep(RankingWork::FirstTime, first);
    let line = s.buffer_address(Buffer::Line);
    let mut y = 0xF0i32;
    for (k, r) in rows.iter().enumerate() {
        if k > 0 {
            y = y.wrapping_add(0x19);
        }
        s.keep(RankingWork::Place, r.place);
        let (name, time_font, place) = if r.mine {
            (Font::PlayerName, Font::PlayerTime, Font::PlayerPlace)
        } else {
            (Font::Name, Font::Time, Font::Place)
        };
        y = match r.name {
            Name::String(id) => s.string(name, id, 0xFA, y)?,
            Name::Text(t) => s.text(name, t, 0xFA, y)?,
        }
        .y;
        s.time_line(time_font, &time(r.time, show_hours), 0x258, y)?;
        let o = s.line(place, &decimal(r.place, Pad::None), 0xBE, y)?;
        y = o.y;
        let mut palette = o.after;
        if r.place != 1 {
            let gap = r.time.wrapping_sub(first);
            s.keep(RankingWork::Gap, gap);
            y = s.string(place, 0x579, 0x276, y)?.y;
            s.time_line(time_font, &time(gap, show_hours), 0x2E4, y)?;
            palette = line;
        }
        let sprite_y = y.wrapping_add(4);
        s.keep(RankingWork::SpriteY, sprite_y as u32);
        s.sprite(flag_glyph(r.nation), 0xD2, sprite_y, palette)?;
        s.sprite(car_glyph(r.car), 0x1AE, sprite_y, palette)?;
    }
    if let Some(place) = after {
        s.keep(RankingWork::Place, place);
    }
    Ok(())
}

/// A championship standings row: an entry of the history.
#[derive(Clone, Copy, Debug)]
pub struct StandingRow<T> {
    /// +4 above 0: the highlight fonts.
    pub player: bool,
    /// +0x70.
    pub position: u32,
    /// +0x14.
    pub name: T,
    /// +0x38 .. +0x50: the six rounds' points and the total (below 0: "-").
    pub points: [u32; 7],
    /// +8.
    pub nation: u32,
    /// +0xC.
    pub car: u32,
}

/// `screens::draw_championship_standings` 0x4A5410: the column headings, the six rounds'
/// sprites (0x90 .. in event mode 0x13, else 0x8A ..; the palette where the last heading
/// ended), then the rows from y 0xF0, 0x19 apart; each row's sprites with the palette where
/// its total ended.
pub fn championship_standings<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    first_row: u32,
    mode_13: bool,
    rows: &[StandingRow<S::Text>],
) -> Result<(), S::Error> {
    s.string(Font::Place, 0x484, 0x73, 0xDC)?;
    s.string(Font::Name, 0x485, 0x9C, 0xDC)?;
    let o = s.string(Font::Name, 0x4F9, 0x2DE, 0xDC)?;
    let first_round = if mode_13 { 0x90 } else { 0x8A };
    for n in 0..6u32 {
        s.sprite(first_round + n, 0x196 + 0x36 * n as i32, 0x8C, o.after)?;
    }
    s.keep(Work::Row, first_row);
    s.keep(Work::RowEnd, first_row.wrapping_add(10));
    let mut y = 0xF0i32;
    for (n, r) in rows.iter().enumerate() {
        let (place, font) = if r.player {
            (Font::PlayerPlace, Font::PlayerName)
        } else {
            (Font::Place, Font::Name)
        };
        y = s.line(place, &decimal(r.position, Pad::None), 0x73, y)?.y;
        let mut o = s.text(font, r.name, 0x9C, y)?;
        y = o.y;
        for (k, &v) in r.points.iter().enumerate() {
            let text = if (v as i32) < 0 {
                b"-".to_vec()
            } else {
                decimal(v, Pad::None)
            };
            o = s.line(font, &text, 0x19A + 0x36 * k as i32, y)?;
            y = o.y;
        }
        s.sprite(flag_glyph(r.nation), 0x7E, y, o.after)?;
        s.sprite(car_glyph(r.car), 0x154, y, o.after)?;
        y = y.wrapping_add(0x19);
        s.keep(Work::Row, first_row.wrapping_add(n as u32 + 1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Invalid;
    use crate::screen::{Bar, Drawn};

    #[derive(Default)]
    struct Record(Vec<String>, Vec<u8>);

    impl Screen for Record {
        type Font = Font;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, f: Font, t: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} n{t} {x:#x} {y:#x}"));
            Ok(Drawn {
                y,
                ..Drawn::default()
            })
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
            Ok(Drawn {
                y,
                ..Drawn::default()
            })
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
            Ok(Drawn {
                y,
                after: 0x900 + id,
                ..Drawn::default()
            })
        }
        fn string_bytes(&mut self, _: u32) -> Vec<u8> {
            unreachable!()
        }
        fn strings(&mut self, _: u32, _: u32) -> Result<Vec<u8>, Invalid> {
            unreachable!()
        }
        fn sprite(&mut self, g: u32, x: i32, y: i32, p: u32) -> Result<(), Invalid> {
            self.0.push(format!("sprite {g:#x} {x:#x} {y:#x} {p:#x}"));
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
        fn buffer_address(&mut self, _: Buffer) -> u32 {
            0x1000
        }
    }

    #[test]
    fn page() {
        let row = |n: u32, player: bool, finished: bool| Row {
            player,
            position: n + 1,
            name: n,
            finished,
            not_timed: !finished && n == 2,
            time: 1000 + 100 * n,
            nation: n,
            car: 0x18,
        };
        let rows = [
            row(0, false, true),
            row(1, true, true),
            row(2, false, false),
        ];
        let mut s = Record::default();
        let heading = Heading::Titles {
            title: 0x4EA,
            name: 0x300,
        };
        results_page(&mut s, heading, 1000, 3, true, false, &rows).unwrap();
        assert_eq!(s.0[0], "Title s4ea 0x310 0x30");
        assert!(s.0.contains(&"keep OverallFirst 1000".to_string()));
        assert!(s.0.contains(&"keep RowEnd 13".to_string()));
        assert!(s.0.contains(&"PlayerPlace s579 0x276 0x109".to_string()));
        assert!(s.0.contains(&"keep Gap 100".to_string()));
        assert!(s.0.contains(&"sprite 0x5d 0x1ae 0xf0 0x1000".to_string()));
        assert!(s.0.contains(&"Name s4fb 0x1ef 0x122".to_string()));
        assert!(s.0.contains(&"sprite 0x1e 0xd2 0x122 0xdfb".to_string()));
        assert_eq!(s.0.last().unwrap(), "keep Row 6");
    }
}
