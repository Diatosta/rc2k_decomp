//! The players' details pages (from `screens::draw_player_details`
//! 0x46F894, the same code as `draw_player2_details` 0x4677B5, and `draw_player34_details`
//! 0x467975 without the second view): the name (and the typing cursor), the nationality
//! and its flag, the views, the race number, the co-driver, the sensitivity's bar; and the
//! typing cursor (`screens::draw_name_cursor` 0x4677EF).

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
    /// The controller's item of the menu, greyed out without a controller
    /// (`g_controllerCount` 0).
    pub controller_item: u32,
    pub controllers: bool,
    pub name: T,
    /// A name is being typed (`dword_7556A8` not 0): the cursor after it.
    pub typing: bool,
    /// The cursor's blink (`dword_7559A8`): drawn while 1.
    pub blink: bool,
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

/// The typing cursor: the name measured from x, string 4 after it while the blink is 1.
/// The carry: the cursor's text's, or (not drawn) its `cmp blink, 1`.
pub fn name_cursor<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    name: S::Text,
    x: i32,
    blink: bool,
) -> Result<bool, S::Error> {
    let width = s.measure(Font::Measure, name, x)?;
    if blink {
        Ok(
            s.string(Font::Value, 4, width.wrapping_add(0x1BB) as i32, 0xD4)?
                .too_wide,
        )
    } else {
        Ok(true)
    }
}

/// The page; the typing cursor after the name measured from the x it was drawn from. The lines below the views are 0x1B lower on a page
/// with the second view.
pub fn player_details<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    d: &Details<S::Text>,
) -> Result<(), S::Error> {
    s.item(d.controller_item, d.controllers)?;
    let x = 0x1BB;
    s.text(Font::Value, d.name, x, 0xD4)?;
    if d.typing {
        name_cursor(s, d.name, x, d.blink)?;
    }
    s.keep(Work::Seconds, d.nationality_id);
    s.string(Font::Value, d.nationality_id, x, 0xEF)?;
    let width = s.measure_string(Font::Measure, d.nationality_id, x)?;
    s.sprite(0x1C + d.nationality, width.wrapping_add(0x1CA) as i32, 0xF3)?;
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
    s.string(Font::Value, codriver, x, y + 0x1B)?;
    let bar = y + 0x36;
    s.sprite(4, 0x1BA, bar)?;
    let marker = d.sensitivity.wrapping_mul(2).wrapping_add(0x1BA);
    s.keep(Work::Kept, marker);
    s.sprite(6, marker as i32, bar)?;
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
            self.0.push(format!("keep {w:?} {v:#x}"));
        }
        fn bar(&mut self, _: Bar) -> Result<(), Invalid> {
            unreachable!()
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
        fn item(&mut self, n: u32, on: bool) -> Result<(), Invalid> {
            self.0.push(format!("item {n} {on}"));
            Ok(())
        }
        fn measure(&mut self, _: Font, _: u32, _: i32) -> Result<u32, Invalid> {
            Ok(30)
        }
        fn measure_string(&mut self, _: Font, _: u32, _: i32) -> Result<u32, Invalid> {
            Ok(30)
        }
    }

    #[test]
    fn pages() {
        let d = Details {
            controller_item: 9,
            controllers: true,
            name: 1,
            typing: true,
            blink: true,
            nationality: 3,
            nationality_id: 0x700,
            view_id: 0x710,
            second_view_id: Some(0x720),
            race_number: 12,
            codriver_sex: 1,
            sensitivity: 5,
        };
        let mut s = Record::default();
        player_details(&mut s, &d).unwrap();
        assert_eq!(s.0[0], "item 9 true");
        assert_eq!(s.0[2], "Value s4 0x1d9 0xd4");
        assert!(s.0.contains(&"sprite 0x1f 0x1e8 0xf3".to_string()));
        assert!(s.0.contains(&"Value \"12\\0\" 0x1bb 0x140".to_string()));
        assert_eq!(s.0.last().unwrap(), "sprite 0x6 0x1c4 0x176");
        let mut s = Record::default();
        let short = Details {
            typing: false,
            second_view_id: None,
            ..d
        };
        player_details(&mut s, &short).unwrap();
        assert!(!s.0.iter().any(|l| l.contains(" s4 ")));
        assert_eq!(s.0.last().unwrap(), "sprite 0x6 0x1c4 0x15b");
    }
}
