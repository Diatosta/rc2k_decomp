//! The keyboard pages' tables: the
//! keys the redefine page works on, the copy of a player's controls the controller pages keep, and
//! the reserved keys. Bytes only, at addresses one past a multiple of four. One static.

use crate::memory::session::with_runs;

/// The keys' tables.
pub struct KeyTables {
    /// The copy of the player's twelve keys the page works on (`menu::redefine_keys`).
    pub redefined_keys: [u8; 14],
    /// The 0x2C bytes of a player's controls the controller pages copy out and back
    /// (`menu::keep_player_keys`).
    pub controls: [u8; 44],
    /// The keys that cannot be taken, ended by a NUL.
    pub reserved_keys: [u8; 31],
}

impl KeyTables {
    pub const fn new() -> Self {
        KeyTables {
            redefined_keys: [0; 14],
            controls: with_runs::<44>(&[(
                0x0,
                &[
                    0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
                    0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
                    0x00, 0x01,
                ],
            )]),
            reserved_keys: with_runs::<31>(&[(
                0x0,
                &[
                    0x70, 0x01, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x57,
                    0x58, 0x63, 0x54, 0x71, 0x78, 0x79, 0x7a, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01,
                    0x01, 0x01,
                ],
            )]),
        }
    }
}
