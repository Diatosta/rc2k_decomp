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

pub const HISTORY_ENTRY_LEN: u32 = 0x78;

/// `screens::find_history_entry`: from entry 0 on, the number of the first whose class
/// (+0x10) and position (+0x70) are these (`fields(n)`); entry 0 is looked at before any
/// end test, the others only if their first dword (`first(n)`) is not -1. `None`: the end
/// met first (the original's fatal error 0x8070A7FA).
pub fn find_history(
    mut first: impl FnMut(u32) -> u32,
    mut fields: impl FnMut(u32) -> (u32, u32),
    class: u32,
    position: u32,
) -> Option<u32> {
    let mut n = 0;
    loop {
        if fields(n) == (class, position) {
            return Some(n);
        }
        n += 1;
        if first(n) == u32::MAX {
            return None;
        }
    }
}

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
    for (id, x, font) in [
        (0x484u32, 0x73, Font::Position),
        (0x485, 0x9C, Font::Text),
        (0x4F9, 0x2DE, Font::Text),
    ] {
        s.string(font, id, x, 0xDC)?;
    }
    let flags = if event_mode_13 { 0x90 } else { 0x8A };
    for n in 0..6u32 {
        s.sprite(flags + n, 0x196 + 0x36 * n as i32, 0x8C)?;
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
        s.sprite(flag_glyph(r.nation), 0x7E, y)?;
        s.sprite(car_glyph(r.car), 0x154, y)?;
        y = y.wrapping_add(0x19);
        s.keep(Work::Position, first.wrapping_add(n + 1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_search() {
        let table = [(5, 1, 3), (0, 0, 1), (0, 1, 1), (u32::MAX, 1, 2)];
        let first = |n: u32| table[n as usize].0;
        let fields = |n: u32| (table[n as usize].1, table[n as usize].2);
        assert_eq!(find_history(first, fields, 1, 3), Some(0));
        assert_eq!(find_history(first, fields, 1, 1), Some(2));
        // The end's own fields are never looked at.
        assert_eq!(find_history(first, fields, 1, 2), None);
    }
}
