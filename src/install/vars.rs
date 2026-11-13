//! The installer's variables: the flags and counters of its run, the folder and file lists, the
//! registry names and scripts, the language list, and the pages' tables.
//!
//! Structs of cells, with the old layout's fields in its order and its gaps. The pages' tables
//! hold handles (`install::page_links`) in place of addresses. The pages' lists,
//! handler records, edit record and boxes are typed records (`menu::list_handle`); the pages
//! are read by place through their fields in order (`install::pages_map`), which runs on into
//! the bytes after a record.

use crate::gfx::typefaces::{FontId, fonts};
use crate::install::page_links::{LinkCell, ListId, PageLink};
use crate::menu::list_handle::{
    BoxRect, EditRecord, Item, ListData3, ListData4, ListData5, ListData6, ListData10, ListData18,
    ListData26, ListHeader,
};
use crate::session::{PlainByte, PlainBytes, PlainDword, with_runs};

/// The installer's flags, counters and lists.
#[derive(rc2k_derive::Cells)]
pub struct InstallerRun {
    /// Padding that keeps every field its alignment (the struct is 16-byte aligned).
    pub _lead: PlainBytes<8>,
    /// The installation is this version (`install::mark_main_page`).
    pub same_version: PlainDword,
    /// -u: the update run (`install::install_clio`).
    pub updating: PlainDword,
    /// -i: the paths were given on the command line.
    pub from_command: PlainDword,
    /// The shortcuts are added.
    pub add_items: PlainDword,
    /// The shortcuts are removed (`install::uninstall`).
    pub remove_items: PlainDword,
    /// DirectX's set-up is run at the end.
    pub run_dxsetup: PlainDword,
    /// -p: installing (`install::check_at_start`).
    pub installing: PlainDword,
    /// The registration is offered.
    pub offer_registration: PlainDword,
    /// The registration program is run at the end.
    pub run_registration: PlainDword,
    /// The registration program's script: a count, "ereg\eregip.exe" from +2
    /// (`install::run_after_exit`).
    pub ereg: PlainBytes<20>,
    /// 1 a bad folder, 2 no room.
    pub path_problem: PlainDword,
    /// The lack of room is ignored.
    pub ignore_room: PlainDword,
    /// -i's first digit.
    pub given_mode: PlainDword,
    /// -i's second digit.
    pub given_language: PlainDword,
    /// 26 of (found, free KB), by drive letter.
    pub drives: [PlainDword; 52],
    /// The drives found.
    pub drive_count: PlainDword,
    /// The first letter looked at.
    pub first_drive: PlainDword,
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
    /// `install::name_tables::FOLDERS`).
    pub dirs: PlainDword,
    /// The files' names (a 0x40000-byte buffer; 0 now:
    /// `install::name_tables::FILES`).
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
    /// The open registry key.
    pub reg_key: PlainDword,
}

impl InstallerRun {
    pub const fn new() -> Self {
        InstallerRun {
            _lead: PlainBytes::new([0; 8]),
            same_version: PlainDword::new(0x0),
            updating: PlainDword::new(0x0),
            from_command: PlainDword::new(0x0),
            add_items: PlainDword::new(0x0),
            remove_items: PlainDword::new(0x0),
            run_dxsetup: PlainDword::new(0x0),
            installing: PlainDword::new(0x0),
            offer_registration: PlainDword::new(0x0),
            run_registration: PlainDword::new(0x0),
            ereg: PlainBytes::new(with_runs::<20>(&[(0, b";4ereg\\eregip.exe")])),
            path_problem: PlainDword::new(0x0),
            ignore_room: PlainDword::new(0x0),
            given_mode: PlainDword::new(0x2),
            given_language: PlainDword::new(0x0),
            drives: [const { PlainDword::new(0x0) }; 52],
            drive_count: PlainDword::new(0x0),
            first_drive: PlainDword::new(0x63),
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
            reg_key: PlainDword::new(0x0),
        }
    }
}

