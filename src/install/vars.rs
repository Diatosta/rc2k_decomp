//! The installer's variables, out of RAL.EXE's image : the flags
//! and counters of its run, the folder and file lists, the registry names and scripts, the
//! language list, and the pages' tables. Split by owner from the rest of the
//! image's scattered variables (`rest::low`); the network's and the strings' neighbours are
//! in their own modules.
//!
//! `#[repr(C)]` structs of atomic cells with the image's layout, first values the image's
//! (`tests`); `moved::MOVED` points the image's addresses at them (`home`): the pages'
//! tables hold pointers into each other and into the installer's texts, as IDA addresses
//! resolved where they are read. The pages' lists, handler records, edit record and boxes
//! are typed records (`menu::list_handle`) with the image's bytes; a list is read by place
//! (`install::installer`'s `rd`/`rb`), which runs on into the bytes after it as the
//! original's reads do.

use crate::install::page_links::PageLink;
use crate::menu::list_handle::{
    BoxRect, EditRecord, Item, ListData3, ListData4, ListData5, ListData6, ListData10, ListData18,
    ListData26, ListHeader,
};
use crate::session::{PlainByte, PlainBytes, PlainDword, PlainLoose, with_runs};

/// The installer's flags, counters and lists (IDA 0x51B9C8 .. 0x51BB60, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct InstallerRun {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: PlainBytes<8>,
    /// 0x51B9C8: `dword_51B9C8`: the installation is this version (`install::mark_main_page`).
    pub same_version: PlainDword,
    /// 0x51B9CC: `dword_51B9CC`: -u: the update run (`install::install_clio`).
    pub updating: PlainDword,
    /// 0x51B9D0: `dword_51B9D0`: -i: the paths were given on the command line.
    pub from_command: PlainDword,
    /// 0x51B9D4: `g_addStartItems`: the shortcuts are added.
    pub add_items: PlainDword,
    /// 0x51B9D8: `g_removeStartItems`: the shortcuts are removed (`install::uninstall`).
    pub remove_items: PlainDword,
    /// 0x51B9DC: `g_runDxSetup`: DirectX's set-up is run at the end.
    pub run_dxsetup: PlainDword,
    /// 0x51B9E0: `g_installing`: -p: installing (`install::check_at_start`).
    pub installing: PlainDword,
    /// 0x51B9E4: `dword_51B9E4`: the registration is offered.
    pub offer_registration: PlainDword,
    /// 0x51B9E8: `g_runRegistration`: the registration program is run at the end.
    pub run_registration: PlainDword,
    /// 0x51B9EC: `a4eregEregipExe`: the registration program's script: a count, "ereg\eregip.exe" from +2 (`install::run_after_exit`).
    pub ereg: PlainBytes<20>,
    /// 0x51BA00: `g_pathProblem`: 1 a bad folder, 2 no room.
    pub path_problem: PlainDword,
    /// 0x51BA04: `g_ignoreRoom`: the lack of room is ignored.
    pub ignore_room: PlainDword,
    /// 0x51BA08: `g_givenMediaMode`: -i's first digit.
    pub given_mode: PlainDword,
    /// 0x51BA0C: `g_givenLanguage`: -i's second digit.
    pub given_language: PlainDword,
    /// 0x51BA10: `dword_51BA10`: 26 of (found, free KB), by drive letter.
    pub drives: [PlainDword; 52],
    /// 0x51BAE0: `g_driveCount`: the drives found.
    pub drive_count: PlainDword,
    /// 0x51BAE4: `g_firstDrive`: the first letter looked at.
    pub first_drive: PlainDword,
    /// 0x51BAE8: `g_filesCopied`: the files copied so far.
    pub files_copied: PlainDword,
    /// 0x51BAEC: `g_filesToCopy`: the files to copy.
    pub files_to_copy: PlainDword,
    /// 0x51BAF0: `dword_51BAF0`: `install::install_files`' spare.
    pub copy_spare: PlainDword,
    /// 0x51BAF4: Not named: 4 bytes (a gap, or data nothing reads here).
    pub _gap_51baf4: PlainBytes<4>,
    /// 0x51BAF8: `g_roomNeededKb`: KB needed.
    pub room_needed: PlainDword,
    /// 0x51BAFC: `stru_51BAFC`: the installer's memory list (a `MemoryList` of 16 bytes, then its work).
    pub buffers: PlainBytes<64>,
    /// 0x51BB3C: `g_installBarGlyphs`: the progress bar's glyph table.
    pub bar_glyphs: PlainDword,
    /// 0x51BB40: `g_dirNames`: the folders' names (a 0x10000-byte buffer; 0 now:
    /// `install::name_tables::FOLDERS`).
    pub dirs: PlainDword,
    /// 0x51BB44: `g_fileNames`: the files' names (a 0x40000-byte buffer; 0 now:
    /// `install::name_tables::FILES`).
    pub files: PlainDword,
    /// 0x51BB48: `g_dirCount`: the folders listed.
    pub dir_count: PlainDword,
    /// 0x51BB4C: `g_fileCount`: the files listed.
    pub file_count: PlainDword,
    /// 0x51BB50: `g_dirScan`: the next folder to search (its number).
    pub dir_scan: PlainDword,
    /// 0x51BB54: `g_dirCursor`: the folders' table's end (its bytes).
    pub dir_cursor: PlainDword,
    /// 0x51BB58: `g_fileCursor`: the files' table's end (its bytes).
    pub file_cursor: PlainDword,
    /// 0x51BB5C: `phkResult`: the open registry key.
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
            buffers: PlainBytes::new([0; 64]),
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

