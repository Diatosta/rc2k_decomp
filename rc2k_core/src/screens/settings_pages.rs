//! The settings pages' drawing: `screens::draw_sound_settings`
//! 0x47022A, `draw_graphics_settings`, `draw_visual_preferences`, `draw_volume_settings`
//! and `draw_options_checklist`: each setting's name (a string by the setting's value) in
//! its place, the volume bars, the options' check marks.

use crate::screens::screen::{BAR_HAS_NO_LENGTH, Bar, Buffer, Drawn, Screen};
use crate::strings::format::{Pad, decimal};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// The settings' names (`stru_51E8C4`).
    Value,
    /// The volume page's ends (`stru_51E3C4`).
    Small,
    /// The checklist's title (`stru_51E904`).
    Title,
    /// The checklist's heads (`stru_51E944`).
    Head,
    /// The checklist's lines (`stru_51E8E4`).
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `dword_7559EC`.
    Scratch,
    /// `seconds` (0x7559AC).
    Seconds,
    /// `dword_7559F0`.
    Glyph,
    /// `dword_755A18`: the check marks' x.
    CheckX,
}

/// The names: each string `id` kept and drawn at (0x1D1, y). What the last one leaves.
fn names<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    rows: impl IntoIterator<Item = (u32, i32)>,
) -> Result<Drawn, S::Error> {
    let mut last = Drawn::default();
    for (id, y) in rows {
        s.keep(Work::Scratch, id);
        last = s.string(Font::Value, id, 0x1D1, y)?;
    }
    Ok(last)
}

/// The sound page's menu items, in the order the original sets them (true: choosable):
/// 4 .. 6 on, then off those the card lacks (3D sound, EAX, A3D 2).
pub fn sound_items(has_3d: bool, has_eax: bool, has_a3d2: bool) -> Vec<(u32, bool)> {
    let mut out = vec![(4, true), (5, true), (6, true)];
    for (has, item) in [(has_3d, 4), (has_eax, 5), (has_a3d2, 6)] {
        if !has {
            out.push((item, false));
        }
    }
    out
}

/// The sound page's six names (each a string id), from y 0x10B every 0x1B.
pub fn sound<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    ids: [u32; 6],
) -> Result<Drawn, S::Error> {
    names(s, ids.into_iter().zip((0..).map(|n| 0x10B + 0x1B * n)))
}

/// The graphics page's item 2 is choosable when both its switches are on (they sum to 2).
pub fn graphics_item(a: u32, b: u32) -> bool {
    a.wrapping_add(b) == 2
}

/// The graphics page: eight names at rows 0, 2 .. 8 (y 0xDC every 0x1B); after the first,
/// the screen mode "w x h x depth" in the text buffer at (0x1D1, 0xF7).
pub fn graphics<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    ids: [u32; 8],
    mode: [u32; 3],
) -> Result<Drawn, S::Error> {
    let mut last = Drawn::default();
    for (n, (id, row)) in ids.into_iter().zip([0, 2, 3, 4, 5, 6, 7, 8]).enumerate() {
        last = names(s, [(id, 0xDC + 0x1B * row)])?;
        if n == 0 {
            let mut line = decimal(mode[0], Pad::None);
            line.push(b'x');
            line.extend(decimal(mode[1], Pad::None));
            line.push(b'x');
            line.extend(decimal(mode[2], Pad::None));
            line.push(0);
            s.store(Buffer::Text, &line)?;
            s.draw(Font::Value, Buffer::Text, 0x1D1, 0xF7)?;
        }
    }
    Ok(last)
}

/// The visual preferences page: nine names from y 0xC8 every 0x1D. The carry of the last.
pub fn visual_preferences<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    ids: [u32; 9],
) -> Result<bool, S::Error> {
    Ok(names(s, ids.into_iter().zip((0..).map(|n| 0xC8 + 0x1D * n)))?.too_wide)
}

/// The volume page: the name `id` (kept in `seconds`) at (0x1EA, 0x1B3); eight bar
/// backgrounds (sprite 0x19 at x 0x222, y 0xDA every 0x1C); eight bars of the volumes (0 ..
/// 0x40: v x 100 / 0x40 percent; a v whose v x 100 wraps to 0 is fatal error 0x80701010);
/// then each row's knob (sprite 0x1B at x 0x1E6) and its two ends (strings 0x547, 0x548).
/// The sprites' palette is where the last text ended.
pub fn volume<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    id: u32,
    volumes: [u32; 8],
) -> Result<(), S::Error> {
    s.keep(Work::Seconds, id);
    s.string(Font::Value, id, 0x1EA, 0x1B3)?;
    for n in 0..8 {
        s.sprite(0x19, 0x222, 0xDA + 0x1C * n)?;
    }
    for (n, v) in volumes.into_iter().enumerate() {
        let percent = if v == 0 {
            0
        } else {
            let p = v.wrapping_mul(100);
            if p == 0 {
                return Err(s.fatal(BAR_HAS_NO_LENGTH, ""));
            }
            p / 0x40
        };
        s.bar(Bar {
            x: 0x221,
            y: 0xDA + 0x1C * n as i32,
            width: 0x76,
            fill_glyph: 0x18,
            end_glyph: 0x87,
            percent: percent as i32,
        })?;
    }
    for n in 0..8 {
        let y = 0xD9 + 0x1C * n;
        s.sprite(0x1B, 0x1E6, y)?;
        s.string(Font::Small, 0x547, 0x203, y + 4)?;
        s.string(Font::Small, 0x548, 0x2B7, y + 4)?;
    }
    Ok(())
}