/// The registry names, the installed record, the scripts and file names, and the languages.
#[derive(rc2k_derive::Cells)]
pub struct InstallerNames {
    /// "SOFTWARE\Magnetic Fields\RC99".
    pub key_name: PlainBytes<30>,
    /// "status".
    pub status_name: PlainBytes<7>,
    /// "source".
    pub source_name: PlainBytes<7>,
    /// The installed status byte (`install::check_installed_version`).
    pub status: PlainByte,
    /// The folder installed from (0xFF bytes used).
    pub source: PlainBytes<259>,
    /// The size of the source folder's name.
    pub source_size: PlainDword,
    /// The registry value's type.
    pub reg_type: PlainDword,
    /// A gap of 176 byte(s): nothing reads it.
    pub _gap_51bc98: PlainBytes<176>,
    /// The count script (`install::install_files`).
    pub script_count: PlainBytes<200>,
    /// The uninstall script, 17 bytes, then the mark (`SCRIPT_MARK`), then the rest of the script
    /// to its end (5) at +0x2F: the save and settings folders. `install::run_script` reads on
    /// into the rest when the mark is 1; 5 in the mark ends the script there.
    pub script: PlainBytes<{ SCRIPT_MARK + 1 + 30 }>,
    /// "ral.exe" (in ".\ral.exe").
    pub program: PlainBytes<8>,
    /// "ral.exe".
    pub ral_exe: PlainBytes<8>,
    /// A gap of 4 byte(s): nothing reads it.
    pub _gap_51be50: PlainBytes<4>,
    /// "redist\directx\dxsetup.exe".
    pub dxsetup: PlainBytes<27>,
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
    key_name: PlainBytes::new(with_runs::<30>(&[(0, b"SOFTWARE\\Magnetic Fields\\RC99")])),
    status_name: PlainBytes::new(with_runs::<7>(&[(0, b"status")])),
    source_name: PlainBytes::new(with_runs::<7>(&[(0, b"source")])),
    status: PlainByte::new(0x0),
    source: PlainBytes::new([0; 259]),
    source_size: PlainDword::new(0x0),
    reg_type: PlainDword::new(0x0),
    _gap_51bc98: PlainBytes::new(with_runs::<176>(&[(20, b"\x18"), (44, b"PROGMAN\x00[CreateGroup(Rally Championship)]\x00[DeleteGroup(Rally Championship)]\x00[AddItem(\x00,Rally)]\x00,Uninstall Rally)]\x00,View Readme)]")])),
    script_count: PlainBytes::new(with_runs::<200>(&[(0, b"\x03ral.exe\x03*.txt\x03*.pdf\x08\x02\x01files\x04a\x04b\x04d\x08\x03\x04fmv\t\x01g\x04frnt\x04graphics\x08\x03\x04slides\t\x02\x04levels\x04%locale\x04m\x04maps\x08\x03\x04music\t\x04replay\x04s\x01%speech\x04m\x04f\x08\x03\x04cars\x04rally\t\x02\x02\t\x01savedata\x01carsetup\x02\x01replay\x02\x01savegame\x02\x01snapshot\x02\x02\x01var\x03ral.cfg\x02\x06\x05")])),
    script: PlainBytes::new(with_runs::<{ SCRIPT_MARK + 1 + 30 }>(&[(0, b"\x03*.*\x01files\x02\x04files"), (SCRIPT_MARK, b"\x01"), (SCRIPT_MARK + 1, b"savedata\x02\x04savedata\x01var\x02\x04var\x05.\\")])),
    program: PlainBytes::new(with_runs::<8>(&[(0, b"ral.exe")])),
    ral_exe: PlainBytes::new(with_runs::<8>(&[(0, b"ral.exe")])),
    _gap_51be50: PlainBytes::new(with_runs::<4>(&[(0, b" -u")])),
    dxsetup: PlainBytes::new(with_runs::<27>(&[(0, b"redist\\directx\\dxsetup.exe")])),
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
/// offsets from the list's place, the font's link and the text's string id (the dwords in this
/// order are its bytes in the pages' run, `install::pages_map`).
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
    pub fn font_link(&self) -> PageLink {
        self.font.get()
    }
}