impl InstallerRun {
    pub const START: usize = 0x51B9C8;
    pub const LEN: usize = 0x198;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&raw const crate::install::owner::INSTALL.get().run) + 8)
            as usize
    }
}

/// The registry names, the installed record, the scripts and file names, the languages (IDA 0x51BB60 .. 0x51C407, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct InstallerNames {
    /// 0x51BB60: `aSoftwareMagnet`: "SOFTWARE\Magnetic Fields\RC99".
    pub key_name: PlainBytes<30>,
    /// 0x51BB7E: `aStatus`: "status".
    pub status_name: PlainBytes<7>,
    /// 0x51BB85: `aSource`: "source".
    pub source_name: PlainBytes<7>,
    /// 0x51BB8C: `g_installStatus`: the installed status byte (`install::check_installed_version`).
    pub status: PlainByte,
    /// 0x51BB8D: `g_installSource`: the folder installed from (0xFF bytes used).
    pub source: PlainBytes<259>,
    /// 0x51BC90: `dword_51BC90`: the size of the source folder's name.
    pub source_size: PlainDword,
    /// 0x51BC94: `Type`: the registry value's type.
    pub reg_type: PlainDword,
    /// 0x51BC98: Not named: 176 bytes (a gap, or data nothing reads here).
    pub _gap_51bc98: PlainBytes<176>,
    /// 0x51BD48: `g_countScript`: the count script (`install::install_files`).
    pub script_count: PlainBytes<200>,
    /// 0x51BE10: `byte_51BE10`: the uninstall script.
    pub script: PlainBytes<17>,
    /// 0x51BE21: `byte_51BE21`: a byte of the script (`install::finish_page`).
    pub script_mark: PlainByte,
    /// 0x51BE22: Not named: 30 bytes (a gap, or data nothing reads here).
    pub _gap_51be22: PlainBytes<30>,
    /// 0x51BE40: `path`: "ral.exe" (in ".\ral.exe").
    pub program: PlainBytes<8>,
    /// 0x51BE48: `aRalExe_0`: "ral.exe".
    pub ral_exe: PlainBytes<8>,
    /// 0x51BE50: Not named: 4 bytes (a gap, or data nothing reads here).
    pub _gap_51be50: PlainBytes<4>,
    /// 0x51BE54: `aRedistDirectxD`: "redist\directx\dxsetup.exe".
    pub dxsetup: PlainBytes<27>,
    /// 0x51BE6F: `aFilesReadmeRea`: "files\%readme\readme.txt".
    pub readme: PlainBytes<25>,
    /// 0x51BE88: `aFilesReadmeMan`: "files\%readme\manual.pdf".
    pub manual: PlainBytes<25>,
    /// 0x51BEA1: `aFilesDGfxInsta`: "files\d\gfx\install.bun".
    pub installer_data: PlainBytes<24>,
    /// 0x51BEB9: `aFilesDGfxInsta_0`: "files\d\gfx\install.pal".
    pub install_pal: PlainBytes<24>,
    /// 0x51BED1: `aFilesDGfxBackd`: "files\d\gfx\backdrop.pcx".
    pub backdrop: PlainBytes<25>,
    /// 0x51BEEA: `g_languageCount`: the languages found.
    pub language_count: PlainLoose,
    /// 0x51BEEE: `g_languages`: the languages found, 0x80 bytes each (eight).
    pub languages: PlainBytes<1024>,
    /// 0x51C2EE: Not named: 128 bytes (a gap, or data nothing reads here).
    pub _gap_51c2ee: PlainBytes<128>,
    /// 0x51C36E: `aLangCfg`: "lang.cfg".
    pub lang_cfg: PlainBytes<10>,
    /// 0x51C378: Not named: 127 bytes (a gap, or data nothing reads here).
    pub _gap_51c378: PlainBytes<127>,
    /// 0x51C3F7: `g_roomByMode`: KB needed by installation mode.
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
    script: PlainBytes::new(with_runs::<17>(&[(0, b"\x03*.*\x01files\x02\x04files")])),
    script_mark: PlainByte::new(0x1),
    _gap_51be22: PlainBytes::new(with_runs::<30>(&[(0, b"savedata\x02\x04savedata\x01var\x02\x04var\x05.\\")])),
    program: PlainBytes::new(with_runs::<8>(&[(0, b"ral.exe")])),
    ral_exe: PlainBytes::new(with_runs::<8>(&[(0, b"ral.exe")])),
    _gap_51be50: PlainBytes::new(with_runs::<4>(&[(0, b" -u")])),
    dxsetup: PlainBytes::new(with_runs::<27>(&[(0, b"redist\\directx\\dxsetup.exe")])),
    readme: PlainBytes::new(with_runs::<25>(&[(0, b"files\\%readme\\readme.txt")])),
    manual: PlainBytes::new(with_runs::<25>(&[(0, b"files\\%readme\\manual.pdf")])),
    installer_data: PlainBytes::new(with_runs::<24>(&[(0, b"files\\d\\gfx\\install.bun")])),
    install_pal: PlainBytes::new(with_runs::<24>(&[(0, b"files\\d\\gfx\\install.pal")])),
    backdrop: PlainBytes::new(with_runs::<25>(&[(0, b"files\\d\\gfx\\backdrop.pcx")])),
    language_count: PlainLoose::new(0x0),
    languages: PlainBytes::new([0; 1024]),
    _gap_51c2ee: PlainBytes::new([0; 128]),
    lang_cfg: PlainBytes::new(with_runs::<10>(&[(0, b"lang.cfg")])),
    _gap_51c378: PlainBytes::new(with_runs::<127>(&[(0, b"lang1\x00\xee\xbeQ\x00\x7f\x00\x00\x00lang2\x00n\xbfQ\x00\x7f\x00\x00\x00lang3\x00\xee\xbfQ\x00\x7f\x00\x00\x00lang4\x00n\xc0Q\x00\x7f\x00\x00\x00lang5\x00\xee\xc0Q\x00\x7f\x00\x00\x00lang6\x00n\xc1Q\x00\x7f\x00\x00\x00lang7\x00\xee\xc1Q\x00\x7f\x00\x00\x00lang8\x00n\xc2Q\x00\x7f\x00\x00\x00lang9\x00\xee\xc2Q\x00\x7f")])),
    room_by_mode: PlainBytes::new(with_runs::<16>(&[(5, b"("), (10, b"\x05\x00\x00(\n")])),
    _tail: PlainBytes::new([0; 9]),
        }
    }
}

