//! The controller pages' lists: `screens::draw_control_bindings`
//! 0x46A829 (what each of the player's ten controls is bound to) and
//! `screens::draw_redefined_keys` 0x47CFA7 (the keys on a redefine keys page).

use crate::Invalid;
use crate::screens::screen::{Buffer, Screen};
use crate::strings::format::{Pad, axis_name, decimal};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Font {
    /// `stru_51E8C4`.
    Value,
}

/// These pages keep nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {}

/// A binding (the player's +0x50 + 4 x n): dl a button, dh a direction, 0 nothing.
#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub binding: u32,
    /// The device kind (+0x34).
    pub kind: u32,
    /// The row's axis (+0x3B - n), read for kind 3 on the first two rows.
    pub axis: u8,
}

/// The directions' names: `word_755200` for a device kind below 2, else `word_755212`.
pub struct DirectionNames {
    pub pad: [u16; 9],
    pub stick: [u16; 9],
}

/// Row n's text (no NUL): string 0x3BD for nothing; else the direction's name (for kind 3
/// on the first two rows with the axis), then for a button ('+' after a direction) string
/// 0x3C0, a space and its number. Refused: a direction past the name tables (the original
/// reads past them).
pub fn binding_text<S: Screen>(
    s: &mut S,
    b: &Binding,
    n: usize,
    names: &DirectionNames,
) -> Result<Vec<u8>, Invalid> {
    let (dl, dh) = (b.binding as u8, (b.binding >> 8) as u8);
    if b.binding == 0 {
        return Ok(s.string_bytes(0x3BD));
    }
    let mut out = Vec::new();
    if dh != 0 {
        let table = if b.kind < 2 { &names.pad } else { &names.stick };
        let Some(&id) = table.get(dh as usize) else {
            return Err(Invalid("a direction past the name tables"));
        };
        out.extend(s.string_bytes(id as u32));
        if b.kind == 3 && n < 2 {
            out.extend(axis_name(b.axis));
        }
    }
    if dl != 0 {
        if dh != 0 {
            out.push(b'+');
        }
        out.extend(s.string_bytes(0x3C0));
        out.push(b' ');
        out.extend(decimal(dl as u32, Pad::None));
    }
    Ok(out)
}

/// The one-character text's room (`byte_A8BC54` .. 0x100 bytes).
const CELLS_LEN: usize = 0x100;

/// Each text (no NUL) into `byte_A8BC54` and drawn; refused before anything when one is past
/// its room.
fn check_room<E: From<Invalid>>(texts: &[Vec<u8>]) -> Result<(), E> {
    if texts.iter().any(|t| t.len() + 1 > CELLS_LEN) {
        return Err(Invalid("a text past its buffer").into());
    }
    Ok(())
}

fn draw_cells<S: Screen<Font = Font>>(
    s: &mut S,
    text: &[u8],
    x: i32,
    y: i32,
) -> Result<bool, S::Error> {
    let mut t = text.to_vec();
    t.push(0);
    s.store(Buffer::Cells, &t)?;
    Ok(s.draw(Font::Value, Buffer::Cells, x, y)?.too_wide)
}

/// The ten bindings at x 0x1B8 from y 0xD5 every 0x1B; the row being learnt (`learning`,
/// from 0) left out. The carry of the last row (clear when it is the one being learnt).
pub fn control_bindings<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    texts: &[Vec<u8>; 10],
    learning: Option<usize>,
) -> Result<bool, S::Error> {
    check_room(texts)?;
    let mut carry = false;
    for (n, text) in texts.iter().enumerate() {
        if learning == Some(n) {
            carry = false;
            continue;
        }
        carry = draw_cells(s, text, 0x1B8, 0xD5 + 0x1B * n as i32)?;
    }
    Ok(carry)
}

/// The keys' places drawn, in order (place 8 is not).
pub const KEY_PLACES: [usize; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 9, 10, 11];

/// The keys' names (row n at x 0x1CC, y 0xAF + n x 0x1D); at the row being redefined the
/// cursor (string 4) while the blink is 1, else nothing (the carry then that of `cmp
/// blink, 1`). The carry of the last row.
pub fn redefined_keys<S: Screen<Font = Font, Work = Work>>(
    s: &mut S,
    names: &[Vec<u8>; 11],
    redefined: Option<usize>,
    blink: bool,
) -> Result<bool, S::Error> {
    check_room(names)?;
    let mut carry = false;
    for (row, name) in names.iter().enumerate() {
        let y = 0xAF + row as i32 * 0x1D;
        carry = if redefined == Some(row) {
            if blink {
                s.string(Font::Value, 4, 0x1CC, y)?.too_wide
            } else {
                true
            }
        } else {
            draw_cells(s, name, 0x1CC, y)?
        };
    }
    Ok(carry)
}
