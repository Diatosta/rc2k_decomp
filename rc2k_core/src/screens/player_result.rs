//! The current player's result pages (a championship stage and the rally's). Each: labels,
//! the times heading of another family (drawn by the caller's closure, in its place), the
//! player's values, the nationality's and the car's sprites.

use crate::screens::best_scores::{car_glyph, flag_glyph};
use crate::screens::screen::Screen;
use crate::strings::format::{Pad, decimal, time};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    Labels,
    Values,
    /// The stage page measures the name with it.
    Measure,
    /// The rally page's name.
    Name,
}

/// The pages keep nothing in work variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// The other family's heading, drawn in its place.
pub type Heading<'a, S> = &'a mut dyn FnMut(&mut S) -> Result<(), <S as Screen>::Error>;

#[derive(Clone, Debug)]
pub struct Player<T> {
    pub name: T,
    pub nation: u32,
    pub car: u32,
    /// Hundredths.
    pub time: u32,
    /// The top speed as formatted (no NUL).
    pub speed: Vec<u8>,
}

/// The stage page: labels 0x206 .. 0x208, the heading, the time and the speed at x 0x1FE, the
/// nationality's sprite, the name at 0x221 and the car's sprite after it (the name's width
/// measured in `Measure`).
pub fn stage_result<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    heading: Heading<S>,
    p: &Player<S::Text>,
    show_hours: bool,
) -> Result<(), S::Error> {
    for (n, id) in [0x206u32, 0x207, 0x208].into_iter().enumerate() {
        s.string(Font::Labels, id, 0x1EC, 0xCF + 0x2D * n as i32)?;
    }
    heading(s)?;
    s.line(Font::Values, &time(p.time, show_hours), 0x1FE, 0xFC)?;
    s.line(Font::Values, &p.speed, 0x1FE, 0x12A)?;
    s.sprite(flag_glyph(p.nation), 0x1FE, 0xCF)?;
    s.text(Font::Values, p.name, 0x221, 0xCF)?;
    let width = s.measure(Font::Measure, p.name, 0x221)?;
    s.sprite(car_glyph(p.car), (width as i32).wrapping_add(0x221), 0xCF)?;
    Ok(())
}

/// The rally page's labels: string, x, y, font.
const RALLY_LABELS: [(u32, i32, i32, Font); 7] = [
    (0x213, 0xED, 0xE7, Font::Values),
    (0x20E, 0x155, 0x12B, Font::Labels),
    (0x20F, 0x155, 0x142, Font::Labels),
    (0x208, 0x155, 0x159, Font::Labels),
    (0x210, 0x155, 0x187, Font::Labels),
    (0x211, 0x155, 0x19F, Font::Labels),
    (0x212, 0x155, 0x1CD, Font::Labels),
];

/// A player time of 0x980000: none (string 0x513).
pub const NO_TIME: u32 = 0x98_0000;

/// The rally page's figures (`Results.positions[0]`, `Results.time_bonus[0]`, and the six at x
/// 0x21C).
#[derive(Clone, Copy, Debug)]
pub struct RallyFigures {
    pub position: u32,
    pub best: u32,
    /// `Results.points[0]`, `Results.time_bonus[0]`, `Results.extra[0]`, `Results.sums[0]`,
    /// `Results.before[0]`, `Results.totals[0]`, as numbers.
    pub column: [u32; 6],
}

/// The rally page: labels, heading, the position, the time (or string 0x513), the best time
/// and the speed, the six numbers at x 0x21C, the nationality's sprite, the name in `Name` at
/// 0x168 and the car's sprite at 0x1FE.
pub fn rally_result<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    heading: Heading<S>,
    p: &Player<S::Text>,
    f: &RallyFigures,
    show_hours: bool,
) -> Result<(), S::Error> {
    for (id, x, y, font) in RALLY_LABELS {
        s.string(font, id, x, y)?;
    }
    heading(s)?;
    s.line(Font::Values, &decimal(f.position, Pad::None), 0x18C, 0x12B)?;
    if p.time == NO_TIME {
        s.string(Font::Values, 0x513, 0x1CF, 0xE7)?;
    } else {
        s.line(Font::Values, &time(p.time, show_hours), 0x23A, 0xE7)?;
    }
    s.line(Font::Values, &time(f.best, show_hours), 0x18C, 0x142)?;
    s.line(Font::Values, &p.speed, 0x18C, 0x159)?;
    for (v, y) in f
        .column
        .into_iter()
        .zip([0x12B, 0x142, 0x159, 0x187, 0x19F, 0x1CD])
    {
        s.line(Font::Values, &decimal(v, Pad::None), 0x21C, y)?;
    }
    s.sprite(flag_glyph(p.nation), 0x146, 0xE7)?;
    s.text(Font::Name, p.name, 0x168, 0xE7)?;
    s.sprite(car_glyph(p.car), 0x1FE, 0xE7)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Invalid;
    use crate::screens::screen::{Bar, Buffer, Drawn};

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
            Ok(Drawn::default())
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
        fn keep(&mut self, _: Work, _: u32) {
            unreachable!()
        }
        fn bar(&mut self, _: Bar) -> Result<(), Invalid> {
            unreachable!()
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
        fn measure(&mut self, f: Font, t: u32, x: i32) -> Result<u32, Invalid> {
            self.0.push(format!("measure {f:?} n{t} {x:#x}"));
            Ok(40)
        }
    }

    #[test]
    fn pages() {
        let p = Player {
            name: 7,
            nation: 2,
            car: 0x19,
            time: 1234,
            speed: b"150".to_vec(),
        };
        let mut s = Record::default();
        stage_result(
            &mut s,
            &mut |s: &mut Record| {
                s.0.push("heading".into());
                Ok(())
            },
            &p,
            false,
        )
        .unwrap();
        assert_eq!(s.0[3], "heading");
        assert!(s.0.contains(&"sprite 0x1e 0x1fe 0xcf".to_string()));
        assert_eq!(s.0.last().unwrap(), "sprite 0x5c 0x249 0xcf");
        let f = RallyFigures {
            position: 3,
            best: 1000,
            column: [1, 2, 3, 4, 5, 6],
        };
        let mut s = Record::default();
        let none = Player { time: NO_TIME, ..p };
        rally_result(&mut s, &mut |_: &mut Record| Ok(()), &none, &f, false).unwrap();
        assert!(s.0.contains(&"Values s513 0x1cf 0xe7".to_string()));
        assert!(s.0.contains(&"sprite 0x1e 0x146 0xe7".to_string()));
        assert_eq!(s.0.last().unwrap(), "sprite 0x5c 0x1fe 0xe7");
    }
}