impl InstallerNames {
    pub const START: usize = 0x51BB60;
    pub const LEN: usize = 0x8A7;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&raw const crate::install::owner::INSTALL.get().names)
            as usize
    }
}

/// A record of a drawing list on the pages (`install::draw_string_list`): 16 bytes at any
/// address, the offsets from the list's place, the font's link and the text's string id.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct StringRecord {
    pub dx: PlainLoose,
    pub dy: PlainLoose,
    /// The link to the font (`page_links::PageLink`).
    pub font: PlainLoose,
    pub string: PlainLoose,
}

impl StringRecord {
    pub const fn new(dx: u32, dy: u32, font: u32, string: u32) -> Self {
        Self {
            dx: PlainLoose::new(dx),
            dy: PlainLoose::new(dy),
            font: PlainLoose::new(font),
            string: PlainLoose::new(string),
        }
    }
}

/// A record of a glyph list on the pages (`install::blit_glyph_list`): 16 bytes at any
/// address, the offsets from the list's place, the glyph's number and the palette's link.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct GlyphRecord {
    pub dx: PlainLoose,
    pub dy: PlainLoose,
    pub glyph: PlainLoose,
    /// The link to the palette (`page_links::PageLink`).
    pub palette: PlainLoose,
}

impl GlyphRecord {
    pub const fn new(dx: u32, dy: u32, glyph: u32, palette: u32) -> Self {
        Self {
            dx: PlainLoose::new(dx),
            dy: PlainLoose::new(dy),
            glyph: PlainLoose::new(glyph),
            palette: PlainLoose::new(palette),
        }
    }
}

