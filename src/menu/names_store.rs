//! The players' names and the text being typed, out of RAL.EXE's image (IDA 0x755784 .. 0x7558C4; phase
//! 7, typed): the four players' names (0x20 bytes each) and what follows them, the name `menu::standings`
//! keeps, and the text as it was before an entry started. A `#[repr(C)]` struct with the image's
//! layout, one static; `moved::MOVED` points the range at it. (Split 2026-10-25 from the front end's
//! block `menu::store`.)

use crate::session::Bytes;

/// The names and the text backup (IDA 0x755784 .. 0x7558C4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Names {
    /// `g_playerNames`: The four players' names, 0x20 bytes each (`menu::player_names`).
    pub player_names: Bytes<128>,
    /// `byte_7557E4`: The rest of the names' area.
    pub more_names: Bytes<128>,
    /// `g_keptPlayerName`: The name kept by `menu::standings` (0x1F bytes and a NUL).
    pub kept_name: Bytes<32>,
    /// `unk_7558A4`: The text as it was before the entry started (`menu::text_entry`).
    pub text_backup: Bytes<32>,
}

/// First values the image's (`tests::name_buffers_starts_as_the_games`).
pub static NAME_BUFFERS: Names = Names {
    player_names: Bytes::new([0; 128]),
    more_names: Bytes::new([0; 128]),
    kept_name: Bytes::new([0; 32]),
    text_backup: Bytes::new([0; 32]),
};

/// IDA 0x755784, the first byte of the piece.
pub const START: usize = 0x755784;
/// IDA 0x7558C4, the end of the piece.
pub const END: usize = 0x7558C4;

/// Where IDA 0x755784 is now (`moved::MOVED`'s home).
#[cfg(test)]
pub fn names_addr() -> u32 {
    crate::pointers::game_address(&raw const NAME_BUFFERS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moved::image_bytes;
    use core::mem::{offset_of, size_of};

    #[test]
    fn name_buffers_starts_as_the_games() {
        assert!(
            END - START <= size_of::<Names>()
                && size_of::<Names>() <= (END - START).next_multiple_of(4)
        );
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host(names_addr()).cast_const(),
                END - START,
            )
        };
        assert_eq!(mine, &image_bytes(START, END - START)[..]);
        let at = |field: usize, ida: usize| assert_eq!(field, ida - START, "{ida:#X}");
        at(offset_of!(Names, player_names), 0x755784);
        at(offset_of!(Names, more_names), 0x755804);
        at(offset_of!(Names, kept_name), 0x755884);
        at(offset_of!(Names, text_backup), 0x7558A4);
    }
}
