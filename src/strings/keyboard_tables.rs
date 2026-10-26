//! The built-in NEC keyboard (`strings::load_language`), out of RAL.EXE's image (phase 5), typed
//! (phase 7): its file head and its layout with the tables the layout's pointers name. The
//! rest of the image's range (nothing reads it) is kept as it was. `#[repr(C)]` struct of byte
//! buffers with the image's layout, first values the image's (`tests`); `moved::MOVED` points
//! the image's addresses at it: the layout's pointers name its tables.

use crate::session::{Bytes, with_runs};

/// The built-in NEC keyboard's file head, layout and tables (IDA 0x4F0DC8 .. 0x4F3034, the image's layout).
#[repr(C, align(16))]
pub struct KeyboardTables {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<8>,
    /// 0x4F0DC8: Not named: 24 bytes (a gap, or data nothing reads here).
    pub _gap_4f0dc8: Bytes<24>,
    /// 0x4F0DE0: `aKeyb`: the built-in file's head ("KEYB", its kinds; 0x30 bytes copied into the keyboard file).
    pub nec_head: Bytes<48>,
    /// 0x4F0E10: `unk_4F0E10`: the layout (pointers to the tables from +4) and the tables after it.
    pub nec_layout: Bytes<8740>,
}

pub static KEYBOARD_TABLES: KeyboardTables = KeyboardTables {
    _lead: Bytes::new([0; 8]),
    _gap_4f0dc8: Bytes::new(with_runs::<24>(&[(0, b"Les Chants Magn\xe9tiques.")])),
    nec_head: Bytes::new(with_runs::<48>(&[(0, b"KEYB"), (8, b"\x0f\x00\x01"), (16, b"Japanese NEC PC-98")])),
    nec_layout: Bytes::new(with_runs::<8740>(&[(4, b"$\x0eO\x00\xa4\x0eO"), (16, b"$\x0fO\x00\x00\x001234567890-^\x00\x00qwertyuiop@[\x00\x00asdfghjkl;:\x00\x00]zxcvbnm,./\x00*\x00 "), (89, b"=,\x00\x00\x00-\x00\x00\x00+"), (106, b"\\"), (118, b"/"), (150, b"!\"#$%&'()\x00=`\x00\x00QWERTYUIOP~{\x00\x00ASDFGHJKL+*_\x00}ZXCVBNM<>?\x00*\x00 "), (217, b"=,789-456+1230.\x00\x00|\x00\x00,"), (246, b"/"), (278, b"\x05\x00\x06\x00\x07\x00\x08\x00\t\x00\n\x00\x0b\x00\x0c\x00\r\x00\x0e\x00\x0f\x00\x10\x00\x8b\x00\x12\x00\x13\x00\x14\x00\x15\x00\x16\x00\x17\x00\x18\x00\x19\x00\x1a\x00\x1b\x00\x1c\x00\x1d\x00\x8c\x00\x1e\x00 \x00\x8d\x00\"\x00#\x00$\x00%\x00&\x00'\x00(\x00)\x00*\x00+\x00\x83\x00y\x00\x8e\x00\x1f\x000\x001\x002\x003\x004\x005\x006\x007\x008\x009\x00:\x00;\x00\x91\x00=\x00>\x00?\x00@\x00A\x00B\x00C\x00D\x00E\x00F\x00G\x00H\x00\x8f\x00\x90\x00K\x00L\x00M\x00N\x00O\x00P\x00Q\x00R\x00S\x00T\x00U\x00V\x00W\x00X\x00\x00\x00Y\x00Z\x00["), (468, b"\\\x00]\x00^\x00_\x00\x00\x00a\x00b\x00c\x00d\x00e\x00f\x00g\x00h\x00i\x00j\x00\x00\x00k\x00l"), (536, b"\xff\xff\xff\xff")])),
};

impl KeyboardTables {
    pub const START: usize = 0x4F0DC8;
    pub const LEN: usize = 0x226C;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&KEYBOARD_TABLES as *const KeyboardTables) + 8) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboardtables_starts_as_the_games() {
        let size = size_of::<KeyboardTables>();
        assert!(
            8 + KeyboardTables::LEN <= size
                && size <= (8 + KeyboardTables::LEN).next_multiple_of(16)
        );
        let image = crate::moved::image_bytes(KeyboardTables::START, KeyboardTables::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(KeyboardTables::home()) as *const u8,
                KeyboardTables::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
