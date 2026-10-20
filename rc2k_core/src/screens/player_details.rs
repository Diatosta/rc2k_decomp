//! The players' details pages (converted 2026-10-20 from `screens::draw_player_details`
//! 0x46F894, the same code as `draw_player2_details` 0x4677B5, and `draw_player34_details`
//! 0x467975 without the second view): the name (and the typing cursor), the nationality
//! and its flag, the views, the race number, the co-driver, the sensitivity's bar.

use crate::screens::screen::Screen;
use crate::strings::format::{Pad, decimal};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E8C4`.
    Value,
    /// `stru_51E824`: the nationality measured for its flag's place.
    Measure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `seconds` (0x7559AC): the string drawn.
    Seconds,
    /// `dword_7559EC`.
    Kept,
}

/// What the page shows, its strings looked up.
#[derive(Clone, Copy, Debug)]
pub struct Details<T> {
    pub name: T,
    /// A name is being typed (`dword_7556A8` not 0): the cursor after it.
    pub typing: bool,
    pub nationality: u32,
    /// `dword_758816[nationality]`.
    pub nationality_id: u32,
    /// `dword_7589BA[view]`.
    pub view_id: u32,
    /// `dword_7589DA[second view]` on the pages that have it.
    pub second_view_id: Option<u32>,
    pub race_number: u32,
    pub codriver_sex: u32,
    pub sensitivity: u32,
}

/// The typing cursor, drawn after the name from the x the name was drawn from.
pub type Cursor<'a, S> = &'a mut dyn FnMut(&mut S, i32) -> Result<(), <S as Screen>::Error>;

/// The page; `cursor` draws the typing cursor after the name (`screens::draw_name_cursor`,
/// with the x the name was drawn from). The lines below the views are 0x1B lower on a page
/// with the second view. The flag's palette is the measuring's line count, the bar's what
/// the last text left.
pub fn player_details<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    d: &Details<S::Text>,
    cursor: Cursor<S>,
) -> Result<(), S::Error> {
    let x = 0x1BB;
    s.text(Font::Value, d.name, x, 0xD4)?;
    if d.typing {
        cursor(s, x)?;
    }
    s.keep(Work::Seconds, d.nationality_id);
    s.string(Font::Value, d.nationality_id, x, 0xEF)?;
    let (width, lines) = s.measure_string(Font::Measure, d.nationality_id, x)?;
    s.sprite(
        0x1C + d.nationality,
        width.wrapping_add(0x1CA) as i32,
        0xF3,
        lines,
    )?;
    s.keep(Work::Seconds, d.view_id);
    s.string(Font::Value, d.view_id, x, 0x10A)?;
    let mut y = 0x125;
    if let Some(id) = d.second_view_id {
        s.keep(Work::Seconds, id);
        s.string(Font::Value, id, x, y)?;
        y += 0x1B;
    }
    s.line(Font::Value, &decimal(d.race_number, Pad::None), x, y)?;
    let codriver = d.codriver_sex.wrapping_add(0x18A);
    s.keep(Work::Kept, codriver);
    let palette = s.string(Font::Value, codriver, x, y + 0x1B)?.after;
    let bar = y + 0x36;
    s.sprite(4, 0x1BA, bar, palette)?;
    let marker = d.sensitivity.wrapping_mul(2).wrapping_add(0x1BA);
    s.keep(Work::Kept, marker);
    s.sprite(6, marker as i32, bar, palette)?;
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
        fn draw_time(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn draw_clock(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn string(&mut self, f: Font, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} s{id:x} {x:#x} {y:#x}"));
            Ok(Drawn {
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
            self.0.push(format!("keep {w:?} {v:#x}"));
        }
        fn bar(&mut self, _: Bar) -> Result<(), Invalid> {
            unreachable!()
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
        fn measure_string(&mut self, _: Font, _: u32, _: i32) -> Result<(u32, u32), Invalid> {
            Ok((30, 1))
        }
    }

    #[test]
    fn pages() {
        let d = Details {
            name: 1,
            typing: true,
            nationality: 3,
            nationality_id: 0x700,
            view_id: 0x710,
            second_view_id: Some(0x720),
            race_number: 12,
            codriver_sex: 1,
            sensitivity: 5,
        };
        let mut s = Record::default();
        player_details(&mut s, &d, &mut |s: &mut Record, x| {
            s.0.push(format!("cursor {x:#x}"));
            Ok(())
        })
        .unwrap();
        assert_eq!(s.0[1], "cursor 0x1bb");
        assert!(s.0.contains(&"sprite 0x1f 0x1e8 0xf3 0x1".to_string()));
        assert!(s.0.contains(&"Value \"12\\0\" 0x1bb 0x140".to_string()));
        assert_eq!(s.0.last().unwrap(), "sprite 0x6 0x1c4 0x176 0xa8b");
        let mut s = Record::default();
        let short = Details {
            typing: false,
            second_view_id: None,
            ..d
        };
        player_details(&mut s, &short, &mut |_: &mut Record, _| unreachable!()).unwrap();
        assert_eq!(s.0.last().unwrap(), "sprite 0x6 0x1c4 0x15b 0xa8b");
    }
}
