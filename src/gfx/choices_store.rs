//! The graphics settings page's choices (the screen mode chosen and the modes to choose from, and
//! the switches the page's items copy), out of RAL.EXE's image (phase 5), typed (phase 7).
//! Placed in a file of its own beside the graphics module's stores (converted in parallel). The
//! options' item tables (`menu::option_items`) name these variables by IDA address, through
//! `moved::MOVED`. `#[repr(C)]` struct of atomic cells with the image's layout, first values the
//! image's (`tests`).

use crate::session::{Bytes, Dword};

/// The graphics page's choices (IDA 0x4F50C0 .. 0x4F51F0, the image's layout).
#[repr(C, align(16))]
pub struct ScreenChoices {
    /// 0x4F50C0: Not named: 4 bytes (a gap, or data nothing reads here).
    pub _gap_4f50c0: Bytes<4>,
    /// 0x4F50C4: `dword_4F50C4`: the switch `menu::toggle_75564c` flips.
    pub switch_modes: Dword,
    /// 0x4F50C8: `dword_4F50C8`: the screen mode chosen.
    pub mode: Dword,
    /// 0x4F50CC: `dword_4F50CC`: the copy of `dword_755644`.
    pub switch_hud: Dword,
    /// 0x4F50D0: `dword_4F50D0`: the copy of `dword_755640`.
    pub switch_copy: Dword,
    /// 0x4F50D4: `dword_4F50D4`: read by the page's item 2 (`screens::draw_graphics_settings`).
    pub page_a: Dword,
    /// 0x4F50D8: `dword_4F50D8`: the same.
    pub page_b: Dword,
    /// 0x4F50DC: Not named: 8 bytes (a gap, or data nothing reads here).
    pub _gap_4f50dc: Bytes<8>,
    /// 0x4F50E4: `dword_4F50E4`: how many modes there are to choose from.
    pub modes: Dword,
    /// 0x4F50E8: `unk_4F50E8`: the modes to choose from, 0x10 bytes each (three dwords used).
    pub list: Bytes<264>,
}

pub static SCREEN_CHOICES: ScreenChoices = ScreenChoices {
    _gap_4f50c0: Bytes::new([0; 4]),
    switch_modes: Dword::new(0x0),
    mode: Dword::new(0x0),
    switch_hud: Dword::new(0x0),
    switch_copy: Dword::new(0x0),
    page_a: Dword::new(0x0),
    page_b: Dword::new(0x0),
    _gap_4f50dc: Bytes::new([0; 8]),
    modes: Dword::new(0x0),
    list: Bytes::new([0; 264]),
};

impl ScreenChoices {
    pub const START: usize = 0x4F50C0;
    pub const LEN: usize = 0x130;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        &SCREEN_CHOICES as *const ScreenChoices as usize
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
            std::slice::from_raw_parts(ScreenChoices::home() as *const u8, ScreenChoices::LEN)
        };
        assert_eq!(mine, &image[..]);
    }
}
