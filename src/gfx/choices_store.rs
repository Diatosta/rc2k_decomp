//! The graphics settings page's choices (the screen mode chosen and the modes to choose from, and
//! the switches the page's items copy), out of RAL.EXE's image (phase 5), typed (phase 7).
//! Placed in a file of its own beside the graphics module's stores (converted in parallel). The
//! options' item tables (`menu::option_items`) name these variables by IDA address, through
//! `moved::MOVED`. `#[repr(C)]` struct of atomic cells with the image's layout, first values the
//! image's (`tests`).

use crate::session::{Bytes, Dword};

/// The graphics page's choices (IDA 0x4F50C0 .. 0x4F51F0, the image's layout; the rows' texts
/// in the language file: `tokAVPPrimary` / `tokAVPSecondary`, `tokAVPDoubleBuffer` /
/// `tokAVPTripleBuffer`, `tokOff` / `tokOn`).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct ScreenChoices {
    /// 0x4F50C0: `unk_4F50C0`: 1 once the choices are made (`gfx::screen_mode_choices`).
    pub made: Dword,
    /// 0x4F50C4: `dword_4F50C4`: the video card chosen, 0 primary, 1 secondary (the page's
    /// row 0; `menu::toggle_75564c` flips it).
    pub card: Dword,
    /// 0x4F50C8: `dword_4F50C8`: the resolution chosen, its place in `list` (row 1).
    pub resolution: Dword,
    /// 0x4F50CC: `dword_4F50CC`: the buffering, 0 double, 1 triple (row 2; the copy of
    /// `dword_755644`).
    pub buffering: Dword,
    /// 0x4F50D0: `dword_4F50D0`: mipmapping, 0 off, 1 on (row 3; the copy of
    /// `dword_755640`; into `RACE_VARS.mip_setting`), held to `mip_limit`.
    pub mipmapping: Dword,
    /// 0x4F50D4: `dword_4F50D4`: 1 when there is a primary 3D card (`gfx::cards`).
    pub has_primary: Dword,
    /// 0x4F50D8: `dword_4F50D8`: 1 when there is a secondary one; row 0 is choosable only
    /// with both (`screens::draw_graphics_settings`).
    pub has_secondary: Dword,
    /// 0x4F50DC: 1 once `list` is made.
    pub listed: Dword,
    /// 0x4F50E0: the mipmapping's limit when the list was made (`CARDS_3D.mips`).
    pub mip_limit: Dword,
    /// 0x4F50E4: `dword_4F50E4`: how many resolutions there are in `list`.
    pub resolutions: Dword,
    /// 0x4F50E8: `unk_4F50E8`: the resolutions, 0x10 bytes each (width, height, the pixel
    /// kind's figure, the mode); room for 16 and 8 bytes more.
    pub list: Bytes<264>,
}

pub static SCREEN_CHOICES: ScreenChoices = ScreenChoices {
    made: Dword::new(0x0),
    card: Dword::new(0x0),
    resolution: Dword::new(0x0),
    buffering: Dword::new(0x0),
    mipmapping: Dword::new(0x0),
    has_primary: Dword::new(0x0),
    has_secondary: Dword::new(0x0),
    listed: Dword::new(0x0),
    mip_limit: Dword::new(0x0),
    resolutions: Dword::new(0x0),
    list: Bytes::new([0; 264]),
};

impl ScreenChoices {
    pub const START: usize = 0x4F50C0;
    pub const LEN: usize = 0x130;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    #[cfg(test)]
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const SCREEN_CHOICES) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screenchoices_starts_as_the_games() {
        let size = size_of::<ScreenChoices>();
        assert!(ScreenChoices::LEN <= size && size <= (ScreenChoices::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(ScreenChoices::START, ScreenChoices::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(ScreenChoices::home()).cast_const(),
                ScreenChoices::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
