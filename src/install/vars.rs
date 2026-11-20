//! The installer's variables: the flags and counters of its run, the folder and file lists, the
//! installed record, scripts, the language list, and the pages' tables.
//!
//! Structs of cells, with the old layout's fields in its order and its gaps. The pages' tables
//! hold handles (`install::page_links`). The pages' items are `rc2k_core::install::model`.

use crate::gfx::typefaces::{FontId, fonts};
use crate::install::page_links::{LinkCell, PageLink};
use crate::memory::session::{PlainByte, PlainBytes, PlainDword, with_runs};

/// The installer's flags, counters and lists.
#[derive(rc2k_derive::Cells)]
pub struct InstallerRun {
    /// Padding that keeps every field its alignment (the struct is 16-byte aligned).
    pub _lead: PlainBytes<8>,
    /// The installation is this version (`install::mark_main_page`).
    pub same_version: PlainDword,
    /// The shortcuts are added.
    pub add_items: PlainDword,
    /// The shortcuts are removed (`install::uninstall`).
    pub remove_items: PlainDword,
    /// 1 a bad folder, 2 no room.
    pub path_problem: PlainDword,
    /// 26 of (found, free KB), by drive letter.
    pub drives: [PlainDword; 52],
    /// The drives found.
    pub drive_count: PlainDword,
    /// The files copied so far.
    pub files_copied: PlainDword,
    /// The files to copy.
    pub files_to_copy: PlainDword,
    /// `install::install_files`' spare.
    pub copy_spare: PlainDword,
    /// A gap of 4 byte(s): nothing reads it.
    pub _gap_51baf4: PlainBytes<4>,
    /// KB needed.
    pub room_needed: PlainDword,
    /// The installer's memory list (a `MemoryList` of 16 bytes, then its work).
    pub buffers: crate::memory::ListCells,
    /// The 48 bytes after the list (no code reads or writes them).
    pub buffers_rest: PlainBytes<48>,
    /// The progress bar's glyph table.
    pub bar_glyphs: PlainDword,
    /// The folders' names (a 0x10000-byte buffer; 0 now:
    /// `install::owner::Install::folders`).
    pub dirs: PlainDword,
    /// The files' names (a 0x40000-byte buffer; 0 now:
    /// `install::owner::Install::files`).
    pub files: PlainDword,
    /// The folders listed.
    pub dir_count: PlainDword,
    /// The files listed.
    pub file_count: PlainDword,
    /// The next folder to search (its number).
    pub dir_scan: PlainDword,
    /// The folders' table's end (its bytes).
    pub dir_cursor: PlainDword,
    /// The files' table's end (its bytes).
    pub file_cursor: PlainDword,
}

impl InstallerRun {
    pub const fn new() -> Self {
        InstallerRun {
            _lead: PlainBytes::new([0; 8]),
            same_version: PlainDword::new(0x0),
            add_items: PlainDword::new(0x0),
            remove_items: PlainDword::new(0x0),
            path_problem: PlainDword::new(0x0),
            drives: [const { PlainDword::new(0x0) }; 52],
            drive_count: PlainDword::new(0x0),
            files_copied: PlainDword::new(0x0),
            files_to_copy: PlainDword::new(0x0),
            copy_spare: PlainDword::new(0x0),
            _gap_51baf4: PlainBytes::new([0; 4]),
            room_needed: PlainDword::new(0x0),
            buffers: crate::memory::ListCells::empty(),
            buffers_rest: PlainBytes::new([0; 48]),
            bar_glyphs: PlainDword::new(0x0),
            dirs: PlainDword::new(0x0),
            files: PlainDword::new(0x0),
            dir_count: PlainDword::new(0x0),
            file_count: PlainDword::new(0x0),
            dir_scan: PlainDword::new(0x0),
            dir_cursor: PlainDword::new(0x0),
            file_cursor: PlainDword::new(0x0),
        }
    }
}

