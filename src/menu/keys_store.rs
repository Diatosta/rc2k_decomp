//! The keyboard pages' tables, out of RAL.EXE's image (IDA 0x755595 .. 0x755640; phase 7, typed): the
//! keys the redefine page works on, the copy of a player's controls the controller pages keep, and
//! the reserved keys. Bytes only, at addresses one past a multiple of four. A `#[repr(C)]` struct with
//! the image's layout, one static; `moved::MOVED` points the range at it. (Split 2026-10-25 from the
//! front end's block `menu::store`.)

use crate::session::{Bytes, with_runs};

/// The keys' tables (IDA 0x755595 .. 0x755640).
#[repr(C)]
pub struct KeyTables {
    /// `g_redefinedKeys`: The copy of the player's twelve keys the page works on (`menu::redefine_keys`).
    pub redefined_keys: Bytes<14>,
    /// Unreferenced bytes (IDA 0x7555A3 .. 0x7555B1).
    pub _gap_7555a3: Bytes<14>,
    /// `unk_7555B1`: The 0x2C bytes of a player's controls the controller pages copy out and back (`menu::keep_player_keys`).
    pub controls: Bytes<44>,
    /// Unreferenced bytes (IDA 0x7555DD .. 0x755621).
    pub _gap_7555dd: Bytes<68>,
    /// `g_reservedKeys`: The keys that cannot be taken, ended by a NUL.
    pub reserved_keys: Bytes<31>,
}

/// First values the image's (`tests::key_tables_starts_as_the_games`).
pub static KEY_TABLES: KeyTables = KeyTables {
    redefined_keys: Bytes::new([0; 14]),
    _gap_7555a3: Bytes::new([0; 14]),
    controls: Bytes::new(with_runs::<44>(&[(
        0x0,
        &[
            0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        ],
    )])),
    _gap_7555dd: Bytes::new(with_runs::<68>(&[(
        0x0,
        &[0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01],
    )])),
    reserved_keys: Bytes::new(with_runs::<31>(&[(
        0x0,
        &[
            0x70, 0x01, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x57, 0x58,
            0x63, 0x54, 0x71, 0x78, 0x79, 0x7a, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
        ],
    )])),
};

/// IDA 0x755595, the first byte of the piece.
pub const START: usize = 0x755595;
/// IDA 0x755640, the end of the piece.
pub const END: usize = 0x755640;

/// Where IDA 0x755595 is now (`moved::MOVED`'s home).
pub fn key_tables_addr() -> u32 {
    crate::pointers::game_address(&raw const KEY_TABLES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn key_tables_starts_as_the_games() {
        assert!(
            END - START <= size_of::<KeyTables>()
                && size_of::<KeyTables>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(key_tables_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(KeyTables, redefined_keys), 0x755595);
        at(offset_of!(KeyTables, controls), 0x7555B1);
        at(offset_of!(KeyTables, reserved_keys), 0x755621);
    }
}
