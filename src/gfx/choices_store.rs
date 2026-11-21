//! The graphics settings page's choices: the screen mode chosen, the modes to choose from, and the
//! switches the page's items copy.

use crate::errors::BadInput;
use crate::memory::session::{PlainBytes, Run};
use std::cell::Cell;

/// The graphics page's choices (the rows' texts are in the language file: `tokAVPPrimary` /
/// `tokAVPSecondary`, `tokAVPDoubleBuffer` / `tokAVPTripleBuffer`, `tokOff` / `tokOn`).
pub struct ScreenChoices {
    /// 1 once the choices are made (`gfx::screen_mode_choices`).
    pub made: Cell<u32>,
    /// The video card chosen, 0 primary, 1 secondary (the page's row 0;
    /// `OptionSwitch::ScreenModeCard` flips it).
    pub card: Cell<u32>,
    /// The resolution chosen, its place in `list` (row 1).
    pub resolution: Cell<u32>,
    /// The buffering, 0 double, 1 triple (row 2; the copy of `MenuOptions.buffering`).
    pub buffering: Cell<u32>,
    /// Mipmapping, 0 off, 1 on (row 3; the copy of `MenuOptions.mipmapping`; into
    /// `RACE_VARS.mip_setting`), held to `mip_limit`.
    pub mipmapping: Cell<u32>,
    /// 1 when there is a primary 3D card .
    pub has_primary: Cell<u32>,
    /// 1 when there is a secondary one; row 0 is choosable only with both
    /// (`screens::draw_graphics_settings`).
    pub has_secondary: Cell<u32>,
    /// 1 once `list` is made.
    pub listed: Cell<u32>,
    /// The mipmapping's limit when the list was made (`CARDS_3D.mips`).
    pub mip_limit: Cell<u32>,
    /// How many resolutions there are in `list`.
    pub resolutions: Cell<u32>,
    /// The resolutions, 0x10 bytes each (width, height, the pixel kind's figure, the mode); room
    /// for 64 and 8 bytes more.
    pub list: ModeList,
}

/// One resolution of the list (`ScreenChoices.list.modes[0].width`, 0x10 bytes).
pub struct ScreenMode {
    /// +0.
    pub width: Cell<u32>,
    /// +4.
    pub height: Cell<u32>,
    /// The pixel kind's figure.
    pub figure: Cell<u32>,
    /// The mode.
    pub mode: Cell<u32>,
}

/// The resolutions' list: room for 64 and 8 bytes more.
pub struct ModeList {
    pub modes: [ScreenMode; 64],
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
    /// Cell<u32> `index` (0 width, 1 height, 2 figure, 3 mode) of the resolution at `place`. A
    /// place past the 64 reads on into the 8 bytes after the list, and past those is refused
    /// (`what` names it in the log).
    pub fn dword(&self, place: u32, index: usize, what: &'static str) -> Result<u32, BadInput> {
        let n = 4 * place as usize + index;
        crate::memory::session::read_runs(&[&self.modes, &self.spare], n, what)
    }
}

impl ScreenChoices {
    pub const fn new() -> Self {
        ScreenChoices {
            made: Cell::new(0x0),
            card: Cell::new(0x0),
            resolution: Cell::new(0x0),
            buffering: Cell::new(0x0),
            mipmapping: Cell::new(0x0),
            has_primary: Cell::new(0x0),
            has_secondary: Cell::new(0x0),
            listed: Cell::new(0x0),
            mip_limit: Cell::new(0x0),
            resolutions: Cell::new(0x0),
            list: ModeList {
                modes: [const {
                    ScreenMode {
                        width: Cell::new(0),
                        height: Cell::new(0),
                        figure: Cell::new(0),
                        mode: Cell::new(0),
                    }
                }; 64],
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
        for place in 0..64 {
            for index in 0..4 {
                let want = 0x100 * place + index + 1;
                assert_eq!(list.dword(place, index as usize, "t").ok(), Some(want));
            }
        }
        assert_eq!(list.dword(64, 0, "t").ok(), Some(0xAAAA_0001));
        assert_eq!(list.dword(64, 1, "t").ok(), Some(0xAAAA_0002));
        assert!(list.dword(64, 2, "t").is_err());
    }
}
