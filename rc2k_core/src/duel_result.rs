//! The two-player stage result (converted 2026-10-20 from `screens::draw_duel_result`
//! 0x48647A): the heads, the faster of players 1 and 2 first, the stage, then per row the
//! time and sprites and name, and by player 1 and 2 (not by order) the figures.

use crate::best_scores::{car_glyph, flag_glyph};
use crate::format::{Pad, decimal, time};
use crate::player_result::Player;
use crate::screen::{Buffer, Screen};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E584`.
    Big,
    /// `stru_51E5A4`.
    Small,
    /// `stru_51E9A4`.
    Stage,
    /// `stru_51E564`.
    Time,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `dword_758720`, `dword_758724`: the rows' byte offsets in `g_players`.
    First,
    Second,
}

/// A time of 0x36EE80: none (string 0x4FA).
pub const NO_TIME: u32 = 0x36EE80;

/// What the page shows by player 1 and 2.
#[derive(Clone, Copy, Debug)]
pub struct Figures {
    /// The strings `dword_7588D2[dword_758768 - 1]`, `[dword_75876C - 1]`.
    pub labels: [u32; 2],
    /// `dword_758758`, `dword_75875C`.
    pub best: [u32; 2],
    /// `dword_758778` .. (x 0x1C4) and `dword_75877C` .. (x 0x2C8), as numbers.
    pub columns: [[u32; 6]; 2],
}

/// The page: `section` the stage section's name string, `stage` the stage line with its
/// NUL. The nationalities' sprites take the palette where the second speed ended; the cars'
/// where the second name ended.
pub fn duel_result<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    players: &[Player<S::Text>; 2],
    section: u32,
    stage: &[u8],
    f: &Figures,
    show_hours: bool,
) -> Result<(), S::Error> {
    s.string(Font::Big, 0x213, 0x135, 0x114)?;
    s.string(Font::Big, 0x214, 0x236, 0x114)?;
    for (id, y) in [
        (0x20E, 0x12B),
        (0x217, 0x142),
        (0x208, 0x159),
        (0x210, 0x187),
        (0x211, 0x19E),
        (0x212, 0x1CD),
    ] {
        s.string(Font::Small, id, 0xFD, y)?;
    }
    let swapped = players[1].time < players[0].time;
    let (top, bottom) = if swapped {
        (0x216, 0x215)
    } else {
        (0x215, 0x216)
    };
    s.string(Font::Small, top, 0xFD, 0xD0)?;
    s.string(Font::Small, bottom, 0xFD, 0xE7)?;
    s.keep(Work::First, if swapped { 4 } else { 0 });
    s.keep(Work::Second, if swapped { 0 } else { 4 });
    let order = if swapped {
        [&players[1], &players[0]]
    } else {
        [&players[0], &players[1]]
    };
    s.string(Font::Stage, section, 0x2EB, 0x77)?;
    s.store(Buffer::Line, stage)?;
    s.draw(Font::Stage, Buffer::Line, 0x2EB, 0x91)?;
    for (p, y) in order.iter().zip([0xD0, 0xE7]) {
        if p.time == NO_TIME {
            s.string(Font::Time, 0x4FA, 0x281, y)?;
        } else {
            s.time_line(Font::Time, &time(p.time, show_hours), 0x2AD, y)?;
        }
    }
    s.line(Font::Big, &players[0].speed, 0x135, 0x159)?;
    let palette = s.line(Font::Big, &players[1].speed, 0x236, 0x159)?.after;
    for (p, y) in order.iter().zip([0xD0, 0xE7]) {
        s.sprite(flag_glyph(p.nation), 0x135, y, palette)?;
    }
    let mut palette = 0;
    for (p, y) in order.iter().zip([0xD0, 0xE7]) {
        palette = s.text(Font::Big, p.name, 0x152, y)?.after;
    }
    for (p, y) in order.iter().zip([0xCF, 0xE7]) {
        s.sprite(car_glyph(p.car), 0x1FC, y, palette)?;
    }
    s.line(Font::Big, &time(f.best[0], show_hours), 0x135, 0x142)?;
    s.line(Font::Big, &time(f.best[1], show_hours), 0x236, 0x142)?;
    s.string(Font::Big, f.labels[0], 0x135, 0x12B)?;
    s.string(Font::Big, f.labels[1], 0x236, 0x12B)?;
    let rows = [0x12B, 0x142, 0x159, 0x187, 0x19F, 0x1CD];
    for (x, column) in [0x1C4, 0x2C8].into_iter().zip(f.columns) {
        for (v, y) in column.into_iter().zip(rows) {
            s.line(Font::Big, &decimal(v, Pad::None), x, y)?;
        }
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
                after: 0x500 + t,
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
                after: 0x700 + x as u32,
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
            Ok(Drawn::default())
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
    }

    #[test]
    fn page() {
        let p = |n: u32, time: u32| Player {
            name: n,
            nation: n,
            car: 0x18,
            time,
            speed: b"99".to_vec(),
        };
        let players = [p(1, 2000), p(2, 1000)];
        let f = Figures {
            labels: [0x600, 0x601],
            best: [10, 20],
            columns: [[1; 6], [2; 6]],
        };
        let mut s = Record::default();
        duel_result(&mut s, &players, 0x300, b"stage\0", &f, false).unwrap();
        assert!(s.0.contains(&"Small s216 0xfd 0xd0".to_string()));
        assert!(s.0.contains(&"keep First 4".to_string()));
        assert!(s.0.contains(&"sprite 0x1e 0x135 0xd0 0x936".to_string()));
        assert!(s.0.contains(&"sprite 0x5d 0x1fc 0xcf 0x501".to_string()));
        assert!(s.0.contains(&"Big n2 0x152 0xd0".to_string()));
        assert_eq!(s.0.last().unwrap(), "Big \"2\\0\" 0x2c8 0x1cd");
    }
}
