//! The best scores, class times, stage times and record pages (converted 2026-10-20 from
//! `screens::draw_best_scores` 0x4A9300, `draw_class_times` 0x46D162,
//! `draw_class_times_marked` 0x48AB87, `draw_stage_times` 0x48D0F2,
//! `draw_rally_stage_times` 0x48A948, the headings, `draw_record_times` 0x4A41AE and
//! `draw_event_best_scores` / `draw_overall_best_scores`): ten ranked rows of a name, a
//! figure, the rank, the nationality's flag sprite and the car's.

use crate::screens::screen::Screen;
use crate::strings::format::{Pad, decimal, time};

/// The fonts (the highlight ones for the player's own rows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Name,
    Rank,
    Time,
    MarkedName,
    MarkedRank,
    MarkedTime,
    /// The page's heading and stage line.
    Heading,
    /// The fastest line's string.
    Fastest,
}

/// The work variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// The rank being drawn.
    Rank,
    /// The row's sprites' y.
    RowY,
    /// The last car's sprite.
    CarGlyph,
    /// The event heading's string.
    Heading,
}

/// A row: its name (where the game keeps it), its figure (a score or a time in
/// hundredths), the nationality and car, and whether it is the player's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row<T> {
    pub name: T,
    pub figure: u32,
    pub nation: u32,
    pub car: u32,
    pub marked: bool,
}

/// The rows of a list: row `n` (from 0), and whether the list ends at row `n` (its first
/// dword is -1; row 0 is never asked).
pub struct Rows<'a, T> {
    pub row: &'a dyn Fn(u32) -> Row<T>,
    pub ends_at: &'a dyn Fn(u32) -> bool,
}

/// What a list keeps of each row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    Nothing,
    /// The row's y.
    Y,
    /// The sprites' y, 4 below the row, and the car's sprite.
    Below4,
}

/// A nationality's flag sprite.
pub fn flag_glyph(nation: u32) -> u32 {
    0x1C_u32.wrapping_add(nation)
}

/// A car's sprite (cars 0x18 and 0x19 swapped).
pub fn car_glyph(car: u32) -> u32 {
    let car = if (0x18..=0x19).contains(&car) {
        car ^ 1
    } else {
        car
    };
    car.wrapping_add(0x44)
}

fn fonts(marked: bool) -> (Font, Font, Font) {
    if marked {
        (Font::MarkedName, Font::MarkedTime, Font::MarkedRank)
    } else {
        (Font::Name, Font::Time, Font::Rank)
    }
}

/// A row of a section's best times (an entry of 0x9C bytes): the name (+0x20), the time
/// (+0x18), the nationality (+8), the car (+0x10), a player's (+0x40 is 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionRow<T> {
    pub name: T,
    pub time: u32,
    pub nation: u32,
    pub car: u32,
    pub player: bool,
}

/// A section's best times (`screens::draw_section_best_times` 0x46762B): ten rows from y
/// 0xF0, 0x19 apart, row n kept: the name at 0xF0, the time at 0x1E0, n at 0xC8, each from
/// the last text's y; then the flag's sprite at 0xD2 and the car's at 0x276, and again at
/// 0x276 the sprite of the car's number swapped once more and 0x44 on it (sic,
/// `notes/bugs.md`), their palette where the number's text ended.
pub fn section_best_times<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    rows: &[SectionRow<S::Text>; 10],
    show_hours: bool,
) -> Result<(), S::Error> {
    let mut y = 0xF0;
    for (n, r) in (1u32..).zip(rows) {
        s.keep(Work::Rank, n);
        let (font, place_font) = if r.player {
            (Font::MarkedName, Font::MarkedRank)
        } else {
            (Font::Name, Font::Rank)
        };
        y = s.text(font, r.name, 0xF0, y)?.y;
        y = s.line(font, &time(r.time, show_hours), 0x1E0, y)?.y;
        let place = s.line(place_font, &decimal(n, Pad::None), 0xC8, y)?;
        y = place.y;
        s.sprite(flag_glyph(r.nation), 0xD2, y)?;
        let car = car_glyph(r.car);
        s.sprite(car, 0x276, y)?;
        y = y.wrapping_add(0x19);
    }
    Ok(())
}

