//! The built-in NEC keyboard (`strings::load_language`), out of RAL.EXE's image, typed
//!: its file head and its layout with the tables the layout's pointers name. The
//! rest of the image's range (nothing reads it) is kept as it was. Plain immutable
//! bytes in the image's order, first values the image's (`tests`); nothing names the image's
//! addresses any more (`input::layouts` reads the layout's tables by offset).

use crate::session::with_runs;
#[cfg(test)]
use crate::strings::owner::strings;

/// The built-in NEC keyboard's file head, layout and tables (IDA 0x4F0DC8 .. 0x4F3034, the image's layout).
pub struct KeyboardTables {
    /// 0x4F0DC8: Not named: 24 bytes (a gap, or data nothing reads here).
    pub _gap_4f0dc8: [u8; 24],
    /// 0x4F0DE0: `aKeyb`: the built-in file's head ("KEYB", its kinds; 0x30 bytes copied into the keyboard file).
    pub nec_head: [u8; 48],
    /// 0x4F0E10: `unk_4F0E10`: the layout (pointers to the tables from +4) and the tables after it.
    pub nec_layout: [u8; 8740],
}

impl KeyboardTables {
    pub const fn new() -> Self {
        KeyboardTables {
    _gap_4f0dc8: with_runs::<24>(&[(0, b"Les Chants Magn\xe9tiques.")]),
    nec_head: with_runs::<48>(&[(0, b"KEYB"), (8, b"\x0f\x00\x01"), (16, b"Japanese NEC PC-98")]),
    nec_layout: with_runs::<8740>(&[(4, b"$\x0eO\x00\xa4\x0eO"), (16, b"$\x0fO\x00\x00\x001234567890-^\x00\x00qwertyuiop@[\x00\x00asdfghjkl;:\x00\x00]zxcvbnm,./\x00*\x00 "), (89, b"=,\x00\x00\x00-\x00\x00\x00+"), (106, b"\\"), (118, b"/"), (150, b"!\"#$%&'()\x00=`\x00\x00QWERTYUIOP~{\x00\x00ASDFGHJKL+*_\x00}ZXCVBNM<>?\x00*\x00 "), (217, b"=,789-456+1230.\x00\x00|\x00\x00,"), (246, b"/"), (278, b"\x05\x00\x06\x00\x07\x00\x08\x00\t\x00\n\x00\x0b\x00\x0c\x00\r\x00\x0e\x00\x0f\x00\x10\x00\x8b\x00\x12\x00\x13\x00\x14\x00\x15\x00\x16\x00\x17\x00\x18\x00\x19\x00\x1a\x00\x1b\x00\x1c\x00\x1d\x00\x8c\x00\x1e\x00 \x00\x8d\x00\"\x00#\x00$\x00%\x00&\x00'\x00(\x00)\x00*\x00+\x00\x83\x00y\x00\x8e\x00\x1f\x000\x001\x002\x003\x004\x005\x006\x007\x008\x009\x00:\x00;\x00\x91\x00=\x00>\x00?\x00@\x00A\x00B\x00C\x00D\x00E\x00F\x00G\x00H\x00\x8f\x00\x90\x00K\x00L\x00M\x00N\x00O\x00P\x00Q\x00R\x00S\x00T\x00U\x00V\x00W\x00X\x00\x00\x00Y\x00Z\x00["), (468, b"\\\x00]\x00^\x00_\x00\x00\x00a\x00b\x00c\x00d\x00e\x00f\x00g\x00h\x00i\x00j\x00\x00\x00k\x00l"), (536, b"\xff\xff\xff\xff")]),
}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboardtables_starts_as_the_games() {
        let strings = strings();
        let mut mine = Vec::new();
        mine.extend_from_slice(&strings.language.keyboard._gap_4f0dc8);
        mine.extend_from_slice(&strings.language.keyboard.nec_head);
        mine.extend_from_slice(&strings.language.keyboard.nec_layout);
        crate::test_support::first_values::assert_matches(
            "strings.keyboard_tables.KEYBOARD_TABLES/8812",
            &mine,
        );
    }

    /// The NEC layout's pointers name the offsets `input::layouts` uses.
    #[test]
    fn the_nec_layouts_tables_are_where_the_store_says() {
        let word = |n| {
            crate::test_support::first_values::dword("strings.keyboard_tables.NEC_POINTERS/20", n)
        };
        // The block's address, from the first table's pointer.
        let block = word(1) as usize - 0x14;
        for (n, offset) in [(1, 0x14), (2, 0x94), (4, 0x114)] {
            assert_eq!(word(n) as usize, block + offset);
        }
        assert_eq!(word(3), 0);
    }
}
