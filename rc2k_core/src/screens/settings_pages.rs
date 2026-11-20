//! The settings pages' drawing (sound, graphics, visual preferences, volume, options
//! checklist): each setting's name (a string by the setting's value) in its place, the volume
//! bars, the options' check marks.

use crate::screens::screen::{BAR_HAS_NO_LENGTH, Bar, Buffer, Drawn, Screen};
use crate::strings::format::{Pad, decimal};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// The settings' names.
    Value,
    /// The volume page's ends.
    Small,
    /// The checklist's title.
    Title,
    /// The checklist's heads.
    Head,
    /// The checklist's lines.
    Line,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// The names: each string `id` kept and drawn at (0x1D1, y). What the last one leaves.
fn names<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    rows: impl IntoIterator<Item = (u32, i32)>,
) -> Result<Drawn, S::Error> {
    let mut last = Drawn::default();
    for (id, y) in rows {
        last = s.string(Font::Value, id, 0x1D1, y)?;
    }
    Ok(last)
}

/// The sound page's menu items in the order they are set (true: choosable): 4 and 5 on, then
/// off those the card lacks (3D sound, EAX).
pub fn sound_items(has_3d: bool, has_eax: bool) -> Vec<(u32, bool)> {
    let mut out = vec![(4, true), (5, true)];
    for (has, item) in [(has_3d, 4), (has_eax, 5)] {
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

/// The graphics page's first row (the original's 0xDC, moved up three rows for the rows below the
/// last of the original's, so none lies over the page's buttons).
const GRAPHICS_TOP: i32 = 0xDC - 3 * 0x1B;

/// The graphics page: eight names at rows 0, 2 .. 8 (y 0xDC every 0x1B); after the first,
/// the screen mode "w x h x depth" in the text buffer at (0x1D1, 0xF7).
pub fn graphics<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    ids: [u32; 8],
    mode: &[u8],
) -> Result<Drawn, S::Error> {
    let mut last = Drawn::default();
    for (n, (id, row)) in ids.into_iter().zip([0, 2, 3, 4, 5, 6, 7, 8]).enumerate() {
        last = names(s, [(id, GRAPHICS_TOP + 0x1B * row)])?;
        if n == 0 {
            let mut line = mode.to_vec();
            line.push(0);
            s.store(Buffer::Text, &line)?;
            s.draw(Font::Value, Buffer::Text, 0x1D1, GRAPHICS_TOP + 0x1B)?;
        }
    }
    Ok(last)
}

/// The graphics page's frame limit: `text` (no NUL) in the names' column on row 10, below the
/// rain and snow row.
pub fn graphics_frame_limit<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    text: &[u8],
) -> Result<Drawn, S::Error> {
    s.line(Font::Value, text, 0x1D1, GRAPHICS_TOP + 0x1B * 10)
}

/// The graphics page's rain and snow switch: `text` (no NUL) in the names' column on row 9, after
/// the headlights.
pub fn graphics_rain_and_snow<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    text: &[u8],
) -> Result<Drawn, S::Error> {
    s.line(Font::Value, text, 0x1D1, GRAPHICS_TOP + 0x1B * 9)
}

/// The graphics page's 3D resolution: `text` (no NUL) in the names' column on row 11.
pub fn graphics_render_scale<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    text: &[u8],
) -> Result<Drawn, S::Error> {
    s.line(Font::Value, text, 0x1D1, GRAPHICS_TOP + 0x1B * 11)
}

/// The graphics page's HUD area and screen mode: `hud_box` and `window` (no NUL) in the names'
/// column on rows 12 and 13.
pub fn graphics_display<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    hud_box: &[u8],
    window: &[u8],
) -> Result<Drawn, S::Error> {
    s.line(Font::Value, hud_box, 0x1D1, GRAPHICS_TOP + 0x1B * 12)?;
    s.line(Font::Value, window, 0x1D1, GRAPHICS_TOP + 0x1B * 13)
}

/// The advanced page's values: `texts[n]` (no NUL) in the names' column on row `n`.
pub fn advanced<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    texts: &[Vec<u8>],
) -> Result<Drawn, S::Error> {
    let mut last = Drawn::default();
    for (row, text) in texts.iter().enumerate() {
        last = s.line(Font::Value, text, 0x1D1, 0xBE + 0x1B * row as i32)?;
    }
    Ok(last)
}

