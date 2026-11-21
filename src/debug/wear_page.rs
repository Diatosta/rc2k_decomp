//! The display page's figures: a table of 875 entries (a bar, a label, a figure each), listed
//! as rows in `debug::display_rows`. A bar: the float at its place times 64 over 100, in colour 3, 4 high;
//! a label or figure in colour 2. A surface's figure: the name of the dword's low four bits. Its
//! last call is the rivals' table. Shown by `debug::ui`.

/// The surfaces' names by kind.
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
