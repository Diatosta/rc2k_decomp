//! The players' names and the text being typed: the four players' names (0x20 bytes each) and
//! what follows them, the name `menu::standings` keeps, and the text as it was before an entry
//! started. One static.

use crate::memory::session::PlainBytes;

/// The names and the text backup.
#[derive(rc2k_derive::Cells)]
pub struct Names {
    /// The four players' names, 0x20 bytes each (`menu::player_names`).
    pub player_names: PlainBytes<128>,
    /// `Names.player_names`: The rest of the names' area.
    pub more_names: PlainBytes<128>,
    /// The name kept by `menu::standings` (0x1F bytes and a NUL).
    pub kept_name: PlainBytes<32>,
    /// The text as it was before the entry started (`menu::text_entry`).
    pub text_backup: PlainBytes<32>,
}

/// The whole area as one run of bytes: the four names, the rest of the names, the kept name and
/// the text backup (a name that lacks its end runs on into what follows it).
impl crate::memory::place::Run for Names {
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
