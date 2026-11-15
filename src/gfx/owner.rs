//! Module gfx's owner: the module's state in one struct, handed by reference to the code that runs
//! it in place of global stores.

use crate::d3d::owner::D3d;
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
use crate::gfx::glyph_store::GlyphStore;
use crate::gfx::glyph_tables::GlyphTables;
use crate::gfx::jade_store::JadeFont;
use crate::gfx::locked::Locked;
use crate::gfx::modes::Modes;
use crate::gfx::pcx::Pcx;
use crate::gfx::pcx_planes::LEN as PLANES_LEN;
use crate::gfx::picture::PictureBuffers;
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
use crate::memory::session::{PlainBytes, PlainDword};

/// The PNG reader's state: the file being decoded, its header and the setting that turns PNG files
/// off.
pub struct Png {
    pub file: PngFile,
    pub header: PngHeader,
    /// The `nopngdecode` setting (`var\ral.cfg`): 0 = PNG files are read.
    pub setting: PlainDword,
}

impl Png {
    pub const fn new() -> Self {
        Png {
            file: PngFile::new(),
            header: PngHeader::new(),
            setting: PlainDword::new(0),
        }
    }
}

/// The screen modes: the 3D cards found, the mode table, the chosen modes and the graphics
/// settings' choices.
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

/// The picture loader and the decoders: the picture being loaded, the decoded picture, the PCX
/// file, the row of planes a screenshot writes and the showing switch.
pub struct Pictures {
    pub picture: Picture,
    pub decoder: Decoder,
    pub pcx: Pcx,
    /// The planes of the row being written (a buffer, zero at the start).
    pub planes: PlainBytes<PLANES_LEN>,
    /// 1 while a picture is being shown.
    pub showing: PlainDword,
    /// The picture buffers (`gfx::picture`).
    pub buffers: PictureBuffers,
}

impl Pictures {
    pub const fn new() -> Self {
        Pictures {
            picture: Picture::new(),
            decoder: Decoder::new(),
            pcx: Pcx::new(),
            planes: PlainBytes::new([0; PLANES_LEN]),
            showing: PlainDword::new(0),
            buffers: PictureBuffers::new(),
        }
    }
}

/// The fonts: the menu fonts, the font objects, the jade font, the game fonts' memory and the glyph
/// tables.
pub struct FontStores {
    pub font_block: FontBlock,
    pub typefaces: FontObjects,
    pub jade: JadeFont,
    pub game_fonts_memory: GameFontsMemory,
    pub glyph_tables: GlyphTables,
    /// The palettes and glyph tables the fonts and sprites draw with.
    pub glyph_store: GlyphStore,
    /// The blended font's palette (`BLENDED_PLACE`), 1024 bytes of which 15 entries are made; the
    /// installer's bar draws with it too.
    pub blended_palette: [PlainDword; 256],
    /// The kerning row table the text drawer keeps between glyphs: 0 at the start.
    pub kerning: std::sync::Mutex<crate::text::glyph_kerning::State>,
}

impl FontStores {
    pub const fn new() -> Self {
        FontStores {
            font_block: FontBlock::new(),
            typefaces: FontObjects::new(),
            jade: JadeFont::new(),
            game_fonts_memory: GameFontsMemory::new(),
            glyph_tables: GlyphTables::new(),
            glyph_store: GlyphStore::new(),
            blended_palette: [const { PlainDword::new(0) }; 256],
            kerning: std::sync::Mutex::new([0; rc2k_core::strings::text::KERNING_ROWS * 2]),
        }
    }
}

/// The drawing target: the blitters' variables, the loading bar, the draw surface, the surface
/// handles and the row and thirds tables.
pub struct Drawing {
    pub blit: Blitter,
    pub bar: BarVars,
    /// The surface the screen is drawn on: atomic cells, which the drawing threads read through
    /// `platform::direct3d::Handles` while the game's thread waits.
    pub draw_surface: DrawSurface,
    pub surfaces: Surfaces,
    /// Each row of a picture (602 of them), zero at the start: its offset in the decoded buffer.
    pub row_table: [PlainDword; ROW_TABLE_ROOM as usize],
    /// `byte_4F4B30`: n / 3 for the sums of three components (zero until filled).
    pub thirds: PlainBytes<TABLE_LEN>,
    /// The picture buffer's number while it is the surface (`gfx::picture_as_surface`), else 0.
    pub on_picture: std::sync::atomic::AtomicU32,
}

impl Drawing {
    pub const fn new() -> Self {
        Drawing {
            blit: Blitter::new(),
            bar: BarVars::new(),
            draw_surface: DrawSurface::new(),
            surfaces: Surfaces::new(),
            row_table: [const { PlainDword::new(0) }; ROW_TABLE_ROOM as usize],
            thirds: PlainBytes::new([0; TABLE_LEN]),
            on_picture: std::sync::atomic::AtomicU32::new(0),
        }
    }
}

/// The 3D card's textures: the upload's variables, the card's tables and the windscreen drops'
/// batch.
pub struct Card3d {
    pub upload: Upload,
    /// The card's tables: atomic cells, which the drawing threads read through
    /// `platform::direct3d::Handles` while the game's thread waits.
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

pub struct Gfx {
    pub display: Display,
    pub pixel: Pixel,
    /// The glyph sheet's table variables (`build_glyphs`).
    pub sheet: SheetVars,
    /// The screenshot switch and name.
    pub shots: Shots,
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
    /// The 3D cards' settings, scene flag and modes.
    pub d3d: D3d,
}

impl Gfx {
    /// The state at start-up: every store in its first state.
    pub const fn new() -> Self {
        Self {
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
            d3d: D3d::new(),
        }
    }
}
