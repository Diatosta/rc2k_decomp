//! `gfx::read_picture_file` (0x4119F4): reads a picture file, PNG or PCX, into the picture
//! buffer as 32-bit pixels.
//!
//! Takes: path, which may be written to (see `files::swap_missing_extension`)
//! A path ending in `.pcx` goes to `gfx::read_pcx`. Any other first has its ending swapped
//! if the file is missing, and then goes to `gfx::read_png` and, if that fails, to
//! `read_pcx` (which is how a `.png` path whose file is missing ends up read as PCX).
//! That is with the setting `setting` at 0, as it is in the owner's game. With another
//! value a path ending in `.png` is not read at all (the value 2: success, nothing done) or
//! replaced by `files\d\gfx\chatzone.pcx`, and other paths go to `read_pcx` alone.
//! 14 callers. Refuses a path of fewer than four characters (the original looks at the four
//! bytes before the text's end) and what the readers refuse. The name is a reading.

use crate::errors::Refused;
use crate::files::swap_missing_extension::swap_missing_extension;
use crate::gfx::read_pcx::read_pcx;
use crate::gfx::read_png::read_png;

// ---- game data ----

/// IDA's address of the PNG setting.
pub const PNG_SETTING_START: usize = 0x51B97C;

/// Returns whether a picture was read. Converted: `rc2k_core::gfx::loading::{ending,
/// reader}`.
pub fn read_picture_file(
    path: &dyn crate::session::TextPlace,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    read_picture_file_in(path, gfx, crate::system::owner::system())
}

/// `read_picture_file` for a caller that holds the system.
pub fn read_picture_file_in(
    path: &dyn crate::session::TextPlace,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<bool, Refused> {
    use rc2k_core::gfx::loading::{Reader, ending, reader};
    let png = &gfx.png;
    let text = path.text("a picture's path")?;
    if &ending(&text)? == b".pcx" {
        return Ok(read_pcx(&text, gfx)?);
    }
    let text = swap_missing_extension(path)?;
    match reader(ending(&text)?, png.setting.get()) {
        Reader::PngThenPcx => Ok(read_png(png, &text, gfx, system)? || read_pcx(&text, gfx)?),
        Reader::Pcx => Ok(read_pcx(&text, gfx)?),
        Reader::Nothing => Ok(true),
        Reader::Chatzone => {
            let chatzone = &crate::gfx::texts::CHATZONE;
            Ok(read_pcx(&chatzone[..chatzone.len() - 1], gfx)?)
        }
    }
}

#[cfg(test)]
mod block_tests {
    use super::*;

    #[test]
    fn first_value_is_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::gfx::owner::GFX.unchecked().png.setting,
            PNG_SETTING_START,
            PNG_SETTING_START + 4,
        );
        assert_eq!(
            crate::moved::place(PNG_SETTING_START),
            crate::gfx::owner::GFX.unchecked().png.setting.addr() as usize
        );
    }
}
