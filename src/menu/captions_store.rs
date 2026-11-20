//! The screens' string-id tables and the "screen left" flag: the nationalities, the duel's
//! labels, the places' points, the player views and the lists of captions' string ids.
//! The tables nothing writes are constants; the flag and the caption lists (two entries of
//! which the start-section picks write) are plain fields behind one `RefCell`: the game's thread
//! alone reads and writes them, each borrow lasting one statement.

use crate::memory::session::GameThread;
use std::cell::{RefCell, RefMut};

/// The nationalities' name string ids (`screens::draw_player_details`).
pub const NATIONALITIES: [u32; 47] = [
    0x57D, 0x57E, 0x57F, 0x580, 0x581, 0x582, 0x583, 0x584, 0x585, 0x586, 0x587, 0x588, 0x589,
    0x58A, 0x58B, 0x58C, 0x58D, 0x58E, 0x58F, 0x590, 0x591, 0x592, 0x593, 0x594, 0x595, 0x596,
    0x597, 0x598, 0x599, 0x59A, 0x59B, 0x59C, 0x59D, 0x59E, 0x59F, 0x5A0, 0x5A1, 0x5A2, 0x5A3,
    0x5A4, 0xFFFFFFFF, 0x3C0, 0x3C1, 0x3C2, 0x3C3, 0x3C4, 0xFFFFFFFF,
];

/// The duel result's labels' string ids, by value (-1 ends).
pub const DUEL_LABELS: [u32; 22] = [
    0x4FE, 0x4FF, 0x500, 0x501, 0x502, 0x503, 0x504, 0x505, 0x506, 0x507, 0x508, 0x509, 0x50A,
    0x50B, 0x50C, 0x50D, 0x50E, 0x50F, 0x510, 0x511, 0x512, 0xFFFFFFFF,
];

/// The points of each place (0, 20000, ..., -1 ends).
pub const PLACE_POINTS: [u32; 22] = [
    0x0, 0x4E20, 0x4A38, 0x4650, 0x4268, 0x3E80, 0x3A98, 0x36B0, 0x32C8, 0x2EE0, 0x2AF8, 0x2710,
    0x2328, 0x1F40, 0x1B58, 0x1770, 0x1388, 0xFA0, 0xBB8, 0x7D0, 0x3E8, 0xFFFFFFFF,
];

/// The views' name string ids (`screens::draw_player_details`).
pub const VIEWS: [u32; 8] = [0x378, 0x377, 0x376, 0x37C, 0x379, 0x37A, 0x37B, 0xFFFFFFFF];

/// The second views' name string ids.
pub const SECOND_VIEWS: [u32; 14] = [
    0x377, 0x378, 0x379, 0x37A, 0x37B, 0x37C, 0xFFFFFFFF, 0x37D, 0x37E, 0xFFFFFFFF, 0x186, 0x187,
    0x188, 0xFFFFFFFF,
];

/// The lists of captions' string ids, each ended by -1, as the game starts
/// (`screens::draw_highlighted_caption`, `menu::option_steps`).
const CAPTION_LISTS: [u32; 97] = [
    0x130,
    0x132,
    0x133,
    0x136,
    0x536,
    0xFFFFFFFF,
    0x1D8,
    0x1D9,
    0x1DA,
    0x1DB,
    0x1DD,
    0x13A,
    0xFFFFFFFF,
    0x1DA,
    0x3D6,
    0x3D7,
    0x3D8,
    0x3D9,
    0x3DA,
    0x3DB,
    0xFFFFFFFF,
    0x1D9,
    0x3DC,
    0x3DD,
    0x3DE,
    0x3DF,
    0x3E0,
    0x3E1,
    0xFFFFFFFF,
    0x1E6,
    0x1E8,
    0x1E9,
    0x1EA,
    0x1EB,
    0x13A,
    0xFFFFFFFF,
    0x1DE,
    0x1DF,
    0x13A,
    0xFFFFFFFF,
    0x1E6,
    0x1E7,
    0x13A,
    0xFFFFFFFF,
    0x1EF,
    0x1F0,
    0x1F1,
    0x1F2,
    0x1F3,
    0x1F4,
    0x13A,
    0xFFFFFFFF,
    0x148,
    0x149,
    0x14A,
    0x13A,
    0xFFFFFFFF,
    0x22D,
    0x233,
    0x225,
    0x234,
    0x227,
    0x223,
    0x231,
    0x22E,
    0x229,
    0x222,
    0x224,
    0x228,
    0x230,
    0x22B,
    0x235,
    0x236,
    0x1038,
    0xFFFFFFFF,
    0x22F,
    0x232,
    0x226,
    0x22C,
    0x22A,
    0x237,
    0xFFFFFFFF,
    0x0,
    0x1,
    0x2,
    0x3,
    0x4,
    0x5,
    // The single rally's players menu (carousel menu 0x15): the in-turn items'
    // names, then the split screens of three and four players (own texts) and back.
    0x1E6,
    0x1E8,
    0x1E9,
    0x1EA,
    0x1EB,
    rc2k_core::strings::own::id::SPLIT_THREE,
    rc2k_core::strings::own::id::SPLIT_FOUR,
    0x13A,
    0xFFFFFFFF,
];

/// The screens' string-id values that change.
#[derive(Clone, Copy)]
pub struct CaptionValues {
    /// True when the screen was left with a key (Escape in `handle_keys`, a yes / no answer, a
    /// name entry); the loop puts its leave question on it and the screens that look at it clear it.
    pub left_by_key: bool,
    /// The lists of captions' string ids, each ended by -1 (`screens::draw_highlighted_caption`,
    /// `menu::option_steps`).
    pub caption_lists: [u32; 97],
}

/// The string-id tables that change.
pub struct CaptionTables(GameThread<RefCell<CaptionValues>>);

impl CaptionTables {
    pub const fn new() -> Self {
        CaptionTables(GameThread::new(RefCell::new(CaptionValues {
            left_by_key: false,
            caption_lists: CAPTION_LISTS,
        })))
    }

    /// A copy of the values: no borrow outlives the statement that reads them.
    pub fn read(&self) -> CaptionValues {
        *self.0.get().borrow()
    }

    /// The values, to write.
    pub fn write(&self) -> RefMut<'_, CaptionValues> {
        self.0.get().borrow_mut()
    }
}