/// The options checklist's lines: string, x, y, font.
pub const CHECKLIST_LINES: [(u32, i32, i32, Font); 14] = [
    (0x524, 0x190, 0x64, Font::Title),
    (0x525, 0x186, 0x82, Font::Head),
    (0x526, 0x186, 0x9D, Font::Head),
    (0x527, 0x186, 0xB8, Font::Head),
    (0x528, 0x186, 0xD3, Font::Line),
    (0x529, 0x186, 0xEE, Font::Line),
    (0x52A, 0x186, 0x109, Font::Line),
    (0x52B, 0x186, 0x124, Font::Line),
    (0x52C, 0x186, 0x13F, Font::Line),
    (0x52D, 0x186, 0x15A, Font::Line),
    (0x52E, 0x186, 0x175, Font::Line),
    (0x52F, 0x186, 0x190, Font::Line),
    (0x530, 0x186, 0x1AB, Font::Line),
    (0x531, 0x186, 0x1C6, Font::Line),
];

/// The options checklist: the title (string 0x190, " ", the player's number from 1) in
/// `byte_A8BC54` at (0x190, 0x14), the lines, the progress bar at `percent`, its end
/// sprite 0x88 and labels (0x4AE, 0x4AF), then a check mark (sprite 0x9E + (v ^ 1)) for
/// each of `ticks` from y 0x82 every 0x1B, at x 0x190 (kept), its palette v ^ 1.
pub fn options_checklist<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    player: u32,
    percent: i32,
    ticks: &[u32],
) -> Result<(), S::Error> {
    let mut title = s.string_bytes(0x190);
    title.push(b' ');
    title.extend(decimal(player.wrapping_add(1), Pad::None));
    title.push(0);
    s.store(Buffer::Cells, &title)?;
    s.draw(Font::Title, Buffer::Cells, 0x190, 0x14)?;
    for (id, x, y, font) in CHECKLIST_LINES {
        s.string(font, id, x, y)?;
    }
    s.bar(Bar {
        x: 0x128,
        y: 0x1E3,
        width: 0xD0,
        fill_glyph: 0x89,
        end_glyph: 0x87,
        percent,
    })?;
    s.sprite(0x88, 0xEC, 0x1E2)?;
    s.string(Font::Small, 0x4AE, 0x10A, 0x1E6)?;
    s.string(Font::Small, 0x4AF, 0x217, 0x1E6)?;
    s.keep(Work::CheckX, 0x190);
    let mut y = 0x82u32;
    for &t in ticks {
        s.keep(Work::Scratch, y);
        let v = t ^ 1;
        let glyph = 0x9E_u32.wrapping_add(v);
        s.keep(Work::Glyph, glyph);
        s.sprite(glyph, 0x190, y as i32)?;
        y += 0x1B;
        s.keep(Work::Scratch, y);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Invalid;

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
        fn draw_clock(&mut self, _: Font, _: Buffer, _: i32, _: i32) -> Result<(), Invalid> {
            unreachable!()
        }
        fn string(&mut self, f: Font, id: u32, x: i32, y: i32) -> Result<Drawn, Invalid> {
            self.0.push(format!("{f:?} s{id:x} {x:#x} {y:#x}"));
            Ok(Drawn { ..Drawn::default() })
        }
        fn string_bytes(&mut self, id: u32) -> Vec<u8> {
            format!("<{id:x}>").into_bytes()
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
        fn bar(&mut self, b: Bar) -> Result<(), Invalid> {
            self.0.push(format!("bar {} {}", b.y, b.percent));
            Ok(())
        }
        fn fatal(&mut self, _: u32, m: &'static str) -> Invalid {
            Invalid(m)
        }
    }

    #[test]
    fn pages() {
        assert_eq!(
            sound_items(true, false, true),
            [(4, true), (5, true), (6, true), (5, false)]
        );
        let mut s = Record::default();
        graphics(&mut s, [1, 2, 3, 4, 5, 6, 7, 8], [640, 480, 16]).unwrap();
        assert_eq!(s.0[2], "Value \"640x480x16\\0\" 0x1d1 0xf7");
        assert_eq!(s.0[4], "Value s2 0x1d1 0x112");
        let mut s = Record::default();
        volume(&mut s, 0x56D, [0x40, 0, 0x20, 0, 0, 0, 0, 1]).unwrap();
        assert!(s.0.contains(&"bar 218 100".to_string()));
        assert!(s.0.contains(&"bar 274 50".to_string()));
        assert!(s.0.contains(&"bar 414 1".to_string()));
        let mut s = Record::default();
        assert!(volume(&mut s, 0, [0x0290_0000, 0, 0, 0, 0, 0, 0, 0]).is_ok());
        let mut s = Record::default();
        options_checklist(&mut s, 1, 30, &[1, 0]).unwrap();
        assert_eq!(s.0[0], "Title \"<190> 2\\0\" 0x190 0x14");
        assert!(s.0.contains(&"sprite 0x9e 0x190 0x82".to_string()));
        assert!(s.0.contains(&"sprite 0x9f 0x190 0x9d".to_string()));
    }
}