/// A score list (`draw_scores`): from rank 1 until ten or a list's end: the name (x 0xF0),
/// the score (x 0x21C), the rank (x 0xC8), each from the last text's y; the flag's sprite
/// (x 0xD2) and the car's (x 0x1CC) `sprite_dy` below, their palette where the rank's
/// text ended; rows 0x19 apart. With `keeps_y` the sprites' y is kept.
pub fn scores<S, T>(
    s: &mut S,
    rows: &Rows<T>,
    keeps_y: bool,
    sprite_dy: i32,
) -> Result<(), S::Error>
where
    S: Screen<Font = Font, Text = T, Work = Work>,
    T: Copy,
{
    let mut y = 0xF0;
    for rank in 1..=10u32 {
        let n = rank - 1;
        if n > 0 && (rows.ends_at)(n) {
            break;
        }
        s.keep(Work::Rank, rank);
        let row = (rows.row)(n);
        let (name_font, _, rank_font) = fonts(row.marked);
        y = s.text(name_font, row.name, 0xF0, y)?.y;
        y = s
            .line(name_font, &decimal(row.figure, Pad::None), 0x21C, y)?
            .y;
        let out = s.line(rank_font, &decimal(rank, Pad::None), 0xC8, y)?;
        y = out.y;
        let sy = y.wrapping_add(sprite_dy);
        if keeps_y {
            s.keep(Work::RowY, sy as u32);
        }
        s.sprite(flag_glyph(row.nation), 0xD2, sy)?;
        s.sprite(car_glyph(row.car), 0x1CC, sy)?;
        y = y.wrapping_add(0x19);
    }
    Ok(())
}

/// A times list (`draw_time_rows`): as `scores`, the figure a time (`show_hours`) drawn in
/// fixed columns ending at x 0x276 (the rank then at the name's y). True when all ten rows
/// were drawn and the list does not end right after them.
pub fn time_rows<S, T>(
    s: &mut S,
    rows: &Rows<T>,
    keep: Keep,
    show_hours: bool,
) -> Result<bool, S::Error>
where
    S: Screen<Font = Font, Text = T, Work = Work>,
    T: Copy,
{
    let mut y = 0xF0;
    for rank in 1..=10u32 {
        let n = rank - 1;
        if n > 0 && (rows.ends_at)(n) {
            return Ok(false);
        }
        s.keep(Work::Rank, rank);
        let row = (rows.row)(n);
        let (name_font, time_font, rank_font) = fonts(row.marked);
        y = s.text(name_font, row.name, 0xF0, y)?.y;
        s.time_line(time_font, &time(row.figure, show_hours), 0x276, y)?;
        let out = s.line(rank_font, &decimal(rank, Pad::None), 0xC8, y)?;
        y = out.y;
        let sy = if keep == Keep::Below4 {
            y.wrapping_add(4)
        } else {
            y
        };
        if keep != Keep::Nothing {
            s.keep(Work::RowY, sy as u32);
        }
        s.sprite(flag_glyph(row.nation), 0xD2, sy)?;
        let glyph = car_glyph(row.car);
        s.sprite(glyph, 0x1CC, sy)?;
        if keep == Keep::Below4 {
            s.keep(Work::CarGlyph, glyph);
        }
        y = y.wrapping_add(0x19);
    }
    // The original looks at the entry after the tenth before it counts the ten: a list
    // ending there counts as ended early.
    Ok(!(rows.ends_at)(10))
}

/// A page's heading: string `id` at (0x2EB, 0x77), then a stage line (no NUL) at
/// (0x2EB, 0x91) when there is one; `line` gives it once the string is drawn (its errors
/// are the caller's: the original builds it there).
pub fn heading<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    id: u32,
    line: Option<&dyn Fn() -> Result<Vec<u8>, S::Error>>,
) -> Result<(), S::Error> {
    s.string(Font::Heading, id, 0x2EB, 0x77)?;
    if let Some(line) = line {
        s.line(Font::Heading, &line()?, 0x2EB, 0x91)?;
    }
    Ok(())
}

/// The heading string of the event and overall best scores: in event mode 0x13 the
/// section's (`section_string`), else 0x35E + the group (kept) for the event's, 0x362 for
/// the overall.
pub fn scores_heading<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    section_string: Option<u32>,
    event_group: Option<u32>,
) -> u32 {
    match (section_string, event_group) {
        (Some(id), _) => id,
        (None, Some(group)) => {
            let id = 0x35E + group;
            s.keep(Work::Heading, id);
            id
        }
        (None, None) => 0x362,
    }
}