/// A record of a glyph list on the pages (`install::blit_glyph_list`): 16 bytes, the offsets
/// from the list's place, the glyph's number and the palette's link (the dwords in this order
/// are its bytes in the pages' run, `install::pages_map`).
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
    pub fn palette_link(&self) -> PageLink {
        self.palette.get()
    }
}

/// The byte of `InstallerNames::script` that is the mark: 1 goes on into the script's rest (the
/// save and settings folders removed too), 5 ends the script there (`install::finish_page`).
pub const SCRIPT_MARK: usize = 17;

/// The page: its pieces, the list menu's state, the fonts and the pages' tables. The pages'
/// bytes are run on through the fields in declaration order (`install::pages_map::PagesChain`,
/// its frozen table `NAMED_PLACES`), not through the record's memory layout.
#[derive(rc2k_derive::Cells)]
pub struct InstallerPages {
    /// Padding before the pages' run starts: the offsets the pieces keep for items count from it.
    pub _lead: PlainBytes<7>,
    /// The page's pieces: a count, then 0x30 bytes each.
    pub layout: PlainBytes<4096>,
    /// The list's choice.
    pub choice: PlainDword,
    /// The choice before the key.
    pub choice_before: PlainDword,
    /// How many repeats of the key.
    pub repeats: PlainDword,
    /// The last key (`race::viewer` reads it too).
    pub last_key: PlainByte,
    /// A key was taken.
    pub key_taken: PlainByte,
    /// The list's answer.
    pub list_answer: PlainByte,
    /// The list is left.
    pub list_leave: PlainByte,
    /// A gap of 4 byte(s): nothing reads it.
    pub _gap_51d417: PlainBytes<4>,
    /// The key's clock.
    pub key_clock: PlainDword,
    /// The frame's clock.
    pub frame_clock: PlainDword,
    /// The move's clock.
    pub move_clock: PlainDword,
    /// A gap of 12 byte(s): nothing reads it.
    pub _gap_51d427: PlainBytes<12>,
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
    /// Font by mark, 8 bytes a kind (`install::lay_out_pieces`, from +4).
    pub kind_fonts: PlainBytes<108>,
    /// The page entries (dwords): the page's list, then the entries its answers lead on to
    /// (`page_records::PageEntry`); the normal run's first entry is first, then the update run's
    /// and the drive page's (`chosen_menu`).
    pub page_table: [PlainDword; 55],
    /// `language_page`'s list: the nine languages and cancel.
    pub language_menu: ListData10,
    /// `mark_main_page`'s list: play, uninstall, registration and the rest of the main page. The
    /// first item's kind, marks and text and the second item's text are set by
    /// `install::check_at_start::set_script_entry`.
    pub main_menu: ListData5,
    /// `mode_page`'s list (the installation kind).
    pub mode_menu: ListData5,
    /// `install::drive_page`'s list when there are 8 or fewer drives (`install::list_drives` sets
    /// the item count and the page entry's link).
    pub drive_menu_8: ListData10,
    /// The drive page's list when there are 9 to 16 drives.
    pub drive_menu_16: ListData18,
    /// The drive page's list when there are more drives.
    pub menu_more: ListData26,
    /// `folder_page`'s list; its second item is the edit item (`folder_edit`), whose text link
    /// (+8) `run_installer` sets on Unix.
    pub folder_menu: ListData6,
    /// `room_page`'s list.
    pub room_menu: ListData3,
    /// `dxsetup_page`'s list.
    pub dxsetup_menu: ListData4,
    /// `finish_page`'s list.
    pub finish_menu: ListData4,
    /// The pages that are only an action: a list header with no items (flags 0x40), each; the
    /// third's action is where `check_clio_disc` puts the update step.
    pub action_pages: [ListHeader; 4],
    /// The folder item's edit record.
    pub folder_edit: EditRecord,
    /// The title's item.
    pub title: Item,
    /// The boxes items are placed in, `{x, y, width, height}` each: the title item's (0), the
    /// title's (1), the drive texts' (2 .. 8).
    pub boxes: [BoxRect; 9],
    /// The characters a folder name may not hold.
    pub bad_folder_chars: PlainBytes<20>,
    /// Four bytes after the characters (a read on past them reaches these).
    pub _tail: PlainBytes<4>,
}

