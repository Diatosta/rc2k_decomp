//! The championship's four results pages (converted 2026-10-20):
//! `screens::draw_class_rally_results` 0x4A3376, `draw_overall_rally_results` 0x4A37E2,
//! `draw_class_stage_results` 0x4A2111, `draw_overall_stage_results` 0x4A2F46: a heading,
//! the column headings and ten rows of the ranked times. `screens::draw_class_ranking`
//! 0x47C16A's rows found in its list converted 2026-10-21 (`class_rows`).

use crate::Invalid;
use crate::screens::best_scores::{car_glyph, flag_glyph};
use crate::screens::screen::Screen;
use crate::strings::format::{Pad, decimal, time};

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
    let mut y = 0xF0i32;
    for (n, r) in rows.iter().enumerate() {
        let (place, name, time_font) = if r.player {
            (Font::PlayerPlace, Font::PlayerName, Font::PlayerTime)
        } else {
            (Font::Place, Font::Name, Font::Time)
        };
        y = s.line(place, &decimal(r.position, Pad::None), 0xBE, y)?.y;
        y = s.text(name, r.name, 0xFA, y)?.y;
        if !r.finished {
            let id = if r.not_timed { 0x4FB } else { 0x4FA };
            y = s.string(name, id, 0x1EF, y)?.y;
        } else {
            s.time_line(time_font, &time(r.time, show_hours), 0x258, y)?;
            if r.position != 1 {
                let gap = r.time.wrapping_sub(first);
                s.keep(Work::Gap, gap);
                y = s.string(place, 0x579, 0x276, y)?.y;
                s.time_line(time_font, &time(gap, show_hours), 0x2E4, y)?;
            }
        }
        s.sprite(flag_glyph(r.nation), 0xD2, y)?;
        s.sprite(car_glyph(r.car), 0x1AE, y)?;
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

/// `class_rows`' answer: each row's entry and place, and the place left after the last.
pub type ClassRows = (Vec<(u32, u32)>, Option<u32>);

/// The most entries `class_rows` walks before it gives up on the list's end.
pub const MOST_ENTRIES: u32 = 0x400;

/// The rows `screens::draw_class_ranking` draws: from the list's first entry (n 0) on, the
/// class's `first`-th entry and up to nine more of the class, each with its place; and
/// the place left when the list ends while the next row is looked for (the loop writes it
/// before it looks). `class_of(n)` is entry n's class (+0x1C), `ends(n)` whether entry n
/// is the end mark (its +0 -1), each asked where the original reads. Refuses a list
/// without its end in `MOST_ENTRIES`, a page that starts past the list (the original reads
/// on) and a row just before the end mark (the original's stack slips).
pub fn class_rows(
    class_of: impl Fn(u32) -> u32,
    ends: impl Fn(u32) -> bool,
    class: u32,
    first: u32,
) -> Result<ClassRows, Invalid> {
    let mut n = 0u32;
    let mut steps = 0;
    let mut step = |n: &mut u32| -> Result<bool, Invalid> {
        steps += 1;
        if steps > MOST_ENTRIES {
            return Err(Invalid("a list without its end in 0x400 entries"));
        }
        *n += 1;
        Ok(ends(*n))
    };
    let mut seen = 0u32;
    loop {
        if class_of(n) == class {
            seen = seen.wrapping_add(1);
            if seen == first {
                break;
            }
        }
        if step(&mut n)? {
            return Err(Invalid(
                "a page that starts past the list (the original reads on)",
            ));
        }
    }
    let mut out = Vec::new();
    let mut place = first;
    loop {
        while class_of(n) != class {
            if step(&mut n)? {
                return Ok((out, Some(place)));
            }
        }
        out.push((n, place));
        if step(&mut n)? {
            return Err(Invalid(
                "a row just before the end mark (the original's stack slip)",
            ));
        }
        place = place.wrapping_add(1);
        if place == first.wrapping_add(10) {
            return Ok((out, None));
        }
    }
}

/// `screens::draw_class_ranking`: a page of one class's times, the rows found by the
/// caller; `after` the place `seconds` is left at when the list ended while the next row
/// was looked for. The sprites 4 below the row.
pub fn class_ranking<S: Screen<Font = Font, Work = RankingWork>>(
    s: &mut S,
    first: u32,
    show_hours: bool,
    rows: &[RankedRow<S::Text>],
    after: Option<u32>,
) -> Result<(), S::Error> {
    s.keep(RankingWork::FirstTime, first);
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
        y = s.line(place, &decimal(r.place, Pad::None), 0xBE, y)?.y;
        if r.place != 1 {
            let gap = r.time.wrapping_sub(first);
            s.keep(RankingWork::Gap, gap);
            y = s.string(place, 0x579, 0x276, y)?.y;
            s.time_line(time_font, &time(gap, show_hours), 0x2E4, y)?;
        }
        let sprite_y = y.wrapping_add(4);
        s.keep(RankingWork::SpriteY, sprite_y as u32);
        s.sprite(flag_glyph(r.nation), 0xD2, sprite_y)?;
        s.sprite(car_glyph(r.car), 0x1AE, sprite_y)?;
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
    s.string(Font::Name, 0x4F9, 0x2DE, 0xDC)?;
    let first_round = if mode_13 { 0x90 } else { 0x8A };
    for n in 0..6u32 {
        s.sprite(first_round + n, 0x196 + 0x36 * n as i32, 0x8C)?;
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
        s.sprite(flag_glyph(r.nation), 0x7E, y)?;
        s.sprite(car_glyph(r.car), 0x154, y)?;
        y = y.wrapping_add(0x19);
        s.keep(Work::Row, first_row.wrapping_add(n as u32 + 1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Invalid;
    use crate::screens::screen::Buffer;
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
                ..Drawn::default()
            })
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
        assert!(s.0.contains(&"sprite 0x5d 0x1ae 0xf0".to_string()));
        assert!(s.0.contains(&"Name s4fb 0x1ef 0x122".to_string()));
        assert!(s.0.contains(&"sprite 0x1e 0xd2 0x122".to_string()));
        assert_eq!(s.0.last().unwrap(), "keep Row 6");
    }

    #[test]
    fn class_list() {
        // Classes of entries 0 .. 7, then the end mark at 8.
        let classes = [1, 2, 1, 1, 2, 1, 2, 2];
        let class_of = |n: u32| classes.get(n as usize).copied().unwrap_or(9);
        let ends = |n: u32| n as usize >= classes.len();
        assert_eq!(
            class_rows(class_of, ends, 1, 2),
            Ok((vec![(2, 2), (3, 3), (5, 4)], Some(5)))
        );
        assert_eq!(
            class_rows(class_of, ends, 2, 1),
            Err(Invalid(
                "a row just before the end mark (the original's stack slip)"
            ))
        );
        assert!(class_rows(class_of, ends, 1, 5).is_err());
        let all = |_| 1;
        let (rows, after) = class_rows(all, |_| false, 1, 1).unwrap();
        assert_eq!((rows.len(), after), (10, None));
    }
}