/// The record's fastest line: string 0x1A2 at (0x142, 0x1FB), the flag's sprite after it
/// (x 0x14F), the name (x 0x172), the car's sprite (x 0x225, after the name), the speed
/// (no NUL) at x 0x262.
pub fn fastest<S, T>(
    s: &mut S,
    nation: u32,
    name: T,
    car: u32,
    speed: &[u8],
) -> Result<(), S::Error>
where
    S: Screen<Font = Font, Text = T, Work = Work>,
    T: Copy,
{
    s.string(Font::Fastest, 0x1A2, 0x142, 0x1FB)?;
    s.sprite(flag_glyph(nation), 0x14F, 0x1FB)?;
    s.text(Font::Name, name, 0x172, 0x1FB)?;
    s.sprite(car_glyph(car), 0x225, 0x1FB)?;
    s.line(Font::Name, speed, 0x262, 0x1FB)?;
    Ok(())
}

/// The replays' range caption: n-m, n the page x 20, m n + 19.
pub fn range_caption(page: u32) -> Vec<u8> {
    let n = page.wrapping_mul(0x14);
    let mut text = decimal(n, Pad::None);
    text.push(b'-');
    text.extend(decimal(n.wrapping_add(0x13), Pad::None));
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Invalid;
    use crate::screens::screen::{Buffer, Drawn};

    #[derive(Default)]
    struct Record(Vec<String>, String);

    impl Screen for Record {
        type Font = Font;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, f: Font, t: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("text {f:?} {t} {x} {y}"));
            Ok(Drawn {
                y: y + 1,
                too_wide: false,
            })
        }
        fn store(&mut self, _: Buffer, b: &[u8]) -> Result<(), Invalid> {
            self.1 = String::from_utf8_lossy(&b[..b.len() - 1]).into_owned();
            Ok(())
        }
        fn draw(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("line {f:?} {} {x} {y}", self.1));
            Ok(Drawn { y, too_wide: false })
        }
        fn draw_clock(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("clock {f:?} {x:#x} {y:#x}"));
            Ok(())
        }
        fn draw_time(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("time {f:?} {} {x} {y}", self.1));
            Ok(())
        }
        fn string(&mut self, f: Font, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("string {f:?} {id:#x} {x} {y}"));
            Ok(Drawn { y, too_wide: false })
        }
        fn string_bytes(&mut self, id: u32) -> Vec<u8> {
            format!("s{id:x}").into_bytes()
        }
        fn strings(&mut self, first: u32, _: u32) -> Result<Vec<u8>, Invalid> {
            Ok(format!("t{first:x}\0").into_bytes())
        }
        fn sprite(&mut self, g: u32, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!("sprite {g:#x} {x} {y}"));
            Ok(())
        }
        fn bar(&mut self, b: crate::screens::screen::Bar) -> Result<(), Invalid> {
            self.0.push(format!("bar {b:?}"));
            Ok(())
        }
        fn fatal(&mut self, _: u32, message: &'static str) -> Invalid {
            Invalid(message)
        }
        fn keep(&mut self, w: Work, v: u32) {
            self.0.push(format!("keep {w:?} {v}"));
        }
    }

    #[test]
    fn lists() {
        let row = |n: u32| Row {
            name: n,
            figure: 100 + n,
            nation: 2,
            car: 0x18,
            marked: n == 1,
        };
        let mut s = Record::default();
        scores(
            &mut s,
            &Rows {
                row: &row,
                ends_at: &|n| n == 2,
            },
            true,
            0,
        )
        .unwrap();
        assert_eq!(s.0[1], "text Name 0 240 240");
        assert_eq!(s.0[2], "line Name 100 540 241");
        assert_eq!(s.0[5], "sprite 0x1e 210 241");
        assert_eq!(s.0[6], "sprite 0x5d 460 241");
        assert_eq!(s.0[8], "text MarkedName 1 240 266");
        assert_eq!(s.0.len(), 14);
        let mut s = Record::default();
        assert!(
            time_rows(
                &mut s,
                &Rows {
                    row: &row,
                    ends_at: &|_| false
                },
                Keep::Below4,
                false
            )
            .unwrap()
        );
        assert_eq!(
            s.0[2],
            "time Time 0\"01.00 630 241"
                .replace("0\"01.00", &String::from_utf8_lossy(&time(100, false)))
        );
        assert_eq!(range_caption(2), b"40-59");
    }
}