/// The installed record, the scripts and file names, and the languages.
#[derive(rc2k_derive::Cells)]
pub struct InstallerNames {
    /// The installed status byte (`install::check_installed_version`).
    pub status: PlainByte,
    /// The folder installed from (0xFF bytes used).
    pub source: PlainBytes<259>,
    /// The size of the source folder's name.
    pub source_size: PlainDword,
    /// A gap of 176 byte(s): nothing reads it.
    pub _gap_51bc98: PlainBytes<176>,
    /// The count script (`install::install_files`).
    pub script_count: PlainBytes<200>,
    /// The uninstall script, 17 bytes, then the mark (`SCRIPT_MARK`), then the rest of the script
    /// to its end (5) after it: the save and settings folders. `install::run_script` reads on
    /// into the rest when the mark is 1; 5 in the mark ends the script there.
    pub script: PlainBytes<{ SCRIPT_MARK + 1 + 30 }>,
    /// "ral.exe" (in ".\ral.exe").
    pub program: PlainBytes<8>,
    /// "ral.exe".
    pub ral_exe: PlainBytes<8>,
    /// A gap of 4 byte(s): nothing reads it.
    pub _gap_51be50: PlainBytes<4>,
    /// "files\%readme\readme.txt".
    pub readme: PlainBytes<25>,
    /// "files\%readme\manual.pdf".
    pub manual: PlainBytes<25>,
    /// "files\d\gfx\install.bun".
    pub installer_data: PlainBytes<24>,
    /// "files\d\gfx\install.pal".
    pub install_pal: PlainBytes<24>,
    /// "files\d\gfx\backdrop.pcx".
    pub backdrop: PlainBytes<25>,
    /// The languages found.
    pub language_count: PlainDword,
    /// The languages found, 0x80 bytes each (eight).
    pub languages: PlainBytes<1024>,
    /// A gap of 128 byte(s): nothing reads it.
    pub _gap_51c2ee: PlainBytes<128>,
    /// "lang.cfg".
    pub lang_cfg: PlainBytes<10>,
    /// A gap of 127 byte(s): nothing reads it.
    pub _gap_51c378: PlainBytes<127>,
    /// KB needed by installation mode.
    pub room_by_mode: PlainBytes<16>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: PlainBytes<9>,
}

impl InstallerNames {
    pub const fn new() -> Self {
        InstallerNames {
    status: PlainByte::new(0x0),
    source: PlainBytes::new([0; 259]),
    source_size: PlainDword::new(0x0),
    _gap_51bc98: PlainBytes::new(with_runs::<176>(&[(20, b"\x18"), (44, b"PROGMAN\x00[CreateGroup(Rally Championship)]\x00[DeleteGroup(Rally Championship)]\x00[AddItem(\x00,Rally)]\x00,Uninstall Rally)]\x00,View Readme)]")])),
    script_count: PlainBytes::new(with_runs::<200>(&[(0, b"\x03ral.exe\x03*.txt\x03*.pdf\x08\x02\x01files\x04a\x04b\x04d\x08\x03\x04fmv\t\x01g\x04frnt\x04graphics\x08\x03\x04slides\t\x02\x04levels\x04%locale\x04m\x04maps\x08\x03\x04music\t\x04replay\x04s\x01%speech\x04m\x04f\x08\x03\x04cars\x04rally\t\x02\x02\t\x01savedata\x01carsetup\x02\x01replay\x02\x01savegame\x02\x01snapshot\x02\x02\x01var\x03ral.cfg\x02\x06\x05")])),
    script: PlainBytes::new(with_runs::<{ SCRIPT_MARK + 1 + 30 }>(&[(0, b"\x03*.*\x01files\x02\x04files"), (SCRIPT_MARK, b"\x01"), (SCRIPT_MARK + 1, b"savedata\x02\x04savedata\x01var\x02\x04var\x05.\\")])),
    program: PlainBytes::new(with_runs::<8>(&[(0, b"ral.exe")])),
    ral_exe: PlainBytes::new(with_runs::<8>(&[(0, b"ral.exe")])),
    _gap_51be50: PlainBytes::new(with_runs::<4>(&[(0, b" -u")])),
    readme: PlainBytes::new(with_runs::<25>(&[(0, b"files\\%readme\\readme.txt")])),
    manual: PlainBytes::new(with_runs::<25>(&[(0, b"files\\%readme\\manual.pdf")])),
    installer_data: PlainBytes::new(with_runs::<24>(&[(0, b"files\\d\\gfx\\install.bun")])),
    install_pal: PlainBytes::new(with_runs::<24>(&[(0, b"files\\d\\gfx\\install.pal")])),
    backdrop: PlainBytes::new(with_runs::<25>(&[(0, b"files\\d\\gfx\\backdrop.pcx")])),
    language_count: PlainDword::new(0x0),
    languages: PlainBytes::new([0; 1024]),
    _gap_51c2ee: PlainBytes::new([0; 128]),
    lang_cfg: PlainBytes::new(with_runs::<10>(&[(0, b"lang.cfg")])),
    _gap_51c378: PlainBytes::new(with_runs::<127>(&[(0, b"lang1\x00\xee\xbeQ\x00\x7f\x00\x00\x00lang2\x00n\xbfQ\x00\x7f\x00\x00\x00lang3\x00\xee\xbfQ\x00\x7f\x00\x00\x00lang4\x00n\xc0Q\x00\x7f\x00\x00\x00lang5\x00\xee\xc0Q\x00\x7f\x00\x00\x00lang6\x00n\xc1Q\x00\x7f\x00\x00\x00lang7\x00\xee\xc1Q\x00\x7f\x00\x00\x00lang8\x00n\xc2Q\x00\x7f\x00\x00\x00lang9\x00\xee\xc2Q\x00\x7f")])),
    room_by_mode: PlainBytes::new(with_runs::<16>(&[(5, b"("), (10, b"\x05\x00\x00(\n")])),
    _tail: PlainBytes::new([0; 9]),
        }
    }
}

