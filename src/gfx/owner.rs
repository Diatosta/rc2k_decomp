//! Module gfx's owner (step 11, owned state): the module's state in one struct, handed by
//! reference to the code that runs it in place of global stores. The PNG reader's state
//! comes first; the other groups join it one by one.
//!
//! `gfx()` is the boundary: the callers outside the module that do not take the owner yet
//! fetch it there and pass it down (`notes/memory_layer_plan.md`, step 11).

use crate::gfx::bar_store::BarVars;
use crate::gfx::blit::Blitter;
use crate::gfx::build_game_fonts::GameFontsMemory;
use crate::gfx::build_glyphs::SheetVars;
use crate::gfx::cards::Cards;
use crate::gfx::choices_store::ScreenChoices;
use crate::gfx::converters::Converters;
use crate::gfx::decoder::Decoder;
use crate::gfx::fill_row_table::ROW_TABLE_ROOM;
use crate::gfx::fill_thirds_table::TABLE_LEN;
use crate::gfx::fonts_store::FontBlock;
use crate::gfx::formats::Formats;
use crate::gfx::front::Surfaces;
use crate::gfx::glyph_tables::GlyphTables;
use crate::gfx::jade_store::JadeFont;
use crate::gfx::locked::Locked;
use crate::gfx::modes::Modes;
use crate::gfx::pcx::Pcx;
use crate::gfx::pcx_planes::LEN as PLANES_LEN;
use crate::gfx::picture_store::Picture;
use crate::gfx::png_file::PngFile;
use crate::gfx::png_header::PngHeader;
use crate::gfx::reports::Reports;
use crate::gfx::rescale_picture::Rescaler;
use crate::gfx::screen_state::{DrawSurface, ScreenMode};
use crate::gfx::shots::Shots;
use crate::gfx::strips::Strips;
use crate::gfx::texture_pixels::Grading;
use crate::gfx::texture_upload::Textures;
use crate::gfx::typefaces::FontObjects;
use crate::gfx::upload::Upload;
use crate::gfx::windscreen_drop_batch::WindscreenDrops;
use crate::session::{Bytes, Dword};

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

/// The picture loader and the decoders: the picture being loaded, the decoded picture,
/// the PCX file, the row of planes a screenshot writes and the showing switch.
pub struct Pictures {
    pub picture: Picture,
    pub decoder: Decoder,
    pub pcx: Pcx,
    /// The planes of the row being written (a buffer, zero at the start).
    pub planes: Bytes<PLANES_LEN>,
    /// `g_showingPicture` (`dword_51B9C4`): 1 while a picture is being shown.
    pub showing: Dword,
}

impl Pictures {
    pub const fn new() -> Self {
        Pictures {
            picture: Picture::new(),
            decoder: Decoder::new(),
            pcx: Pcx::new(),
            planes: Bytes::new([0; PLANES_LEN]),
            showing: Dword::new(0),
        }
    }
}

/// The fonts: the menu fonts, the font objects, the jade font, the game fonts' memory and
/// the glyph tables.
pub struct FontStores {
    pub font_block: FontBlock,
    pub typefaces: FontObjects,
    pub jade: JadeFont,
    pub game_fonts_memory: GameFontsMemory,
    pub glyph_tables: GlyphTables,
}

impl FontStores {
    pub const fn new() -> Self {
        FontStores {
            font_block: FontBlock::new(),
            typefaces: FontObjects::new(),
            jade: JadeFont::new(),
            game_fonts_memory: GameFontsMemory::new(),
            glyph_tables: GlyphTables::new(),
        }
    }
}

/// The drawing target: the blitters' variables, the loading bar, the draw surface, the
/// surface handles and the row and thirds tables.
pub struct Drawing {
    pub blit: Blitter,
    pub bar: BarVars,
    pub draw_surface: DrawSurface,
    pub surfaces: Surfaces,
    /// `dword_4F3FC4`: each row of a picture (602 of them), zero at the start: its offset in
    /// the decoded buffer since phase 12 (the original's addresses).
    pub row_table: [Dword; ROW_TABLE_ROOM as usize],
    /// `byte_4F4B30`: n / 3 for the sums of three components (zero until filled).
    pub thirds: Bytes<TABLE_LEN>,
}

impl Drawing {
    pub const fn new() -> Self {
        Drawing {
            blit: Blitter::new(),
            bar: BarVars::new(),
            draw_surface: DrawSurface::new(),
            surfaces: Surfaces::new(),
            row_table: [const { Dword::new(0) }; ROW_TABLE_ROOM as usize],
            thirds: Bytes::new([0; TABLE_LEN]),
        }
    }
}

/// The 3D card's textures: the upload's variables, the card's tables and the windscreen
/// drops' batch.
pub struct Card3d {
    pub upload: Upload,
    pub texture_tables: Textures,
    pub windscreen_drop_batch: WindscreenDrops,
}

impl Card3d {
    pub const fn new() -> Self {
        Card3d {
            upload: Upload::new(),
            texture_tables: Textures::new(),
            windscreen_drop_batch: WindscreenDrops::new(),
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
    pub pictures: Pictures,
    pub fonts: FontStores,
    pub drawing: Drawing,
    pub card: Card3d,
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
    pictures: Pictures::new(),
    fonts: FontStores::new(),
    drawing: Drawing::new(),
    card: Card3d::new(),
};

/// The module's state, for a caller that does not take it as a parameter yet.
pub fn gfx() -> &'static Gfx {
    &GFX
}
