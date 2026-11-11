//! The display page's figures (`debug::draw_wear_bars`, 0x429B19): one
//! function of 875 calls in a row (a bar, a label, a figure each), a table in
//! `debug::display_rows` (generated from it: `tools/debug_pages/`). A bar: the float at its
//! place times 64 over 100 (`NUMBERS.hundred`), in colour 3, 4 high; a label or figure in
//! colour 2. A surface's figure: the name of the dword's low four bits (`off_5FFFCC`). Its
//! last call is the rivals' table (0x435F43). Shown by `debug::ui`.

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

/// The dword at IDA's address `ida`, where it lives now (`moved::named`); None when it
/// cannot be read (shown as such: the UI does not end the game).
pub fn value_at(ida: usize) -> Option<u32> {
    crate::moved::named(ida)
        .and_then(|at| at.read::<u32>("a debug page's value"))
        .ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_place_is_a_variable_of_the_image() {
        // Each figure's place resolves, outside the dead bytes of the race block.
        let groups = crate::debug::display_rows::GROUPS.iter();
        for group in groups.chain(crate::debug::watch_rows::GROUPS) {
            for at in group.rows.iter().map(super::super::ui::Row::at) {
                let place = crate::moved::place(at) as u32;
                assert!(!crate::race::store::is_dead(place), "{at:#x}");
                assert!(super::value_at(at).is_some(), "{at:#x}");
            }
        }
    }
}