/// A record of a drawing list on the pages (`install::draw_string_list`): 16 bytes, the
/// offsets (dx, dy) from the list's place, the font's link and the text's string id (the dwords in this
/// order are its bytes).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct StringRecord {
    pub dx: PlainDword,
    pub dy: PlainDword,
    pub font: LinkCell,
    pub string: PlainDword,
}

impl StringRecord {
    /// A record drawing text `string` in `font` at (dx, dy) from the list's place; the font is
    /// kept as its link.
    pub const fn new(dx: u32, dy: u32, font: FontId, string: u32) -> Self {
        Self {
            dx: PlainDword::new(dx),
            dy: PlainDword::new(dy),
            font: LinkCell::new(PageLink::of_font(font)),
            string: PlainDword::new(string),
        }
    }

    /// The link the record keeps for its font.
    pub(super) fn font_link(&self) -> PageLink {
        self.font.get()
    }
}

/// A record of a glyph list on the pages (`install::blit_glyph_list`): 16 bytes, the offsets
/// from the list's place, the glyph's number and the palette's link.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct GlyphRecord {
    pub dx: PlainDword,
    pub dy: PlainDword,
    pub glyph: PlainDword,
    pub palette: LinkCell,
}

impl GlyphRecord {
    /// A record blitting glyph `glyph` with `palette` at (dx, dy) from the list's place.
    pub const fn new(dx: u32, dy: u32, glyph: u32, palette: PageLink) -> Self {
        Self {
            dx: PlainDword::new(dx),
            dy: PlainDword::new(dy),
            glyph: PlainDword::new(glyph),
            palette: LinkCell::new(palette),
        }
    }

    /// The link the record keeps for its palette.
    pub(super) fn palette_link(&self) -> PageLink {
        self.palette.get()
    }
}

/// The byte of `InstallerNames::script` that is the mark: 1 goes on into the script's rest (the
/// save and settings folders removed too), 5 ends the script there (`install::removal_page`).
pub(super) const SCRIPT_MARK: usize = 17;

