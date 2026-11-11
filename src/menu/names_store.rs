//! The players' names and the text being typed, out of RAL.EXE's image (IDA 0x755784 .. 0x7558C4; phase
//! 7, typed): the four players' names (0x20 bytes each) and what follows them, the name `menu::standings`
//! keeps, and the text as it was before an entry started. A `#[repr(C)]` struct with the image's
//! layout, one static; `moved::MOVED` points the range at it.

use crate::session::PlainBytes;

/// The names and the text backup (IDA 0x755784 .. 0x7558C4).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Names {
    /// `g_playerNames`: The four players' names, 0x20 bytes each (`menu::player_names`).
    pub player_names: PlainBytes<128>,
    /// `Names.player_names`: The rest of the names' area.
    pub more_names: PlainBytes<128>,
    /// `g_keptPlayerName`: The name kept by `menu::standings` (0x1F bytes and a NUL).
    pub kept_name: PlainBytes<32>,
    /// `unk_7558A4`: The text as it was before the entry started (`menu::text_entry`).
    pub text_backup: PlainBytes<32>,
}

/// The whole area as one run of bytes: the four names, the rest of the names, the kept name and
/// the text backup (a name that lacks its end runs on into what follows it).
impl crate::place::Run for Names {
    fn len(&self) -> usize {
        size_of::<Names>()
    }
    fn get(&self, n: usize) -> u8 {
        match n {
            0..0x80 => self.player_names.get(n),
            0x80..0x100 => self.more_names.get(n - 0x80),
            0x100..0x120 => self.kept_name.get(n - 0x100),
            _ => self.text_backup.get(n - 0x120),
        }
    }
    fn set(&self, n: usize, v: u8) {
        match n {
            0..0x80 => self.player_names.set(n, v),
            0x80..0x100 => self.more_names.set(n - 0x80, v),
            0x100..0x120 => self.kept_name.set(n - 0x100, v),
            _ => self.text_backup.set(n - 0x120, v),
        }
    }
}

/// First values the image's (`tests::name_buffers_starts_as_the_games`).
impl Names {
    pub const fn new() -> Self {
        Names {
            player_names: PlainBytes::new([0; 128]),
            more_names: PlainBytes::new([0; 128]),
            kept_name: PlainBytes::new([0; 32]),
            text_backup: PlainBytes::new([0; 32]),
        }
    }
}

/// IDA 0x755784, the first byte of the piece.
pub const START: usize = 0x755784;
/// IDA 0x7558C4, the end of the piece.
pub const END: usize = 0x7558C4;

/// Where IDA 0x755784 lives (`moved::MOVED`'s home).
#[cfg(test)]
pub fn names_addr() -> u32 {
    crate::pointers::game_address(&raw const crate::menu::owner::menus().play.names)
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