impl InstallerPages {
    pub const fn new() -> Self {
        InstallerPages {
    _lead: PlainBytes::new([0; 7]),
    layout: PlainBytes::new([0; 4096]),
    choice: PlainDword::new(0x0),
    choice_before: PlainDword::new(0x0),
    repeats: PlainDword::new(0x1),
    last_key: PlainByte::new(0x0),
    key_taken: PlainByte::new(0x0),
    list_answer: PlainByte::new(0x0),
    list_leave: PlainByte::new(0x0),
    _gap_51d417: PlainBytes::new([0; 4]),
    key_clock: PlainDword::new(0x0),
    frame_clock: PlainDword::new(0x0),
    move_clock: PlainDword::new(0x0),
    _gap_51d427: PlainBytes::new([0; 12]),
    texts: [StringRecord::new(0x1A9, 0xAF, fonts::TITLE, 0xAF), StringRecord::new(0x1A9, 0xFA, fonts::CAPTION_A, 0xB0)],
    msg_error: [StringRecord::new(0x1A9, 0x15E, fonts::TITLE, 0xB1)],
    msg_cancel: [StringRecord::new(0x1A9, 0x190, fonts::TITLE, 0xB3)],
    remove_strings: [StringRecord::new(0x1A9, 0x12C, fonts::TITLE, 0xBA)],
    bar_glyphs_list: [GlyphRecord::new(0xE1, 0x12C, 0, PageLink::blended_palette()), GlyphRecord::new(0x1A9, 0x12C, 1, PageLink::blended_palette())],
    bar: PlainBytes::new(with_runs::<16>(&[(0, b"\xe0\x00\x00\x00+\x01\x00\x00\x92\x01\x00\x00\x1f")])),
    frame: PlainBytes::new(with_runs::<17>(&[(0, b"\xdf\x00\x00\x00*\x01\x00\x00\x94\x01\x00\x00!")])),
    fonts: [
        PlainDword::new(0x0),
        PlainDword::new(PageLink::of_font(fonts::MEASURE_23).raw()), // IDA 0x51E624
        PlainDword::new(PageLink::of_font(fonts::MEASURE).raw()), // IDA 0x51E824
        PlainDword::new(PageLink::of_font(fonts::MEASURE).raw()), // IDA 0x51E824
        PlainDword::new(PageLink::of_font(FontId::numbered(53)).raw()), // IDA 0x51EA44
        PlainDword::new(PageLink::of_font(FontId::numbered(54)).raw()), // IDA 0x51EA64
        PlainDword::new(PageLink::of_font(FontId::numbered(52)).raw()), // IDA 0x51EA24
        PlainDword::new(PageLink::of_font(FontId::numbered(50)).raw()), // IDA 0x51E9E4
        PlainDword::new(PageLink::of_font(fonts::ITEM_GREY).raw()), // IDA 0x51EA04
        PlainDword::new(PageLink::of_font(FontId::numbered(49)).raw()), // IDA 0x51E9C4
        PlainDword::new(PageLink::of_font(fonts::ARCADE_BUTTON_PULSE).raw()), // IDA 0x51E864
        PlainDword::new(PageLink::of_font(fonts::STAGE_ITEM_B).raw()), // IDA 0x51E884
        PlainDword::new(PageLink::of_font(fonts::CAPTION_B).raw()), // IDA 0x51E844
        PlainDword::new(PageLink::of_font(fonts::VALUES).raw()), // IDA 0x51E6C4
        PlainDword::new(PageLink::of_font(fonts::LABELS).raw()), // IDA 0x51E6E4
        PlainDword::new(PageLink::of_font(fonts::HEADING_23).raw()), // IDA 0x51E6A4
        PlainDword::new(PageLink::of_font(fonts::LIST).raw()), // IDA 0x51E8C4
        PlainDword::new(PageLink::of_font(fonts::ITEM_PLAIN).raw()), // IDA 0x51E8E4
        PlainDword::new(PageLink::of_font(fonts::TITLE).raw()), // IDA 0x51E8A4
        PlainDword::new(PageLink::of_font(fonts::LIST).raw()), // IDA 0x51E8C4
        PlainDword::new(PageLink::of_font(fonts::ITEM_PLAIN).raw()), // IDA 0x51E8E4
        PlainDword::new(PageLink::of_font(fonts::TITLE).raw()), // IDA 0x51E8A4
        PlainDword::new(PageLink::of_font(fonts::ARCADE_BUTTON).raw()), // IDA 0x51E984
        PlainDword::new(PageLink::of_font(fonts::STAGE_TITLE).raw()), // IDA 0x51E9A4
        PlainDword::new(PageLink::of_font(fonts::CAPTION_A).raw()), // IDA 0x51E964
    ],
    kind_fonts: PlainBytes::new(with_runs::<108>(&[(0, b"\x12\x00\t\x0c\x00\x02\x00\x1e\x17\x00\x08\x0b\x01\x02\x00\x1e\n\x00\x07\x10\x02\x02\x00\x1e"), (32, b"\x12\x00\t\x0c\x00\x02\x00\x1e"), (64, b"\x0c\x00\t\x12\x00\x02\x00\x1e\x15\x00\x15\x15\x00\x03\x00,\x0c\x00\x0c\x0c\x00\x02\x00\x1e\x12\x00\x12\x12\x00\x02\x00\x1e\x18\x00\x18\x18\x00\x02\x00\x1e\t")])),
            page_table: [PlainDword::new(PageLink::list(ListId::Language).raw()), PlainDword::new(0x0), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::list(ListId::Main).raw()), PlainDword::new(0x0), PlainDword::new(PageLink::table(20).raw()), PlainDword::new(PageLink::table(48).raw()), PlainDword::new(PageLink::table(31).raw()), PlainDword::new(PageLink::table(51).raw()), PlainDword::new(PageLink::list(ListId::Finish).raw()), PlainDword::new(0x0), PlainDword::new(PageLink::table(45).raw()), PlainDword::new(PageLink::list(ListId::Mode).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(25).raw()), PlainDword::new(PageLink::table(25).raw()), PlainDword::new(PageLink::table(25).raw()), PlainDword::new(PageLink::list(ListId::Drives8).raw()), PlainDword::new(PageLink::table(20).raw()), PlainDword::new(PageLink::table(28).raw()), PlainDword::new(PageLink::list(ListId::Folder).raw()), PlainDword::new(PageLink::table(25).raw()), PlainDword::new(PageLink::table(34).raw()), PlainDword::new(PageLink::list(ListId::Finish).raw()), PlainDword::new(PageLink::table(11).raw()), PlainDword::new(PageLink::table(45).raw()), PlainDword::new(PageLink::list(ListId::InstallFiles).raw()), PlainDword::new(PageLink::table(28).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(PageLink::table(39).raw()), PlainDword::new(PageLink::list(ListId::Room).raw()), PlainDword::new(PageLink::table(25).raw()), PlainDword::new(PageLink::table(34).raw()), PlainDword::new(PageLink::list(ListId::Dxsetup).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(PageLink::list(ListId::Uninstall).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(PageLink::list(ListId::OpenInstalled).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(PageLink::list(ListId::AskRegistration).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(0x9)],
            language_menu: ListData10::new(ListHeader::new(0x00, 0x0A, 0x009B, 0x0, 0x414199, 0x0, 0x0, 0x0), [
                Item::new(0x01, 0x01, 0x102B, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x02, 0x102C, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x03, 0x102D, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x04, 0x102E, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x05, 0x102F, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x06, 0x1030, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x07, 0x1031, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x08, 0x1032, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x09, 0x1033, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x00, 0x0099, 0x0, 0x0, 0x0),
            ]),
            main_menu: ListData5::new(ListHeader::new(0x00, 0x05, 0x0095, 0x0, 0x4141EF, 0x0, 0x0, 0x0), [
                Item::new(0x01, 0x01, 0x0096, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x02, 0x0097, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x03, 0x0098, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x04, 0x057C, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x00, 0x0099, 0x0, 0x0, 0x0),
            ]),
            mode_menu: ListData5::new(ListHeader::new(0x00, 0x05, 0x009A, 0x2, 0x41422B, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0C, 0x00A1, 0x0, 0x0, 0x415304),
                Item::new(0x01, 0x01, 0x00A2, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x02, 0x00A3, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x03, 0x00A4, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x00, 0x00BF, 0x0, 0x0, 0x0),
            ]),
            drive_menu_8: ListData10::new(ListHeader::new(0x00, 0x0A, 0x009A, 0x2, 0x41425E, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0C, 0x009D, PageLink::box_at(7).raw(), 0x0, 0x0),
                Item::new(0x01, 0x00, 0x00BF, PageLink::box_at(8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1011, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1012, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1013, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1014, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1015, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1016, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1017, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1018, PageLink::box_at(3).raw(), 0x0, 0x0),
            ]),
            drive_menu_16: ListData18::new(ListHeader::new(0x00, 0x12, 0x009A, 0x2, 0x41425E, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0C, 0x009D, PageLink::box_at(7).raw(), 0x0, 0x0),
                Item::new(0x01, 0x00, 0x00BF, PageLink::box_at(8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1011, PageLink::box_at(5).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1012, PageLink::box_at(5).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1013, PageLink::box_at(5).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1014, PageLink::box_at(5).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1015, PageLink::box_at(5).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1016, PageLink::box_at(5).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1017, PageLink::box_at(5).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1018, PageLink::box_at(5).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1019, PageLink::box_at(6).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101A, PageLink::box_at(6).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101B, PageLink::box_at(6).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101C, PageLink::box_at(6).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101D, PageLink::box_at(6).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101E, PageLink::box_at(6).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101F, PageLink::box_at(6).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1020, PageLink::box_at(6).raw(), 0x0, 0x0),
            ]),
            menu_more: ListData26::new(ListHeader::new(0x00, 0x1A, 0x009A, 0x2, 0x41425E, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0C, 0x009D, PageLink::box_at(7).raw(), 0x0, 0x0),
                Item::new(0x01, 0x00, 0x00BF, PageLink::box_at(8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1011, PageLink::box_at(2).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1012, PageLink::box_at(2).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1013, PageLink::box_at(2).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1014, PageLink::box_at(2).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1015, PageLink::box_at(2).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1016, PageLink::box_at(2).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1017, PageLink::box_at(2).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1018, PageLink::box_at(2).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1019, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101A, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101B, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101C, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101D, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101E, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101F, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1020, PageLink::box_at(3).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1021, PageLink::box_at(4).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1022, PageLink::box_at(4).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1023, PageLink::box_at(4).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1024, PageLink::box_at(4).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1025, PageLink::box_at(4).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1026, PageLink::box_at(4).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1027, PageLink::box_at(4).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1028, PageLink::box_at(4).raw(), 0x0, 0x0),
            ]),
            folder_menu: ListData6::new(ListHeader::new(0x00, 0x06, 0x009A, 0x0, 0x4142F5, 0x0, 0x0, 0x41534E), [
                Item::new(0x05, 0x0C, 0x009E, 0x0, 0x0, 0x0),
                Item::new(0x24, 0x01, 0x0000, 0x0, PageLink::typed(2).raw(), PageLink::folder_edit().raw()),
                Item::new(0x05, 0x0A, 0x00AA, 0x0, 0x0, 0x0),
                Item::new(0x05, 0x0A, 0x00AC, 0x0, 0x0, 0x0),
                Item::new(0x05, 0x0A, 0x00AD, 0x0, 0x0, 0x0),
                Item::new(0x05, 0x0A, 0x00AB, 0x0, 0x0, 0x0),
            ]),
            room_menu: ListData3::new(ListHeader::new(0x00, 0x03, 0x009A, 0x0, 0x4142C7, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0A, 0x00AD, 0x0, 0x0, 0x415304),
                Item::new(0x01, 0x01, 0x00AE, 0x0, 0x4142DA, 0x0),
                Item::new(0x01, 0x00, 0x00BF, 0x0, 0x0, 0x0),
            ]),
            dxsetup_menu: ListData4::new(ListHeader::new(0x00, 0x04, 0x009A, 0x0, 0x41432F, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0B, 0x00B2, 0x0, 0x0, 0x415304),
                Item::new(0x05, 0x0C, 0x00B4, 0x0, 0x0, 0x415304),
                Item::new(0x01, 0x01, 0x056C, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x00, 0x056B, 0x0, 0x0, 0x0),
            ]),
            finish_menu: ListData4::new(ListHeader::new(0x00, 0x04, 0x00B6, 0x2, 0x414347, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0A, 0x00B7, 0x0, 0x0, 0x415304),
                Item::new(0x01, 0x01, 0x00B8, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x02, 0x00B9, 0x0, 0x0, 0x0),
                Item::new(0x01, 0x00, 0x00BF, 0x0, 0x0, 0x0),
            ]),
            action_pages: [ListHeader::new(0x40, 0, 0, 0, 0x4130CD, 0, 0, 0), ListHeader::new(0x40, 0, 0, 0, 0x4134DF, 0, 0, 0), ListHeader::new(0x40, 0, 0, 0, 0x413629, 0, 0, 0), ListHeader::new(0x40, 0, 0, 0, 0x413650, 0, 0, 0)],
            folder_edit: EditRecord::new(0x02, 0x7C, 0x00A0, 0x0B, 0x00, 0x02E9),
            title: Item::new(0x05, 0x09, 0x0000, PageLink::box_at(0).raw(), 0x0, 0x0),
            boxes: [BoxRect::new(50, 150, 750, 44), BoxRect::new(50, 150, 750, 400), BoxRect::new(125, 150, 200, 400), BoxRect::new(325, 150, 200, 400), BoxRect::new(525, 150, 200, 400), BoxRect::new(225, 150, 200, 400), BoxRect::new(425, 150, 200, 400), BoxRect::new(50, 200, 750, 30), BoxRect::new(50, 495, 750, 30)],
    bad_folder_chars: PlainBytes::new(with_runs::<20>(&[(0, b"\"+*,;=/:<>?\\{}[]|\x7f")])),
    _tail: PlainBytes::new([0; 4]),
        }
    }
}

impl InstallerPages {
    /// The byte `index` of `kind_fonts` (8 bytes a font kind); an index past the table ends
    /// the program as every checked installer read does.
    pub fn kind_font(install: &crate::install::owner::Install, index: u32) -> u8 {
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

/// The installer's path buffers (0x100 bytes each; the network's lobby edits two of them).
#[derive(rc2k_derive::Cells)]
pub struct InstallerPaths {
    /// Padding that keeps every field its alignment (the struct is 16-byte aligned).
    pub _lead: PlainBytes<8>,
    /// The folder installed into (also the first of `-i`'s paths).
    pub game_dir: PlainBytes<256>,
    /// The folder installed from (also `-i`'s second path).
    pub installed: PlainBytes<256>,
    /// A path being built (the lobby's number edited as text).
    pub path_a: PlainBytes<256>,
    /// Another path being built.
    pub path_b: PlainBytes<256>,
    /// The script's path; an edit's backup (the lobby's text before editing).
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
