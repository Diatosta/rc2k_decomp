//! The display page's items (`debug::draw_wear_bars`, 0x429B19, IDA 2026-10-30): one function
//! of 875 calls in a row (a bar, a label, a figure each; `wear_items::ITEMS`, generated from
//! it). A bar: the float at its place times 64 over 100 (`NUMBERS.hundred`), drawn in colour
//! 3, 4 high; a label or figure in colour 2. A surface's figure: the name of the dword's low
//! four bits (`off_5FFFCC`). Its last call is the rivals' table (`sub_435F43`). Shown by
//! `debug::ui` (the "Display" and "Rivals" tabs).

/// An item: its (x, y) and what it shows (a place is IDA's address of a dword).
#[derive(Clone, Copy, Debug)]
#[allow(
    dead_code,
    reason = "the bars' and labels' places are the original's, kept in the table generated from IDA"
)]
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

/// The dword at IDA's address `ida`, where it lives now (`moved::place`); None when it
/// cannot be read (shown as such: the UI does not end the game).
pub fn value_at(ida: usize) -> Option<u32> {
    crate::pointers::GamePtr::new(crate::moved::place(ida) as u32)
        .read::<u32>("a debug page's value")
        .ok()
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
            assert!(value_at(at).is_some(), "{at:#x}");
        }
    }
}
