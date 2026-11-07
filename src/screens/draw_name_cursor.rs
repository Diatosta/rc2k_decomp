//! `screens::draw_name_cursor` (0x467D6C): the cursor after a player's name being typed.
//!
//!
//! The name (`Player::name`) measured with the font at 0x51E824 (its line count and widest line are
//! left in it); not drawn, the carry is that of `cmp dword_7559A8, 1`; with the text cursor's blink
//! (`dword_7559A8`) at 1, string 4 ("_") at x = 0x1BB + the name's width, y = 0xD4, in the font at
//! 0x51E8C4. Three callers (the player details screens). Refuses no player or no name.

// ---- game data ----

// ---- Rust version ----
