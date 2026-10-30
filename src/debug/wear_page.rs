//! The display page's wear and temperature bars and figures (`debug::draw_wear_bars`,
//! 0x429B19, IDA 2026-10-30): one function of 875 calls in a row, here a table
//! (`wear_items::ITEMS`, generated from it). A bar: the float at its place times 64 over
//! `NUMBERS.hundred` (100.0), rounded, a rectangle from (x, y) that long and 4 high in colour
//! 3; a label or figure: in colour 2 at scale 1 (`debug::draw`). A surface's figure: the
//! name of the dword's low four bits (`off_5FFFCC`). The original's scratch dwords
//! (`dword_62403C`, the rectangle's `dword_5C5FD4` .. `E0`) are not written. Its last call,
//! the rivals' table (`sub_435F43`), is not ported yet.

use crate::debug::draw::{fill_rect, print, print_fixed, print_hex, print_whole};
use crate::errors::Refused;
use crate::race::numbers_store::NUMBERS;
use crate::race::text_state_store::TEXT_STATE;

/// An item: its (x, y) and what it shows (a place is IDA's address of a dword).
#[derive(Clone, Copy, Debug)]
pub enum Item {
    Bar(u32, u32, usize),
    Label(u32, u32, &'static [u8]),
    Int(u32, u32, usize),
    Fixed(u32, u32, usize),
    Hex(u32, u32, usize),
    Surface(u32, u32, usize),
}

/// `off_5FFFCC`: the surfaces' names by kind.
pub const SURFACES: [&[u8]; 16] = [
    b"null",
    b"tarmac",
    b"gravel",
    b"mud",
    b"snow",
    b"water",
    b"dust",
    b"grass",
    b"rough",
    b"ice",
    b"cattlegrid",
    b"thick snow",
    b"snow gravel",
    b"null",
    b"null",
    b"null",
];

/// The dword at IDA's address `ida`, where it lives now (`moved::place`).
fn value_at(ida: usize) -> Result<u32, Refused> {
    Ok(
        crate::pointers::GamePtr::new(crate::moved::place(ida) as u32)
            .read::<u32>("a debug page's value")?,
    )
}

/// `debug::draw_wear_bars` (see the header).
pub fn draw_wear_bars() -> Result<(), Refused> {
    use crate::math::x87_float::{int, ld, to_int};
    use rc2k_core::debug::numbers::bar_right;
    for &item in crate::debug::wear_items::ITEMS.iter() {
        if let Item::Bar(x, y, at) = item {
            TEXT_STATE.text_mode.set(3);
            let length = to_int(int(64) * ld(value_at(at)?) / ld(NUMBERS.hundred.bits()));
            fill_rect([x, y, bar_right(x, length), y + 4])?;
            continue;
        }
        TEXT_STATE.text_mode.set(2);
        NUMBERS.debug_scale.set(1);
        match item {
            Item::Label(x, y, text) => print(x, y, text)?,
            Item::Int(x, y, at) => print_whole(x, y, value_at(at)?)?,
            Item::Fixed(x, y, at) => print_fixed(x, y, value_at(at)?)?,
            Item::Hex(x, y, at) => print_hex(x, y, value_at(at)?)?,
            Item::Surface(x, y, at) => print(x, y, SURFACES[(value_at(at)? & 0xF) as usize])?,
            Item::Bar(..) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_place_is_a_variable_of_the_image() {
        // Each place resolves, outside the dead bytes of the race block.
        for &item in crate::debug::wear_items::ITEMS.iter() {
            let at = match item {
                Item::Label(..) => continue,
                Item::Bar(_, _, at)
                | Item::Int(_, _, at)
                | Item::Fixed(_, _, at)
                | Item::Hex(_, _, at)
                | Item::Surface(_, _, at) => at,
            };
            let place = crate::moved::place(at) as u32;
            assert!(!crate::race::store::is_dead(place), "{at:#x}");
            assert!(value_at(at).is_ok(), "{at:#x}");
        }
    }
}
