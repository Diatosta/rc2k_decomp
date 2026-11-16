//! The graphics settings page's choices: the screen mode chosen, the modes to choose from, and the
//! switches the page's items copy. Struct of atomic cells.

use crate::errors::BadInput;
use crate::memory::session::{PlainBytes, PlainDword, Run};

/// The graphics page's choices (the rows' texts are in the language file: `tokAVPPrimary` /
/// `tokAVPSecondary`, `tokAVPDoubleBuffer` / `tokAVPTripleBuffer`, `tokOff` / `tokOn`).
#[derive(rc2k_derive::Cells)]
pub struct ScreenChoices {
    /// 1 once the choices are made (`gfx::screen_mode_choices`).
    pub made: PlainDword,
    /// The video card chosen, 0 primary, 1 secondary (the page's row 0;
    /// `OptionSwitch::ScreenModeCard` flips it).
    pub card: PlainDword,
    /// The resolution chosen, its place in `list` (row 1).
    pub resolution: PlainDword,
    /// The buffering, 0 double, 1 triple (row 2; the copy of `MenuOptions.buffering`).
    pub buffering: PlainDword,
    /// Mipmapping, 0 off, 1 on (row 3; the copy of `MenuOptions.mipmapping`; into
    /// `RACE_VARS.mip_setting`), held to `mip_limit`.
    pub mipmapping: PlainDword,
    /// 1 when there is a primary 3D card .
    pub has_primary: PlainDword,
    /// 1 when there is a secondary one; row 0 is choosable only with both
    /// (`screens::draw_graphics_settings`).
    pub has_secondary: PlainDword,
    /// 1 once `list` is made.
    pub listed: PlainDword,
    /// The mipmapping's limit when the list was made (`CARDS_3D.mips`).
    pub mip_limit: PlainDword,
    /// How many resolutions there are in `list`.
    pub resolutions: PlainDword,
    /// The resolutions, 0x10 bytes each (width, height, the pixel kind's figure, the mode); room
    /// for 16 and 8 bytes more.
    pub list: ModeList,
}

/// One resolution of the list (`ScreenChoices.list.modes[0].width`, 0x10 bytes).
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
    /// place past the 16 reads on into the 8 bytes after the list, and past those is refused
    /// (`what` names it in the log).
    pub fn dword(&self, place: u32, index: usize, what: &'static str) -> Result<u32, BadInput> {
        let n = 4 * place as usize + index;
        crate::memory::session::read_runs(&[&self.modes, &self.spare], n, what)
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

    /// Every place and index to the end of the list reads the field it names, then the spare bytes;
    /// The first past it is refused.
    #[test]
    fn dwords_read_the_fields() {
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
        for place in 0..16 {
            for index in 0..4 {
                let want = 0x100 * place + index + 1;
                assert_eq!(list.dword(place, index as usize, "t").ok(), Some(want));
            }
        }
        assert_eq!(list.dword(16, 0, "t").ok(), Some(0xAAAA_0001));
        assert_eq!(list.dword(16, 1, "t").ok(), Some(0xAAAA_0002));
        assert!(list.dword(16, 2, "t").is_err());
    }
}
