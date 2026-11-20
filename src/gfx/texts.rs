//! The picture and font file names and the one message of the graphics code, as Rust data. Nothing
//! writes them; `build_glyphs` makes a numbered path of a template in a buffer of its own.

/// The text files\g\graphics\sprites\mbuts00.pcx (a NUL ends it).
pub(super) static MBUTS: [u8; 37] = *b"files\\g\\graphics\\sprites\\mbuts00.pcx\0";

/// The text files\d\gfx\jade.bun (a NUL ends it).
pub static JADE: [u8; 21] = *b"files\\d\\gfx\\jade.bun\0";

/// The text files\d\fnt\fnt22_00.pcx (a NUL ends it).
pub(super) static FNT22: [u8; 25] = *b"files\\d\\fnt\\fnt22_00.pcx\0";

/// The text files\d\gfx\chatzone.pcx (a NUL ends it).
pub static CHATZONE: [u8; 25] = *b"files\\d\\gfx\\chatzone.pcx\0";

/// The text files\d\fnt\pal_00.pcx (a NUL ends it).
pub(super) static PAL_00: [u8; 23] = *b"files\\d\\fnt\\pal_00.pcx\0";

/// The text files\d\fnt\fnt13_00.pcx (a NUL ends it).
pub(super) static FNT13: [u8; 25] = *b"files\\d\\fnt\\fnt13_00.pcx\0";

/// The text files\d\fnt\fnt21_00.pcx (a NUL ends it).
pub(super) static FNT21: [u8; 25] = *b"files\\d\\fnt\\fnt21_00.pcx\0";

/// The text files\d\fnt\fnt23_00.pcx (a NUL ends it).
pub(super) static FNT23: [u8; 25] = *b"files\\d\\fnt\\fnt23_00.pcx\0";

/// The text files\d\fnt\fnt27_00.pcx (a NUL ends it).
pub(super) static FNT27: [u8; 25] = *b"files\\d\\fnt\\fnt27_00.pcx\0";