/// The page: the fonts and the pages' tables.
#[derive(rc2k_derive::Cells)]
pub struct InstallerPages {
    /// The copying page's two texts.
    pub texts: [StringRecord; 2],
    /// The error message.
    pub msg_error: [StringRecord; 1],
    /// The cancel message.
    pub msg_cancel: [StringRecord; 1],
    /// The removal page's strings.
    pub remove_strings: [StringRecord; 1],
    /// The bar's glyph list.
    pub bar_glyphs_list: [GlyphRecord; 2],
    /// The bar's x, y, width, height.
    pub bar: PlainBytes<16>,
    /// Its frame.
    pub frame: PlainBytes<17>,
    /// The page's fonts (links to the font objects).
    pub fonts: [PlainDword; 25],
    /// Font by mark, 8 bytes a kind (`install::lay_out_pieces`, from the fifth byte).
    pub kind_fonts: PlainBytes<108>,
    /// The characters a folder name may not hold.
    pub bad_folder_chars: PlainBytes<20>,
}

impl InstallerPages {
    pub const fn new() -> Self {
        InstallerPages {
    texts: [StringRecord::new(0x1A9, 0xAF, fonts::TITLE, 0xAF), StringRecord::new(0x1A9, 0xFA, fonts::CAPTION_A, 0xB0)],
    msg_error: [StringRecord::new(0x1A9, 0x15E, fonts::TITLE, 0xB1)],
    msg_cancel: [StringRecord::new(0x1A9, 0x190, fonts::TITLE, 0xB3)],
    remove_strings: [StringRecord::new(0x1A9, 0x12C, fonts::TITLE, 0xBA)],
    bar_glyphs_list: [GlyphRecord::new(0xE1, 0x12C, 0, PageLink::blended_palette()), GlyphRecord::new(0x1A9, 0x12C, 1, PageLink::blended_palette())],
    bar: PlainBytes::new(with_runs::<16>(&[(0, b"\xe0\x00\x00\x00+\x01\x00\x00\x92\x01\x00\x00\x1f")])),
    frame: PlainBytes::new(with_runs::<17>(&[(0, b"\xdf\x00\x00\x00*\x01\x00\x00\x94\x01\x00\x00!")])),
    fonts: [
        PlainDword::new(0x0),
        PlainDword::new(PageLink::of_font(fonts::MEASURE_23).raw()),
        PlainDword::new(PageLink::of_font(fonts::MEASURE).raw()),
        PlainDword::new(PageLink::of_font(fonts::MEASURE).raw()),
        PlainDword::new(PageLink::of_font(FontId::numbered(53)).raw()),
        PlainDword::new(PageLink::of_font(FontId::numbered(54)).raw()),
        PlainDword::new(PageLink::of_font(FontId::numbered(52)).raw()),
        PlainDword::new(PageLink::of_font(FontId::numbered(50)).raw()),
        PlainDword::new(PageLink::of_font(fonts::ITEM_GREY).raw()),
        PlainDword::new(PageLink::of_font(FontId::numbered(49)).raw()),
        PlainDword::new(PageLink::of_font(fonts::ARCADE_BUTTON_PULSE).raw()),
        PlainDword::new(PageLink::of_font(fonts::STAGE_ITEM_B).raw()),
        PlainDword::new(PageLink::of_font(fonts::CAPTION_B).raw()),
        PlainDword::new(PageLink::of_font(fonts::VALUES).raw()),
        PlainDword::new(PageLink::of_font(fonts::LABELS).raw()),
        PlainDword::new(PageLink::of_font(fonts::HEADING_23).raw()),
        PlainDword::new(PageLink::of_font(fonts::LIST).raw()),
        PlainDword::new(PageLink::of_font(fonts::ITEM_PLAIN).raw()),
        PlainDword::new(PageLink::of_font(fonts::TITLE).raw()),
        PlainDword::new(PageLink::of_font(fonts::LIST).raw()),
        PlainDword::new(PageLink::of_font(fonts::ITEM_PLAIN).raw()),
        PlainDword::new(PageLink::of_font(fonts::TITLE).raw()),
        PlainDword::new(PageLink::of_font(fonts::ARCADE_BUTTON).raw()),
        PlainDword::new(PageLink::of_font(fonts::STAGE_TITLE).raw()),
        PlainDword::new(PageLink::of_font(fonts::CAPTION_A).raw()),
    ],
    kind_fonts: PlainBytes::new(with_runs::<108>(&[(0, b"\x12\x00\t\x0c\x00\x02\x00\x1e\x17\x00\x08\x0b\x01\x02\x00\x1e\n\x00\x07\x10\x02\x02\x00\x1e"), (32, b"\x12\x00\t\x0c\x00\x02\x00\x1e"), (64, b"\x0c\x00\t\x12\x00\x02\x00\x1e\x15\x00\x15\x15\x00\x03\x00,\x0c\x00\x0c\x0c\x00\x02\x00\x1e\x12\x00\x12\x12\x00\x02\x00\x1e\x18\x00\x18\x18\x00\x02\x00\x1e\t")])),
    bad_folder_chars: PlainBytes::new(with_runs::<20>(&[(0, b"\"+*,;=/:<>?\\{}[]|\x7f")])),
        }
    }
}

