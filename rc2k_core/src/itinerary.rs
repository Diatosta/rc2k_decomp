//! The itinerary page (converted 2026-10-20 from `screens::draw_itinerary` 0x472C8C): a
//! section's records row by row, a service with its number and time, a stage with its name,
//! length, number and time, the chosen stage's row in the highlight fonts.

use crate::Invalid;
use crate::format::{Pad, clock_padded, decimal};
use crate::screen::{Buffer, Screen};
use crate::units::{Units, distance_2dp, hhmm_to_minutes};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Heading,
    Plain,
    Marked,
    Clock,
    MarkedClock,
}

/// The work variables (the original keeps its counters in them and reads them back).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// The row's y.
    Y,
    /// The row's number (from 1).
    Row,
    /// The next stage's number (from 1).
    Stage,
    /// The services so far.
    Service,
    /// The last service's string.
    ServiceString,
}

/// A record of the section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// A service, at a time of day (hours * 100 + minutes).
    Service { hhmm: u32 },
    /// A stage: its name's string, its length (hundredths of a mile, 0 none), its time.
    Stage { name: u32, length: u32, hhmm: u32 },
    /// Tags 3 and 4: the list ends.
    End,
}

/// The page: the heading string at (0x2EB, 0x77), the column heads (strings 0x482, 0x47E,
/// 0x480, 0x47F at y 0xD1), then the records from 1 (`row` reads record n where the
/// original does, its errors the caller's: another tag is its fatal error), from y 0xE8,
/// 0x19 apart, until `ends_at(n)` (the next record's first dword -1). `marked` is the
/// chosen row; `units` the game's setting (0 imperial).
pub fn itinerary<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    heading: u32,
    row: &dyn Fn(u32) -> Result<Row, S::Error>,
    ends_at: &dyn Fn(u32) -> bool,
    marked: u32,
    units: u32,
) -> Result<(), S::Error> {
    s.string(Font::Heading, heading, 0x2EB, 0x77)?;
    for (id, x) in [
        (0x482, 0xBA),
        (0x47E, 0x10A),
        (0x480, 0x1F1),
        (0x47F, 0x257),
    ] {
        s.string(Font::Plain, id, x, 0xD1)?;
    }
    let (mut y, mut number, mut stage, mut service) = (0xE8u32, 1u32, 1u32, 0u32);
    s.keep(Work::Y, y);
    s.keep(Work::Row, number);
    s.keep(Work::Stage, stage);
    s.keep(Work::Service, service);
    let mut n = 1;
    loop {
        let at = y as i32;
        match row(n)? {
            Row::End => return Ok(()),
            Row::Service { hhmm } => {
                let id = 0x478 + service;
                s.keep(Work::ServiceString, id);
                s.string(Font::Plain, id, 0x10A, at)?;
                service = service.wrapping_add(1);
                s.keep(Work::Service, service);
                clock_at(s, hhmm, Font::Clock, at)?;
            }
            Row::Stage { name, length, hhmm } => {
                let mark = marked == number;
                let font = if mark { Font::Marked } else { Font::Plain };
                s.string(font, name, 0x10A, at)?;
                if length == 0 {
                    // The original's `append_nul` on both ways: two NULs.
                    s.store(Buffer::Line, &[0, 0])?;
                    s.draw(font, Buffer::Line, 0x1F1, at)?;
                } else {
                    let metric = units != 0;
                    let u = if metric {
                        Units::Metric
                    } else {
                        Units::Imperial
                    };
                    let mut line = distance_2dp(length, u).ok_or(Invalid(
                        "result does not fit 32 bits: the original's div traps",
                    ))?;
                    line.extend(s.string_bytes(if metric { 0x570 } else { 0x56F }));
                    s.line(font, &line, 0x1F1, at)?;
                }
                s.line(font, &decimal(stage, Pad::None), 0xBA, at)?;
                stage = stage.wrapping_add(1);
                s.keep(Work::Stage, stage);
                let clock_font = if mark { Font::MarkedClock } else { Font::Clock };
                clock_at(s, hhmm, clock_font, at)?;
            }
        }
        y = y.wrapping_add(0x19);
        s.keep(Work::Y, y);
        number = number.wrapping_add(1);
        s.keep(Work::Row, number);
        n += 1;
        if ends_at(n) {
            return Ok(());
        }
    }
}

/// A time of day as "09:05" in fixed columns ending at x 0x2C2.
fn clock_at<S: Screen<Font = Font>>(
    s: &mut S,
    hhmm: u32,
    font: Font,
    y: i32,
) -> Result<(), S::Error> {
    let mut text = clock_padded(hhmm_to_minutes(hhmm));
    text.push(0);
    s.store(Buffer::Line, &text)?;
    s.draw_clock(font, Buffer::Line, 0x2C2, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::Drawn;

    #[derive(Default)]
    struct Record(Vec<String>, Vec<u8>);

    impl Screen for Record {
        type Font = Font;
        type Text = u32;
        type Work = Work;
        type Error = Invalid;
        fn text(&mut self, _: Font, _: u32, _: i32, _: i32) -> Result<Drawn, Invalid> {
            unreachable!()
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
        fn draw_time(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn draw_clock(&mut self, f: Font, _: Buffer, x: i32, y: i32) -> Result<(), Invalid> {
            self.0.push(format!(
                "clock {f:?} {:?} {x:#x} {y:#x}",
                String::from_utf8_lossy(&self.1)
            ));
            Ok(())
        }
        fn string(&mut self, f: Font, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} s{id:x} {x:#x} {y:#x}"));
            Ok(Drawn::default())
        }
        fn string_bytes(&mut self, id: u32) -> Vec<u8> {
            format!("<{id:x}>").into_bytes()
        }
        fn strings(&mut self, _: u32, _: u32) -> Result<Vec<u8>, Invalid> {
            unreachable!()
        }
        fn sprite(&mut self, _: u32, _: i32, _: i32, _: u32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn bar(&mut self, b: crate::screen::Bar) -> Result<(), Invalid> {
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
    fn rows() {
        let rows = [
            Row::Stage {
                name: 0x99,
                length: 1234,
                hhmm: 905,
            },
            Row::Service { hhmm: 1030 },
            Row::Stage {
                name: 0x98,
                length: 0,
                hhmm: 1100,
            },
            Row::End,
        ];
        let mut s = Record::default();
        itinerary(
            &mut s,
            0x77,
            &|n| Ok(rows[n as usize - 1]),
            &|_| false,
            3,
            0,
        )
        .unwrap();
        let has = |l: &str| s.0.iter().any(|x| x == l);
        assert!(has("Plain s99 0x10a 0xe8"));
        assert!(has("Plain \"12.34<56f>\\0\" 0x1f1 0xe8"));
        assert!(has("Plain \"1\\0\" 0xba 0xe8"));
        assert!(has("clock Clock \"09:05\\0\" 0x2c2 0xe8"));
        assert!(has("keep ServiceString 1144"));
        assert!(has("Marked s98 0x10a 0x11a"));
        assert!(has("Marked \"\\0\\0\" 0x1f1 0x11a"));
        assert!(has("Marked \"2\\0\" 0xba 0x11a"));
        assert!(has("clock MarkedClock \"11:00\\0\" 0x2c2 0x11a"));
        assert_eq!(s.0.last().unwrap(), "keep Row 4");
    }
}
