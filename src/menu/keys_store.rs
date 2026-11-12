//! The keyboard pages' tables, out of RAL.EXE's image (IDA 0x755595 .. 0x755640): the
//! keys the redefine page works on, the copy of a player's controls the controller pages keep, and
//! the reserved keys. Bytes only, at addresses one past a multiple of four. A `#[repr(C)]` struct with
//! the image's layout, one static.

use crate::session::{PlainBytes, with_runs};

/// The keys' tables (IDA 0x755595 .. 0x755640).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct KeyTables {
    /// `g_redefinedKeys`: The copy of the player's twelve keys the page works on (`menu::redefine_keys`).
    pub redefined_keys: PlainBytes<14>,
    /// Unreferenced bytes (IDA 0x7555A3 .. 0x7555B1).
    pub _gap_7555a3: PlainBytes<14>,
    /// `unk_7555B1`: The 0x2C bytes of a player's controls the controller pages copy out and back (`menu::keep_player_keys`).
    pub controls: PlainBytes<44>,
    /// Unreferenced bytes (IDA 0x7555DD .. 0x755621).
    pub _gap_7555dd: PlainBytes<68>,
    /// `g_reservedKeys`: The keys that cannot be taken, ended by a NUL.
    pub reserved_keys: PlainBytes<31>,
}

/// First values the image's (`tests::key_tables_starts_as_the_games`).
impl KeyTables {
    pub const fn new() -> Self {
        KeyTables {
            redefined_keys: PlainBytes::new([0; 14]),
            _gap_7555a3: PlainBytes::new([0; 14]),
            controls: PlainBytes::new(with_runs::<44>(&[(
                0x0,
                &[
                    0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
                    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
                    0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
                    0x00, 0x01,
                ],
            )])),
            _gap_7555dd: PlainBytes::new(with_runs::<68>(&[(
                0x0,
                &[0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01],
            )])),
            reserved_keys: PlainBytes::new(with_runs::<31>(&[(
                0x0,
                &[
                    0x70, 0x01, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x57,
                    0x58, 0x63, 0x54, 0x71, 0x78, 0x79, 0x7a, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01,
                    0x01, 0x01,
                ],
            )])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn key_tables_starts_as_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            &crate::menu::owner::menus().keys.tables,
        );
        assert_eq!(offset_of!(KeyTables, redefined_keys), 0x0);
        assert_eq!(offset_of!(KeyTables, controls), 0x1c);
        assert_eq!(offset_of!(KeyTables, reserved_keys), 0x8c);
    }
}