impl InstallerPages {
    /// The byte `index` of `kind_fonts` (8 bytes a font kind); an index past the table ends
    /// the program as every checked installer read does.
    pub(super) fn kind_font(install: &crate::install::owner::Install, index: u32) -> u8 {
        if index >= 108 {
            crate::install::installer::refuse(crate::errors::BadInput(
                "a font kind past the installer's table",
            ));
        }
        install.pages.kind_fonts.get(index as usize)
    }

    /// Where font `index` of the page's fonts is (the table keeps links to the font objects). A
    /// slot that names no font object (the unused slot 0, an index off the table) is refused.
    pub fn font_of(
        install: &crate::install::owner::Install,
        index: u32,
    ) -> Result<crate::gfx::typefaces::FontId, crate::errors::BadInput> {
        let cell = install
            .pages
            .fonts
            .get(index as usize)
            .ok_or(crate::errors::BadInput("a page font past the table"))?;
        PageLink::from_raw(cell.get())
            .font()
            .map_err(|_| crate::errors::BadInput("a page font that is no font object"))
    }
}

/// The installer's path buffers (0x100 bytes each).
#[derive(rc2k_derive::Cells)]
pub struct InstallerPaths {
    /// Padding that keeps every field its alignment (the struct is 16-byte aligned).
    pub _lead: PlainBytes<8>,
    /// The folder installed from: the CD or the download folder. The install record's `cdrom=`
    /// line holds it.
    pub game_dir: PlainBytes<256>,
    /// The folder installed into.
    pub installed: PlainBytes<256>,
    /// A path being built.
    pub path_a: PlainBytes<256>,
    /// Another path being built.
    pub path_b: PlainBytes<256>,
    /// The script's path; an edit's backup.
    pub script_path: PlainBytes<256>,
    /// The folder typed on the page.
    pub typed: PlainBytes<256>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: PlainBytes<8>,
}

impl InstallerPaths {
    pub const fn new() -> Self {
        InstallerPaths {
            _lead: PlainBytes::new([0; 8]),
            game_dir: PlainBytes::new([0; 256]),
            installed: PlainBytes::new([0; 256]),
            path_a: PlainBytes::new([0; 256]),
            path_b: PlainBytes::new([0; 256]),
            script_path: PlainBytes::new([0; 256]),
            typed: PlainBytes::new([0; 256]),
            _tail: PlainBytes::new([0; 8]),
        }
    }
}

/// The installer's palette buffer.
#[derive(rc2k_derive::Cells)]
pub struct InstallerPalette {
    /// Padding that keeps every field its alignment (the struct is 16-byte aligned).
    pub _lead: PlainBytes<4>,
    /// The installer's palette file read here (0x300 bytes, `install::open_installer`).
    pub palette: PlainBytes<768>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: PlainBytes<12>,
}

impl InstallerPalette {
    pub const fn new() -> Self {
        InstallerPalette {
            _lead: PlainBytes::new([0; 4]),
            palette: PlainBytes::new([0; 768]),
            _tail: PlainBytes::new([0; 12]),
        }
    }
}