/// The page: its pieces, the list menu's state, the fonts and the pages' tables (IDA 0x51C407 .. 0x51DE2C, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct InstallerPages {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: PlainBytes<7>,
    /// 0x51C407: `g_pageLayout`: the page's pieces: a count, then 0x30 bytes each.
    pub layout: PlainBytes<4096>,
    /// 0x51D407: `g_listChoice`: the list's choice.
    pub choice: PlainLoose,
    /// 0x51D40B: `g_listChoiceBefore`: the choice before the key.
    pub choice_before: PlainLoose,
    /// 0x51D40F: `g_listRepeats`: how many repeats of the key.
    pub repeats: PlainLoose,
    /// 0x51D413: `g_listLastKey`: the last key (`race::viewer` reads it too).
    pub last_key: PlainByte,
    /// 0x51D414: `byte_51D414`: a key was taken.
    pub key_taken: PlainByte,
    /// 0x51D415: `byte_51D415`: the list's answer.
    pub list_answer: PlainByte,
    /// 0x51D416: `g_listLeave`: the list is left.
    pub list_leave: PlainByte,
    /// 0x51D417: Not named: 4 bytes (a gap, or data nothing reads here).
    pub _gap_51d417: PlainBytes<4>,
    /// 0x51D41B: `dword_51D41B`: the key's clock.
    pub key_clock: PlainLoose,
    /// 0x51D41F: `dword_51D41F`: the frame's clock.
    pub frame_clock: PlainLoose,
    /// 0x51D423: `dword_51D423`: the move's clock.
    pub move_clock: PlainLoose,
    /// 0x51D427: Not named: 12 bytes (a gap, or data nothing reads here).
    pub _gap_51d427: PlainBytes<12>,
    /// 0x51D433: `unk_51D433`: the copying page's two texts.
    pub texts: [StringRecord; 2],
    /// 0x51D453: `unk_51D453`: the error message.
    pub msg_error: [StringRecord; 1],
    /// 0x51D463: `unk_51D463`: the cancel message.
    pub msg_cancel: [StringRecord; 1],
    /// 0x51D473: `unk_51D473`: the removal page's strings.
    pub remove_strings: [StringRecord; 1],
    /// 0x51D483: `unk_51D483`: the bar's glyph list.
    pub bar_glyphs_list: [GlyphRecord; 2],
    /// 0x51D4A3: `dword_51D4A3`: the bar's x, y, width, height.
    pub bar: PlainBytes<16>,
    /// 0x51D4B3: `unk_51D4B3`: its frame.
    pub frame: PlainBytes<17>,
    /// 0x51D4C4: `dword_51D4C4`: the page's fonts (pointers).
    pub fonts: [PlainDword; 25],
    /// 0x51D528: `byte_51D528`: font by mark, 8 bytes a kind (`install::lay_out_pieces`, from +4).
    pub kind_fonts: PlainBytes<108>,
    /// 0x51D594: `off_51D594` .. `g_driveMenu`: the page entries (dwords): the page's list, then the entries its answers lead on to (`page_records::PageEntry`); the normal run's first entry is at 0x51D594, the update run's at 0x51D5D8, the drive page's (`chosen_menu`) at 0x51D5F8.
    pub page_table: [PlainDword; 55],
    /// 0x51D670: `language_page`'s list: the nine languages and cancel.
    pub language_menu: ListData10,
    /// 0x51D728: `mark_main_page`'s list: play, uninstall, registration and the rest of the main page. Its highlight is `g_readInstalledRecord`; the first item's kind, marks and text and the second item's text are set by `install::check_at_start::set_script_entry`.
    pub main_menu: ListData5,
    /// 0x51D790: `mode_page`'s list (the installation kind).
    pub mode_menu: ListData5,
    /// 0x51D7F8: `install::drive_page`'s list when there are 8 or fewer drives (`install::list_drives` sets the item count and the page entry's link).
    pub drive_menu_8: ListData10,
    /// 0x51D8B0: The drive page's list when there are 9 to 16 drives.
    pub drive_menu_16: ListData18,
    /// 0x51D9E8: The drive page's list when there are more drives.
    pub menu_more: ListData26,
    /// 0x51DBA0: `folder_page`'s list; its second item is the edit item (`folder_edit`), whose text link (+8) `run_installer` sets on Unix.
    pub folder_menu: ListData6,
    /// 0x51DC18: `room_page`'s list.
    pub room_menu: ListData3,
    /// 0x51DC60: `dxsetup_page`'s list.
    pub dxsetup_menu: ListData4,
    /// 0x51DCB8: `finish_page`'s list.
    pub finish_menu: ListData4,
    /// 0x51DD10: the pages that are only an action: a list header with no items (flags 0x40), each; the third's action (`off_51DD48`) is where `check_clio_disc` puts the update step.
    pub action_pages: [ListHeader; 4],
    /// 0x51DD70: the folder item's edit record.
    pub folder_edit: EditRecord,
    /// `unk_51DD78`: the title's item.
    pub title: Item,
    /// 0x51DD88: the boxes items are placed in, `{x, y, width, height}` each: the title item's (0), the title's (1, `unk_51DD98`), the drive texts' (2 .. 8).
    pub boxes: [BoxRect; 9],
    /// 0x51DE18: `g_badFolderChars`: the characters a folder name may not hold.
    pub bad_folder_chars: PlainBytes<20>,
    /// Up to the record's alignment (no byte of a record of cells outside a field).
    pub _tail: PlainBytes<4>,
}