/// The visual preferences page: nine names from y 0xC8 every 0x1D. Returns the last name's
/// `too_wide`.
pub fn visual_preferences<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    ids: [u32; 9],
) -> Result<bool, S::Error> {
    Ok(names(s, ids.into_iter().zip((0..).map(|n| 0xC8 + 0x1D * n)))?.too_wide)
}

/// The volume page: the race music's switch name `id` at (0x1EA, 0x1CF), in the last row (9); nine
/// bar backgrounds (sprite 0x19 at x 0x222, y 0xDA every 0x1C) for the volumes in rows 0 .. 8, so
/// the sliders are together and the switch is at the bottom; nine bars of the volumes (0 .. 0x40, as
/// percent; a v whose v x 100 wraps to 0 is fatal error 0x80701010); each row's knob (sprite
/// 0x1B) and its two ends (strings 0x547, 0x548).
/// The volume page's rows: the nine volumes (the last the films').
const VOLUME_ROWS: [i32; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];

pub fn volume<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    id: u32,
    volumes: [u32; 9],
) -> Result<(), S::Error> {
    s.string(Font::Value, id, 0x1EA, 0x1B3 + 0x1C)?;
    for n in VOLUME_ROWS {
        s.sprite(0x19, 0x222, 0xDA + 0x1C * n)?;
    }
    for (n, v) in VOLUME_ROWS.into_iter().zip(volumes) {
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
            y: 0xDA + 0x1C * n,
            width: 0x76,
            fill_glyph: 0x18,
            end_glyph: 0x87,
            percent: percent as i32,
        })?;
    }
    for n in VOLUME_ROWS {
        let y = 0xD9 + 0x1C * n;
        s.sprite(0x1B, 0x1E6, y)?;
        s.string(Font::Small, 0x547, 0x203, y + 4)?;
        s.string(Font::Small, 0x548, 0x2B7, y + 4)?;
    }
    Ok(())
}

/// The options checklist's lines: string, x, y, font.
pub(crate) const CHECKLIST_LINES: [(u32, i32, i32, Font); 14] = [
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

/// The options checklist: the title (string 0x190, " ", the player's number from 1) at (0x190,
/// 0x14), the lines, the progress bar at `percent` with its end sprite 0x88 and labels (0x4AE,
/// 0x4AF), then a check mark (sprite 0x9E + (v ^ 1)) for each of `ticks` from y 0x82 every 0x1B.
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
    let mut y = 0x82u32;
    for &t in ticks {
        let v = t ^ 1;
        let glyph = 0x9E_u32.wrapping_add(v);
        s.sprite(glyph, 0x190, y as i32)?;
        y += 0x1B;
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
        assert_eq!(sound_items(true, false), [(4, true), (5, true), (5, false)]);
        let mut s = Record::default();
        graphics(&mut s, [1, 2, 3, 4, 5, 6, 7, 8], b"640 x 480 (4:3)").unwrap();
        assert_eq!(s.0[1], "Value \"640 x 480 (4:3)\\0\" 0x1d1 0xa6");
        assert_eq!(s.0[2], "Value s2 0x1d1 0xc1");
        let mut s = Record::default();
        volume(&mut s, 0x56D, [0x40, 0, 0x20, 0, 0, 0, 0, 1, 0x10]).unwrap();
        assert!(s.0.contains(&"bar 218 100".to_string()));
        assert!(s.0.contains(&"bar 274 50".to_string()));
        assert!(s.0.contains(&"bar 414 1".to_string()));
        // The films' volume is the last slider (row 8); the music switch's row 9 has no bar.
        assert!(s.0.contains(&"bar 442 25".to_string()));
        assert!(!s.0.iter().any(|l| l.starts_with("bar 470")));
        let mut s = Record::default();
        assert!(volume(&mut s, 0, [0x0290_0000, 0, 0, 0, 0, 0, 0, 0, 0]).is_ok());
        let mut s = Record::default();
        options_checklist(&mut s, 1, 30, &[1, 0]).unwrap();
        assert_eq!(s.0[0], "Title \"<190> 2\\0\" 0x190 0x14");
        assert!(s.0.contains(&"sprite 0x9e 0x190 0x82".to_string()));
        assert!(s.0.contains(&"sprite 0x9f 0x190 0x9d".to_string()));
    }
}
