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
