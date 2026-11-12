//! The picture and font file names and the one message of the graphics code: the texts RAL.EXE's
//! read-only data held, as Rust data. Nothing writes them; `build_glyphs` makes a numbered path of a
//! template in a buffer of its own. `tests::texts_are_the_games` compares each to the image while
//! it exists.

/// IDA 0x732C93: the text files\g\graphics\sprites\mbuts00.pcx (a NUL ends it).
pub static MBUTS: [u8; 37] = *b"files\\g\\graphics\\sprites\\mbuts00.pcx\0";

/// IDA 0x5C2CF7: the text Finding 3d hardware (a NUL ends it).
pub static FINDING_3D: [u8; 20] = *b"Finding 3d hardware\0";

/// IDA 0x4F66C8: the text files\d\gfx\jade.bun (a NUL ends it).
pub static JADE: [u8; 21] = *b"files\\d\\gfx\\jade.bun\0";

/// IDA 0x5BE8D8: the text files\d\fnt\fnt22_00.pcx (a NUL ends it).
pub static FNT22: [u8; 25] = *b"files\\d\\fnt\\fnt22_00.pcx\0";

/// IDA 0x5AB400: the text files\d\gfx\chatzone.pcx (a NUL ends it).
pub static CHATZONE: [u8; 25] = *b"files\\d\\gfx\\chatzone.pcx\0";

/// IDA 0x5297DE: the text files\d\fnt\pal_00.pcx (a NUL ends it).
pub static PAL_00: [u8; 23] = *b"files\\d\\fnt\\pal_00.pcx\0";

/// IDA 0x52977A: the text files\d\fnt\fnt13_00.pcx (a NUL ends it).
pub static FNT13: [u8; 25] = *b"files\\d\\fnt\\fnt13_00.pcx\0";

/// IDA 0x529793: the text files\d\fnt\fnt21_00.pcx (a NUL ends it).
pub static FNT21: [u8; 25] = *b"files\\d\\fnt\\fnt21_00.pcx\0";

/// IDA 0x5297AC: the text files\d\fnt\fnt23_00.pcx (a NUL ends it).
pub static FNT23: [u8; 25] = *b"files\\d\\fnt\\fnt23_00.pcx\0";

/// IDA 0x5297C5: the text files\d\fnt\fnt27_00.pcx (a NUL ends it).
pub static FNT27: [u8; 25] = *b"files\\d\\fnt\\fnt27_00.pcx\0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_are_the_games() {
        for (key, text) in [
            ("gfx.texts.MBUTS", &MBUTS[..]),
            ("gfx.texts.FINDING_3D", &FINDING_3D[..]),
            ("gfx.texts.JADE", &JADE[..]),
            ("gfx.texts.FNT22", &FNT22[..]),
            ("gfx.texts.CHATZONE", &CHATZONE[..]),
            ("gfx.texts.PAL_00", &PAL_00[..]),
            ("gfx.texts.FNT13", &FNT13[..]),
            ("gfx.texts.FNT21", &FNT21[..]),
            ("gfx.texts.FNT23", &FNT23[..]),
            ("gfx.texts.FNT27", &FNT27[..]),
        ] {
            assert_eq!(crate::test_support::first_values::bytes(key), text, "{key}");
        }
    }
}
