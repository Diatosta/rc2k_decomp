//! The championship's standings table (converted 2026-10-20 from
//! `screens::draw_championship_table` 0x4A4E1E): the heads, the six rounds' flags and ten
//! rows of the championship history (0x78-byte entries): position, name, the six rounds'
//! points ("-" for below 0), the total, the nationality's and the car's sprites.

use crate::screens::best_scores::{car_glyph, flag_glyph};
use crate::screens::screen::Screen;
use crate::strings::format::{Pad, decimal};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E5A4`, `stru_51E584`: a position, the rest.
    Position,
    Text,
    /// `stru_51E544`, `stru_51E524`: the same for the player's own rows.
    MarkedPosition,
    MarkedText,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `dword_755579`: the position drawn.
    Position,
    /// `dword_7556BC`: the position after the last.
    End,
}

/// A history entry's row: the player's own (+4 above 0), nationality (+8), car (+0xC), name
/// (+0x14), the rounds' points (+0x54 .. +0x68), total (+0x6C), position (+0x70).
#[derive(Clone, Copy, Debug)]
pub struct Row<T> {
    pub mine: bool,
    pub nation: u32,
    pub car: u32,
    pub name: T,
    pub points: [i32; 6],
    pub total: u32,
    pub position: u32,
}

const ROUNDS_X: [i32; 6] = [0x19A, 0x1D0, 0x206, 0x23C, 0x272, 0x2A8];

/// The table: the heads (strings 0x484, 0x485, 0x4F9 at y 0xDC), the rounds' flags
/// (sprites 0x90 .. 0x95 in event mode 0x13, else 0x8A .. 0x8F, at y 0x8C, the palette
/// where the last head ended), then the rows from `first`, each text from the last one's y,
/// rows 0x19 apart from y 0xF0, the sprites' palette where the total ended.
pub fn championship_table<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    first: u32,
    event_mode_13: bool,
    rows: &[Row<S::Text>; 10],
) -> Result<(), S::Error> {
    let mut palette = 0;
    for (id, x, font) in [
        (0x484u32, 0x73, Font::Position),
        (0x485, 0x9C, Font::Text),
        (0x4F9, 0x2DE, Font::Text),
    ] {
        palette = s.string(font, id, x, 0xDC)?.after;
    }
    let flags = if event_mode_13 { 0x90 } else { 0x8A };
    for n in 0..6u32 {
        s.sprite(flags + n, 0x196 + 0x36 * n as i32, 0x8C, palette)?;
    }
    s.keep(Work::Position, first);
    s.keep(Work::End, first.wrapping_add(10));
    let mut y = 0xF0;
    for (n, r) in (0u32..).zip(rows) {
        let (position_font, font) = if r.mine {
            (Font::MarkedPosition, Font::MarkedText)
        } else {
            (Font::Position, Font::Text)
        };
        y = s
            .line(position_font, &decimal(r.position, Pad::None), 0x73, y)?
            .y;
        y = s.text(font, r.name, 0x9C, y)?.y;
        for (&p, x) in r.points.iter().zip(ROUNDS_X) {
            let text = if p < 0 {
                b"-".to_vec()
            } else {
                decimal(p as u32, Pad::None)
            };
            y = s.line(font, &text, x, y)?.y;
        }
        let total = s.line(font, &decimal(r.total, Pad::None), 0x2DE, y)?;
        y = total.y;
        s.sprite(flag_glyph(r.nation), 0x7E, y, total.after)?;
        s.sprite(car_glyph(r.car), 0x154, y, total.after)?;
        y = y.wrapping_add(0x19);
        s.keep(Work::Position, first.wrapping_add(n + 1));
    }
    Ok(())
}
