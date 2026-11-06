//! Module gfx's owner (step 11, owned state): the module's state in one struct, handed by
//! reference to the code that runs it in place of global stores. The PNG reader's state
//! comes first; the other groups join it one by one.
//!
//! `gfx()` is the boundary: the callers outside the module that do not take the owner yet
//! fetch it there and pass it down (`notes/memory_layer_plan.md`, step 11).

use crate::gfx::build_glyphs::SheetVars;
use crate::gfx::cards::Cards;
use crate::gfx::choices_store::ScreenChoices;
use crate::gfx::converters::Converters;
use crate::gfx::formats::Formats;
use crate::gfx::locked::Locked;
use crate::gfx::modes::Modes;
use crate::gfx::png_file::PngFile;
use crate::gfx::png_header::PngHeader;
use crate::gfx::reports::Reports;
use crate::gfx::rescale_picture::Rescaler;
use crate::gfx::screen_state::ScreenMode;
use crate::gfx::shots::Shots;
use crate::gfx::strips::Strips;
use crate::gfx::texture_pixels::Grading;
use crate::session::Dword;

/// The PNG reader's state: the file being decoded, its header and the setting that turns
/// PNG files off.
pub struct Png {
    pub file: PngFile,
    pub header: PngHeader,
    /// `g_pngSetting` (`dword_51B97C`, `nopngdecode`, a `var\ral.cfg` setting): 0 = PNG
    /// files are read.
    pub setting: Dword,
}

impl Png {
    pub const fn new() -> Self {
        Png {
            file: PngFile::new(),
            header: PngHeader::new(),
            setting: Dword::new(0),
        }
    }
}

/// The screen modes: the 3D cards found, the mode table, the chosen modes and the
/// graphics settings' choices.
pub struct Display {
    pub cards: Cards,
    pub modes: Modes,
    pub mode: ScreenMode,
    pub choices: ScreenChoices,
}

impl Display {
    pub const fn new() -> Self {
        Display {
            cards: Cards::new(),
            modes: Modes::new(),
            mode: ScreenMode::new(),
            choices: ScreenChoices::new(),
        }
    }
}

/// The pixel formats and the converters between them.
pub struct Pixel {
    pub formats: Formats,
    pub converters: Converters,
}

impl Pixel {
    pub const fn new() -> Self {
        Pixel {
            formats: Formats::new(),
            converters: Converters::new(),
        }
    }
}

/// Module gfx's state.
pub struct Gfx {
    pub display: Display,
    pub pixel: Pixel,
    /// The glyph sheet's table variables (`build_glyphs`).
    pub sheet: SheetVars,
    /// The screenshot switch and name.
    pub shots: Shots,
    /// `g_surfaceLocked`.
    pub locked: Locked,
    /// The switch that makes Direct3D's errors fatal.
    pub reports: Reports,
    /// The view strips' sizes and places.
    pub strips: Strips,
    /// The texture grading's variables.
    pub grading: Grading,
    pub png: Png,
    /// The picture rescaler's variables.
    pub rescale: Rescaler,
}

/// The one instance. Reached through `gfx()`; `moved` registers its stores' places.
pub static GFX: Gfx = Gfx {
    display: Display::new(),
    pixel: Pixel::new(),
    sheet: SheetVars::new(),
    shots: Shots::new(),
    locked: Locked::new(),
    reports: Reports::new(),
    strips: Strips::new(),
    grading: Grading::new(),
    png: Png::new(),
    rescale: Rescaler::new(),
};

/// The module's state, for a caller that does not take it as a parameter yet.
pub fn gfx() -> &'static Gfx {
    &GFX
}