impl InstallerPages {
    pub const fn new() -> Self {
        InstallerPages {
    _lead: PlainBytes::new([0; 7]),
    layout: PlainBytes::new([0; 4096]),
    choice: PlainLoose::new(0x0),
    choice_before: PlainLoose::new(0x0),
    repeats: PlainLoose::new(0x1),
    last_key: PlainByte::new(0x0),
    key_taken: PlainByte::new(0x0),
    list_answer: PlainByte::new(0x0),
    list_leave: PlainByte::new(0x0),
    _gap_51d417: PlainBytes::new([0; 4]),
    key_clock: PlainLoose::new(0x0),
    frame_clock: PlainLoose::new(0x0),
    move_clock: PlainLoose::new(0x0),
    _gap_51d427: PlainBytes::new([0; 12]),
    texts: [StringRecord::new(0x1A9, 0xAF, PageLink::of_ida(0x51E8A4).raw(), 0xAF), StringRecord::new(0x1A9, 0xFA, PageLink::of_ida(0x51E964).raw(), 0xB0)],
    msg_error: [StringRecord::new(0x1A9, 0x15E, PageLink::of_ida(0x51E8A4).raw(), 0xB1)],
    msg_cancel: [StringRecord::new(0x1A9, 0x190, PageLink::of_ida(0x51E8A4).raw(), 0xB3)],
    remove_strings: [StringRecord::new(0x1A9, 0x12C, PageLink::of_ida(0x51E8A4).raw(), 0xBA)],
    bar_glyphs_list: [GlyphRecord::new(0xE1, 0x12C, 0, PageLink::of_ida(0x4F3B74).raw()), GlyphRecord::new(0x1A9, 0x12C, 1, PageLink::of_ida(0x4F3B74).raw())],
    bar: PlainBytes::new(with_runs::<16>(&[(0, b"\xe0\x00\x00\x00+\x01\x00\x00\x92\x01\x00\x00\x1f")])),
    frame: PlainBytes::new(with_runs::<17>(&[(0, b"\xdf\x00\x00\x00*\x01\x00\x00\x94\x01\x00\x00!")])),
    fonts: [PlainDword::new(0x0), PlainDword::new(0x51e624), PlainDword::new(0x51e824), PlainDword::new(0x51e824), PlainDword::new(0x51ea44), PlainDword::new(0x51ea64), PlainDword::new(0x51ea24), PlainDword::new(0x51e9e4), PlainDword::new(0x51ea04), PlainDword::new(0x51e9c4), PlainDword::new(0x51e864), PlainDword::new(0x51e884), PlainDword::new(0x51e844), PlainDword::new(0x51e6c4), PlainDword::new(0x51e6e4), PlainDword::new(0x51e6a4), PlainDword::new(0x51e8c4), PlainDword::new(0x51e8e4), PlainDword::new(0x51e8a4), PlainDword::new(0x51e8c4), PlainDword::new(0x51e8e4), PlainDword::new(0x51e8a4), PlainDword::new(0x51e984), PlainDword::new(0x51e9a4), PlainDword::new(0x51e964)],
    kind_fonts: PlainBytes::new(with_runs::<108>(&[(0, b"\x12\x00\t\x0c\x00\x02\x00\x1e\x17\x00\x08\x0b\x01\x02\x00\x1e\n\x00\x07\x10\x02\x02\x00\x1e"), (32, b"\x12\x00\t\x0c\x00\x02\x00\x1e"), (64, b"\x0c\x00\t\x12\x00\x02\x00\x1e\x15\x00\x15\x15\x00\x03\x00,\x0c\x00\x0c\x0c\x00\x02\x00\x1e\x12\x00\x12\x12\x00\x02\x00\x1e\x18\x00\x18\x18\x00\x02\x00\x1e\t")])),
            page_table: [PlainDword::new(PageLink::of_ida(0x51D670).raw()), PlainDword::new(0x0), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D728).raw()), PlainDword::new(0x0), PlainDword::new(PageLink::of_ida(0x51D5E4).raw()), PlainDword::new(PageLink::of_ida(0x51D654).raw()), PlainDword::new(PageLink::of_ida(0x51D610).raw()), PlainDword::new(PageLink::of_ida(0x51D660).raw()), PlainDword::new(PageLink::of_ida(0x51DCB8).raw()), PlainDword::new(0x0), PlainDword::new(PageLink::of_ida(0x51D648).raw()), PlainDword::new(PageLink::of_ida(0x51D790).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D5F8).raw()), PlainDword::new(PageLink::of_ida(0x51D5F8).raw()), PlainDword::new(PageLink::of_ida(0x51D5F8).raw()), PlainDword::new(PageLink::of_ida(0x51D7F8).raw()), PlainDword::new(PageLink::of_ida(0x51D5E4).raw()), PlainDword::new(PageLink::of_ida(0x51D604).raw()), PlainDword::new(PageLink::of_ida(0x51DBA0).raw()), PlainDword::new(PageLink::of_ida(0x51D5F8).raw()), PlainDword::new(PageLink::of_ida(0x51D61C).raw()), PlainDword::new(PageLink::of_ida(0x51DCB8).raw()), PlainDword::new(PageLink::of_ida(0x51D5C0).raw()), PlainDword::new(PageLink::of_ida(0x51D648).raw()), PlainDword::new(PageLink::of_ida(0x51DD10).raw()), PlainDword::new(PageLink::of_ida(0x51D604).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(PageLink::of_ida(0x51D630).raw()), PlainDword::new(PageLink::of_ida(0x51DC18).raw()), PlainDword::new(PageLink::of_ida(0x51D5F8).raw()), PlainDword::new(PageLink::of_ida(0x51D61C).raw()), PlainDword::new(PageLink::of_ida(0x51DC60).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(PageLink::of_ida(0x51DD28).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(PageLink::of_ida(0x51DD40).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(PageLink::of_ida(0x51DD58).raw()), PlainDword::new(0x0), PlainDword::new(0x0), PlainDword::new(0x9)],
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
                Item::new(0x05, 0x0C, 0x009D, PageLink::of_ida(0x51DDF8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x00, 0x00BF, PageLink::of_ida(0x51DE08).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1011, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1012, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1013, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1014, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1015, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1016, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1017, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1018, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
            ]),
            drive_menu_16: ListData18::new(ListHeader::new(0x00, 0x12, 0x009A, 0x2, 0x41425E, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0C, 0x009D, PageLink::of_ida(0x51DDF8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x00, 0x00BF, PageLink::of_ida(0x51DE08).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1011, PageLink::of_ida(0x51DDD8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1012, PageLink::of_ida(0x51DDD8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1013, PageLink::of_ida(0x51DDD8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1014, PageLink::of_ida(0x51DDD8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1015, PageLink::of_ida(0x51DDD8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1016, PageLink::of_ida(0x51DDD8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1017, PageLink::of_ida(0x51DDD8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1018, PageLink::of_ida(0x51DDD8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1019, PageLink::of_ida(0x51DDE8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101A, PageLink::of_ida(0x51DDE8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101B, PageLink::of_ida(0x51DDE8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101C, PageLink::of_ida(0x51DDE8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101D, PageLink::of_ida(0x51DDE8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101E, PageLink::of_ida(0x51DDE8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101F, PageLink::of_ida(0x51DDE8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1020, PageLink::of_ida(0x51DDE8).raw(), 0x0, 0x0),
            ]),
            menu_more: ListData26::new(ListHeader::new(0x00, 0x1A, 0x009A, 0x2, 0x41425E, 0x0, 0x0, 0x0), [
                Item::new(0x05, 0x0C, 0x009D, PageLink::of_ida(0x51DDF8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x00, 0x00BF, PageLink::of_ida(0x51DE08).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1011, PageLink::of_ida(0x51DDA8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1012, PageLink::of_ida(0x51DDA8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1013, PageLink::of_ida(0x51DDA8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1014, PageLink::of_ida(0x51DDA8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1015, PageLink::of_ida(0x51DDA8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1016, PageLink::of_ida(0x51DDA8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1017, PageLink::of_ida(0x51DDA8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1018, PageLink::of_ida(0x51DDA8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1019, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101A, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101B, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101C, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101D, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101E, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x101F, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1020, PageLink::of_ida(0x51DDB8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1021, PageLink::of_ida(0x51DDC8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1022, PageLink::of_ida(0x51DDC8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1023, PageLink::of_ida(0x51DDC8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1024, PageLink::of_ida(0x51DDC8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1025, PageLink::of_ida(0x51DDC8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1026, PageLink::of_ida(0x51DDC8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1027, PageLink::of_ida(0x51DDC8).raw(), 0x0, 0x0),
                Item::new(0x01, 0x01, 0x1028, PageLink::of_ida(0x51DDC8).raw(), 0x0, 0x0),
            ]),
            folder_menu: ListData6::new(ListHeader::new(0x00, 0x06, 0x009A, 0x0, 0x4142F5, 0x0, 0x0, 0x41534E), [
                Item::new(0x05, 0x0C, 0x009E, 0x0, 0x0, 0x0),
                Item::new(0x24, 0x01, 0x0000, 0x0, PageLink::of_ida(0x95EBFA).raw(), PageLink::of_ida(0x51DD70).raw()),
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
            title: Item::new(0x05, 0x09, 0x0000, PageLink::of_ida(0x51DD88).raw(), 0x0, 0x0),
            boxes: [BoxRect::new(50, 150, 750, 44), BoxRect::new(50, 150, 750, 400), BoxRect::new(125, 150, 200, 400), BoxRect::new(325, 150, 200, 400), BoxRect::new(525, 150, 200, 400), BoxRect::new(225, 150, 200, 400), BoxRect::new(425, 150, 200, 400), BoxRect::new(50, 200, 750, 30), BoxRect::new(50, 495, 750, 30)],
    bad_folder_chars: PlainBytes::new(with_runs::<20>(&[(0, b"\"+*,;=/:<>?\\{}[]|\x7f")])),
    _tail: PlainBytes::new([0; 4]),
        }
    }
}

// The layout of the image's bytes: every record sits at its IDA address (the struct's first
// byte is 0x51C400), and the whole is the size the bytes had.
const _: () = {
    use core::mem::offset_of;
    const BASE: usize = 0x51C400;
    assert!(offset_of!(InstallerPages, texts) == 0x51D433 - BASE);
    assert!(offset_of!(InstallerPages, msg_error) == 0x51D453 - BASE);
    assert!(offset_of!(InstallerPages, msg_cancel) == 0x51D463 - BASE);
    assert!(offset_of!(InstallerPages, remove_strings) == 0x51D473 - BASE);
    assert!(offset_of!(InstallerPages, bar_glyphs_list) == 0x51D483 - BASE);
    assert!(offset_of!(InstallerPages, kind_fonts) == 0x51D528 - BASE);
    assert!(offset_of!(InstallerPages, page_table) == 0x51D594 - BASE);
    assert!(offset_of!(InstallerPages, language_menu) == 0x51D670 - BASE);
    assert!(offset_of!(InstallerPages, main_menu) == 0x51D728 - BASE);
    assert!(offset_of!(InstallerPages, mode_menu) == 0x51D790 - BASE);
    assert!(offset_of!(InstallerPages, drive_menu_8) == 0x51D7F8 - BASE);
    assert!(offset_of!(InstallerPages, drive_menu_16) == 0x51D8B0 - BASE);
    assert!(offset_of!(InstallerPages, menu_more) == 0x51D9E8 - BASE);
    assert!(offset_of!(InstallerPages, folder_menu) == 0x51DBA0 - BASE);
    assert!(offset_of!(InstallerPages, room_menu) == 0x51DC18 - BASE);
    assert!(offset_of!(InstallerPages, dxsetup_menu) == 0x51DC60 - BASE);
    assert!(offset_of!(InstallerPages, finish_menu) == 0x51DCB8 - BASE);
    assert!(offset_of!(InstallerPages, action_pages) == 0x51DD10 - BASE);
    assert!(offset_of!(InstallerPages, folder_edit) == 0x51DD70 - BASE);
    assert!(offset_of!(InstallerPages, title) == 0x51DD78 - BASE);
    assert!(offset_of!(InstallerPages, boxes) == 0x51DD88 - BASE);
    assert!(offset_of!(InstallerPages, bad_folder_chars) == 0x51DE18 - BASE);
    assert!(size_of::<InstallerPages>() == 0x1A30);
};

impl InstallerPages {
    /// The byte `index` of `kind_fonts` (8 bytes a font kind); an index past the table ends
    /// the program as every checked installer read does.
    pub fn kind_font(index: u32) -> u8 {
        let install = crate::install::owner::install();
        if index >= 108 {
            crate::install::installer::refuse(crate::errors::BadInput(
                "a font kind past the installer's table",
            ));
        }
        install.pages.kind_fonts.get(index as usize)
    }

    /// Where font `index` of the page's fonts is (the table keeps IDA's addresses as
    /// names). A slot that names no font
    /// object (the unused slot 0, an index off the table) is refused.
    pub fn font_of(index: u32) -> Result<crate::gfx::typefaces::FontId, crate::errors::BadInput> {
        let install = crate::install::owner::install();
        let cell = install
            .pages
            .fonts
            .get(index as usize)
            .ok_or(crate::errors::BadInput("a page font past the table"))?;
        crate::gfx::typefaces::FontId::try_at(cell.get() as usize).ok_or(crate::errors::BadInput(
            "a page font that is no font object",
        ))
    }

    /// 0x51D4C4: `dword_51D4C4`, the page fonts' table.
    #[cfg(test)]
    const FONTS_AT: usize = 0x51D4C4;
    pub const START: usize = 0x51C407;
    pub const LEN: usize = 0x1A25;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&raw const crate::install::owner::INSTALL.get().pages) + 7)
            as usize
    }
}

/// The installer's path buffers (0x100 bytes each; the network's lobby edits two of them) (IDA 0x95E6F8 .. 0x95ECF8, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct InstallerPaths {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: PlainBytes<8>,
    /// 0x95E6F8: `byte_95E6F8`: the folder installed into (also the first of `-i`'s paths).
    pub game_dir: PlainBytes<256>,
    /// 0x95E7F8: `byte_95E7F8`: the folder installed from (also `-i`'s second path).
    pub installed: PlainBytes<256>,
    /// 0x95E8F8: `byte_95E8F8`: a path being built (the lobby's number edited as text).
    pub path_a: PlainBytes<256>,
    /// 0x95E9F8: `byte_95E9F8`: another path being built.
    pub path_b: PlainBytes<256>,
    /// 0x95EAF8: `byte_95EAF8`: the script's path; an edit's backup (the lobby's text before editing).
    pub script_path: PlainBytes<256>,
    /// 0x95EBF8: `byte_95EBF8`: the folder typed on the page.
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

impl InstallerPaths {
    pub const START: usize = 0x95E6F8;
    pub const LEN: usize = 0x600;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&raw const crate::install::owner::INSTALL.get().paths) + 8)
            as usize
    }
}

/// The installer's palette buffer (IDA 0x4F3874 .. 0x4F3B74, the image's layout).
#[repr(C, align(16))]
#[derive(rc2k_derive::Cells)]
pub struct InstallerPalette {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: PlainBytes<4>,
    /// 0x4F3874: `unk_4F3874`: the installer's palette file read here (0x300 bytes, `install::open_installer`).
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

impl InstallerPalette {
    pub const START: usize = 0x4F3874;
    pub const LEN: usize = 0x300;
    #[cfg(test)]
    /// Where the byte IDA calls `START` is (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&raw const crate::install::owner::INSTALL.get().palette) + 4)
            as usize
    }
}

/// The installer pages' dwords that name data: the pages' links, their items' fonts and
/// variables (IDA's addresses, sorted). The code pointers beside them stay addresses.
#[cfg(test)]
pub(super) const PAGE_LINKS: [u32; 105] = [
    0x51D43B, 0x51D44B, 0x51D45B, 0x51D46B, 0x51D47B, 0x51D48F, 0x51D49F, 0x51D594, 0x51D59C,
    0x51D5A0, 0x51D5A4, 0x51D5A8, 0x51D5AC, 0x51D5B0, 0x51D5B4, 0x51D5B8, 0x51D5BC, 0x51D5C0,
    0x51D5C8, 0x51D5CC, 0x51D5D0, 0x51D5D4, 0x51D5D8, 0x51D5E0, 0x51D5E4, 0x51D5E8, 0x51D5EC,
    0x51D5F0, 0x51D5F4, 0x51D5F8, 0x51D5FC, 0x51D600, 0x51D604, 0x51D608, 0x51D60C, 0x51D610,
    0x51D614, 0x51D618, 0x51D61C, 0x51D620, 0x51D62C, 0x51D630, 0x51D634, 0x51D638, 0x51D63C,
    0x51D648, 0x51D654, 0x51D660, 0x51D814, 0x51D824, 0x51D834, 0x51D844, 0x51D854, 0x51D864,
    0x51D874, 0x51D884, 0x51D894, 0x51D8A4, 0x51D8CC, 0x51D8DC, 0x51D8EC, 0x51D8FC, 0x51D90C,
    0x51D91C, 0x51D92C, 0x51D93C, 0x51D94C, 0x51D95C, 0x51D96C, 0x51D97C, 0x51D98C, 0x51D99C,
    0x51D9AC, 0x51D9BC, 0x51D9CC, 0x51D9DC, 0x51DA04, 0x51DA14, 0x51DA24, 0x51DA34, 0x51DA44,
    0x51DA54, 0x51DA64, 0x51DA74, 0x51DA84, 0x51DA94, 0x51DAA4, 0x51DAB4, 0x51DAC4, 0x51DAD4,
    0x51DAE4, 0x51DAF4, 0x51DB04, 0x51DB14, 0x51DB24, 0x51DB34, 0x51DB44, 0x51DB54, 0x51DB64,
    0x51DB74, 0x51DB84, 0x51DB94, 0x51DBD0, 0x51DBD4, 0x51DD7C,
];

#[cfg(test)]
mod tests {
    #[test]
    fn page_links_are_the_images_data_pointers() {
        let found: Vec<u32> = crate::moved::image_relocations()
            .into_iter()
            .filter(|&p| {
                let at = p as usize;
                let fonts =
                    (InstallerPages::FONTS_AT..InstallerPages::FONTS_AT + 100).contains(&at);
                let value = crate::moved::image_bytes(at, 4);
                let value = u32::from_le_bytes(value.try_into().unwrap());
                (InstallerPages::START..InstallerPages::START + InstallerPages::LEN).contains(&at)
                    && !fonts
                    && value >= 0x4DA000
            })
            .collect();
        assert_eq!(found, PAGE_LINKS);
        for &p in &PAGE_LINKS {
            let off = (p as usize - InstallerPages::START) as u32;
            let image = crate::moved::image_bytes(p as usize, 4);
            let ida = u32::from_le_bytes(image.try_into().unwrap());
            let link = PageLink::from_raw(
                crate::pointers::GamePtr::to(&crate::install::owner::install().pages)
                    .offset(off + 7)
                    .read::<u32>("a page link")
                    .unwrap(),
            );
            assert_eq!(link, PageLink::of_ida(ida), "{p:#X}");
            // The handle names the place the image's address is now.
            let home = crate::moved::home_of_rva(ida as usize - crate::framework::IMAGE_BASE);
            if let Ok(place) = link.place() {
                assert_eq!(Some(place.addr() as usize), home, "{p:#X}");
            }
        }
    }

    use super::*;

    #[test]
    fn installer_page_fonts_resolve_as_the_images_do() {
        // Slot 0 is no font; the others name font objects, as the image's dwords do.
        assert!(InstallerPages::font_of(0).is_err());
        assert!(InstallerPages::font_of(25).is_err());
        for n in 1..25 {
            let image = crate::moved::image_bytes(0x51D4C4 + 4 * n, 4);
            let ida = u32::from_le_bytes(image[..4].try_into().unwrap()) as usize;
            let font = crate::gfx::typefaces::FontId::try_at(ida).unwrap();
            assert_eq!(font.ida(), ida);
            assert_eq!(InstallerPages::font_of(n as u32).unwrap(), font);
        }
    }

    #[test]
    fn installer_languages_keys_are_the_images() {
        // Nine keys, a slot's address each (0x51BEEE + 0x80 n), kept as the image's numbers.
        for n in 0..9 {
            let at = 0x51C378 + 0xE * n;
            let image = crate::moved::image_bytes(at, 0xE);
            assert_eq!(&image[..5], format!("lang{}", n + 1).as_bytes());
            let slot = u32::from_le_bytes(image[6..10].try_into().unwrap()) as usize;
            assert_eq!(slot, 0x51BEEE + 0x80 * n);
        }
    }

    #[test]
    fn installerrun_starts_as_the_games() {
        // `list_folder_tree`'s test writes these (under the lock).
        let _lock = crate::store::test_lock();
        let size = size_of::<InstallerRun>();
        assert!(
            8 + InstallerRun::LEN <= size && size <= (8 + InstallerRun::LEN).next_multiple_of(16)
        );
        let image = crate::moved::image_bytes(InstallerRun::START, InstallerRun::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(InstallerRun::home()).cast_const(),
                InstallerRun::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn installernames_starts_as_the_games() {
        // `list_folder_tree`'s test writes these (under the lock).
        let _lock = crate::store::test_lock();
        let size = size_of::<InstallerNames>();
        assert!(InstallerNames::LEN <= size && size <= (InstallerNames::LEN).next_multiple_of(16));
        let image = crate::moved::image_bytes(InstallerNames::START, InstallerNames::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(InstallerNames::home()).cast_const(),
                InstallerNames::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn installerpages_starts_as_the_games() {
        let size = size_of::<InstallerPages>();
        assert!(
            7 + InstallerPages::LEN <= size
                && size <= (7 + InstallerPages::LEN).next_multiple_of(16)
        );
        let mut image = crate::moved::image_bytes(InstallerPages::START, InstallerPages::LEN);
        // The links are the handles that stand for the image's addresses.
        for &p in &PAGE_LINKS {
            let at = p as usize - InstallerPages::START;
            let ida = u32::from_le_bytes(image[at..at + 4].try_into().unwrap());
            image[at..at + 4].copy_from_slice(&PageLink::of_ida(ida).raw().to_le_bytes());
        }
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(InstallerPages::home()).cast_const(),
                InstallerPages::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn installerpaths_starts_as_the_games() {
        let size = size_of::<InstallerPaths>();
        assert!(
            8 + InstallerPaths::LEN <= size
                && size <= (8 + InstallerPaths::LEN).next_multiple_of(16)
        );
        let image = crate::moved::image_bytes(InstallerPaths::START, InstallerPaths::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(InstallerPaths::home()).cast_const(),
                InstallerPaths::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }

    #[test]
    fn installerpalette_starts_as_the_games() {
        let size = size_of::<InstallerPalette>();
        assert!(
            4 + InstallerPalette::LEN <= size
                && size <= (4 + InstallerPalette::LEN).next_multiple_of(16)
        );
        let image = crate::moved::image_bytes(InstallerPalette::START, InstallerPalette::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(InstallerPalette::home()).cast_const(),
                InstallerPalette::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
