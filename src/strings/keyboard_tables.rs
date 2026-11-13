//! The built-in NEC keyboard (`strings::load_language`): its file head and its layout with the
//! tables the layout's pointers name. Plain immutable bytes (`input::layouts` reads the layout's
//! tables by offset).

use crate::session::with_runs;

/// The built-in NEC keyboard's file head, layout and tables.
pub struct KeyboardTables {
    /// The built-in file's head ("KEYB", its kinds; 0x30 bytes copied into the keyboard file).
    pub nec_head: [u8; 48],
    /// The layout (pointers to the tables from +4) and the tables after it.
    pub nec_layout: [u8; 8740],
}

impl KeyboardTables {
    pub const fn new() -> Self {
        KeyboardTables {
    nec_head: with_runs::<48>(&[(0, b"KEYB"), (8, b"\x0f\x00\x01"), (16, b"Japanese NEC PC-98")]),
    nec_layout: with_runs::<8740>(&[(4, b"$\x0eO\x00\xa4\x0eO"), (16, b"$\x0fO\x00\x00\x001234567890-^\x00\x00qwertyuiop@[\x00\x00asdfghjkl;:\x00\x00]zxcvbnm,./\x00*\x00 "), (89, b"=,\x00\x00\x00-\x00\x00\x00+"), (106, b"\\"), (118, b"/"), (150, b"!\"#$%&'()\x00=`\x00\x00QWERTYUIOP~{\x00\x00ASDFGHJKL+*_\x00}ZXCVBNM<>?\x00*\x00 "), (217, b"=,789-456+1230.\x00\x00|\x00\x00,"), (246, b"/"), (278, b"\x05\x00\x06\x00\x07\x00\x08\x00\t\x00\n\x00\x0b\x00\x0c\x00\r\x00\x0e\x00\x0f\x00\x10\x00\x8b\x00\x12\x00\x13\x00\x14\x00\x15\x00\x16\x00\x17\x00\x18\x00\x19\x00\x1a\x00\x1b\x00\x1c\x00\x1d\x00\x8c\x00\x1e\x00 \x00\x8d\x00\"\x00#\x00$\x00%\x00&\x00'\x00(\x00)\x00*\x00+\x00\x83\x00y\x00\x8e\x00\x1f\x000\x001\x002\x003\x004\x005\x006\x007\x008\x009\x00:\x00;\x00\x91\x00=\x00>\x00?\x00@\x00A\x00B\x00C\x00D\x00E\x00F\x00G\x00H\x00\x8f\x00\x90\x00K\x00L\x00M\x00N\x00O\x00P\x00Q\x00R\x00S\x00T\x00U\x00V\x00W\x00X\x00\x00\x00Y\x00Z\x00["), (468, b"\\\x00]\x00^\x00_\x00\x00\x00a\x00b\x00c\x00d\x00e\x00f\x00g\x00h\x00i\x00j\x00\x00\x00k\x00l"), (536, b"\xff\xff\xff\xff")]),
}
    }
}
