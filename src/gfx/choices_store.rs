//! The graphics settings page's choices (the screen mode chosen and the modes to choose from, and
//! the switches the page's items copy), out of RAL.EXE's image.
//! Placed in a file of its own beside the graphics module's stores. `#[repr(C)]` struct of
//! atomic cells with the image's layout, first values the image's (`tests`).

use crate::errors::BadInput;
use crate::session::{PlainBytes, PlainDword, Run};

/// The graphics page's choices (IDA 0x4F50C0 .. 0x4F51F0, the image's layout; the rows' texts
/// in the language file: `tokAVPPrimary` / `tokAVPSecondary`, `tokAVPDoubleBuffer` /
/// `tokAVPTripleBuffer`, `tokOff` / `tokOn`).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct ScreenChoices {
    /// 0x4F50C0: `unk_4F50C0`: 1 once the choices are made (`gfx::screen_mode_choices`).
    pub made: PlainDword,
    /// 0x4F50C4: `dword_4F50C4`: the video card chosen, 0 primary, 1 secondary (the page's
    /// row 0; `OptionSwitch::ScreenModeCard` flips it).
    pub card: PlainDword,
    /// 0x4F50C8: `dword_4F50C8`: the resolution chosen, its place in `list` (row 1).
    pub resolution: PlainDword,
    /// 0x4F50CC: `dword_4F50CC`: the buffering, 0 double, 1 triple (row 2; the copy of
    /// `OptionWork.switch_1`).
    pub buffering: PlainDword,
    /// 0x4F50D0: `dword_4F50D0`: mipmapping, 0 off, 1 on (row 3; the copy of
    /// `OptionWork.switch_0`; into `RACE_VARS.mip_setting`), held to `mip_limit`.
    pub mipmapping: PlainDword,
    /// 0x4F50D4: `dword_4F50D4`: 1 when there is a primary 3D card (`gfx::cards`).
    pub has_primary: PlainDword,
    /// 0x4F50D8: `dword_4F50D8`: 1 when there is a secondary one; row 0 is choosable only
    /// with both (`screens::draw_graphics_settings`).
    pub has_secondary: PlainDword,
    /// 0x4F50DC: 1 once `list` is made.
    pub listed: PlainDword,
    /// 0x4F50E0: the mipmapping's limit when the list was made (`CARDS_3D.mips`).
    pub mip_limit: PlainDword,
    /// 0x4F50E4: `dword_4F50E4`: how many resolutions there are in `list`.
    pub resolutions: PlainDword,
    /// 0x4F50E8: `ScreenChoices.list.modes[0].width`: the resolutions, 0x10 bytes each (width, height, the pixel
    /// kind's figure, the mode); room for 16 and 8 bytes more.
    pub list: ModeList,
}

/// One resolution of the list (`ScreenChoices.list.modes[0].width`, 0x10 bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ScreenMode {
    /// +0.
    pub width: PlainDword,
    /// +4.
    pub height: PlainDword,
    /// +8: the pixel kind's figure.
    pub figure: PlainDword,
    /// +0xC: the mode.
    pub mode: PlainDword,
}

/// The resolutions' list: room for 16 and 8 bytes more.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ModeList {
    pub modes: [ScreenMode; 16],
    pub spare: PlainBytes<8>,
}

/// A resolution's four dwords as a run.
impl Run for ScreenMode {
    fn count(&self) -> usize {
        4
    }
    fn bits(&self, i: usize) -> u32 {
        [&self.width, &self.height, &self.figure, &self.mode][i].get()
    }
    fn set_bits(&self, i: usize, v: u32) {
        [&self.width, &self.height, &self.figure, &self.mode][i].set(v);
    }
}

impl ModeList {
    /// PlainDword `index` (0 width, 1 height, 2 figure, 3 mode) of the resolution at `place`. A
    /// place past the 16 reads on into the 8 bytes after the list, as the original does, and
    /// past those is refused (`what` names it in the log).
    pub fn dword(&self, place: u32, index: usize, what: &'static str) -> Result<u32, BadInput> {
        let n = 4 * place as usize + index;
        crate::session::read_runs(&[&self.modes, &self.spare], n, what)
    }
}

impl ScreenChoices {
    pub const fn new() -> Self {
        ScreenChoices {
            made: PlainDword::new(0x0),
            card: PlainDword::new(0x0),
            resolution: PlainDword::new(0x0),
            buffering: PlainDword::new(0x0),
            mipmapping: PlainDword::new(0x0),
            has_primary: PlainDword::new(0x0),
            has_secondary: PlainDword::new(0x0),
            listed: PlainDword::new(0x0),
            mip_limit: PlainDword::new(0x0),
            resolutions: PlainDword::new(0x0),
            list: ModeList {
                modes: [const {
                    ScreenMode {
                        width: PlainDword::new(0),
                        height: PlainDword::new(0),
                        figure: PlainDword::new(0),
                        mode: PlainDword::new(0),
                    }
                }; 16],
                spare: PlainBytes::new([0; 8]),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The old byte read of the list, kept to check the read by field.
    fn old_dword(list: &ModeList, place: u32, index: usize) -> Result<u32, BadInput> {
        crate::cells::load_value(list, (16 * place + 4 * index as u32) as usize)
    }

    /// Every place and index the byte read allowed (to the end of the list) reads the same,
    /// and the first past it is refused by both.
    #[test]
    fn dword_reads_as_the_byte_read() {
        let list = ScreenChoices::new().list;
        for (k, m) in list.modes.iter().enumerate() {
            for (i, v) in [&m.width, &m.height, &m.figure, &m.mode]
                .into_iter()
                .enumerate()
            {
                v.set(0x100 * k as u32 + i as u32 + 1);
            }
        }
        list.spare.set_u32(0, 0xAAAA_0001);
        list.spare.set_u32(4, 0xAAAA_0002);
        for place in 0..20 {
            for index in 0..4 {
                assert_eq!(
                    list.dword(place, index, "t").ok(),
                    old_dword(&list, place, index).ok(),
                    "place {place} index {index}"
                );
            }
        }
        assert!(list.dword(16, 2, "t").is_err());
    }
}
