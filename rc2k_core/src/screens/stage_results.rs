//! The time trial stage results page (converted 2026-10-20 from `screens::draw_stage_results`
//! 0x48E740): ten rows of name, result and rank with the car's and flag's sprites, then the
//! fastest line.

use crate::Invalid;
use crate::screens::best_scores::{car_glyph, flag_glyph};
use crate::screens::screen::{Buffer, Screen};
use crate::strings::format::{Pad, decimal, time};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E584`: names, the gap's string, the speed.
    Rows,
    /// `stru_51E564`: times and the seal codes.
    Small,
    /// `stru_51E5A4`: the ranks.
    Rank,
    /// `stru_51E6E4`: the fastest line's string.
    Fastest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `seconds` (0x7559AC): the rank.
    Rank,
    /// `dword_7559EC`: the seal code's y.
    Scratch,
    /// `dword_75AB24`: the first row's time.
    FirstTime,
    /// `dword_7559F0`: a row's gap.
    Gap,
    /// `dword_7559B0`: the row's y after its rank.
    RowY,
}

/// A row: an entry of the times table (0x9C bytes).
#[derive(Clone, Copy, Debug)]
pub struct Entry<T> {
    /// +0x20.
    pub name: T,
    /// +8.
    pub nation: u32,
    /// +0x10.
    pub car: u32,
    /// +0x14, hundredths.
    pub time: u32,
    /// +0x54, +0x58.
    pub seal: (u32, u32),
}

/// The fastest line: the stage record's holder (its first or second).
#[derive(Clone, Debug)]
pub struct Fastest<T> {
    pub nation: u32,
    pub name: T,
    pub car: u32,
    /// The speed as formatted (no NUL).
    pub speed: Vec<u8>,
}

/// The page's drawing: in seal mode each row's seal code (`seal_code`, which the game keeps
/// at 0x4E0544) one character at a time in `byte_A8BC54` from x 0x20A every 0xE; else its
/// time at 0x276, and for the rows after the first the gap (string 0x579, the time at
/// 0x2EE). The sprites' palette is where the rank's text ended.
pub fn stage_results<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    seal_mode: bool,
    show_hours: bool,
    rows: &[Entry<S::Text>; 10],
    seal_code: &mut dyn FnMut(u32, u32) -> Vec<u8>,
    fastest: &Fastest<S::Text>,
) -> Result<(), S::Error> {
    let mut y = 0xF0i32;
    let mut first = 0;
    for (n, e) in rows.iter().enumerate() {
        let rank = n as u32 + 1;
        s.keep(Work::Rank, rank);
        y = s.text(Font::Rows, e.name, 0xF0, y)?.y;
        if seal_mode {
            let code = seal_code(e.seal.0, e.seal.1);
            if code.is_empty() {
                return Err(Invalid("an empty seal code").into());
            }
            s.keep(Work::Scratch, y as u32);
            for (n, &c) in code.iter().enumerate() {
                s.store(Buffer::Cells, &[c, 0, 0, 0])?;
                s.draw(Font::Small, Buffer::Cells, 0x20A + 0xE * n as i32, y)?;
            }
        } else {
            s.time_line(Font::Small, &time(e.time, show_hours), 0x276, y)?;
            if rank == 1 {
                first = e.time;
                s.keep(Work::FirstTime, first);
            } else {
                let gap = e.time.wrapping_sub(first);
                s.keep(Work::Gap, gap);
                y = s.string(Font::Rows, 0x579, 0x276, y)?.y;
                s.time_line(Font::Small, &time(gap, show_hours), 0x2EE, y)?;
            }
        }
        let out = s.line(Font::Rank, &decimal(rank, Pad::None), 0xC8, y)?;
        y = out.y;
        s.keep(Work::RowY, y as u32);
        s.sprite(flag_glyph(e.nation), 0xD2, y)?;
        s.sprite(car_glyph(e.car), 0x1CC, y)?;
        y = y.wrapping_add(0x19);
    }
    s.string(Font::Fastest, 0x1A2, 0x142, 0x1F1)?;
    s.sprite(flag_glyph(fastest.nation), 0x14F, 0x1F1)?;
    s.text(Font::Rows, fastest.name, 0x172, 0x1F1)?;
    s.sprite(car_glyph(fastest.car), 0x225, 0x1F1)?;
    s.line(Font::Rows, &fastest.speed, 0x262, 0x1F1)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
                y: y + 1,
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
        let rows = std::array::from_fn(|n| Entry {
            name: n as u32,
            nation: n as u32,
            car: 0x19,
            time: 1000 + 100 * n as u32,
            seal: (n as u32, 0),
        });
        let fastest = Fastest {
            nation: 3,
            name: 9,
            car: 2,
            speed: b"99".to_vec(),
        };
        let mut s = Record::default();
        stage_results(
            &mut s,
            false,
            false,
            &rows,
            &mut |_, _| unreachable!(),
            &fastest,
        )
        .unwrap();
        assert_eq!(s.0[1], "Rows n0 0xf0 0xf0");
        assert!(s.0.contains(&"keep Gap 100".to_string()));
        assert!(s.0.contains(&"sprite 0x5c 0x1cc 0xf1".to_string()));
        assert_eq!(s.0.last().unwrap(), "Rows \"99\\0\" 0x262 0x1f1");
        let mut s = Record::default();
        let mut code = |a: u32, _| format!("C{a}").into_bytes();
        stage_results(&mut s, true, false, &rows, &mut code, &fastest).unwrap();
        assert_eq!(s.0[3], "Small \"C\\0\\0\\0\" 0x20a 0xf1");
        assert_eq!(s.0[4], "Small \"0\\0\\0\\0\" 0x218 0xf1");
    }
}
