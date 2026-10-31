//! The installer's variables, out of RAL.EXE's image (phase 5), typed (phase 7): the flags
//! and counters of its run, the folder and file lists, the registry names and scripts, the
//! language list, and the pages' tables. Split by owner (2026-10-25) from the rest of the
//! image's scattered variables (`rest::low`); the network's and the strings' neighbours are
//! in their own modules.
//!
//! `#[repr(C)]` structs of atomic cells with the image's layout, first values the image's
//! (`tests`); `moved::MOVED` points the image's addresses at them (`home`): the pages'
//! tables hold pointers into each other and into the installer's texts, which are repointed
//! at start (`moved::point_into_homes`). The pages' lists are walked by pointer
//! (`install::installer`'s `rd`/`rb`), so their tables stay byte buffers.

use crate::session::{Byte, Bytes, Dword, Loose, Word, with_runs};

/// The installer's flags, counters and lists (IDA 0x51B9C8 .. 0x51BB60, the image's layout).
#[repr(C, align(16))]
pub struct InstallerRun {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<8>,
    /// 0x51B9C8: `dword_51B9C8`: the installation is this version (`install::mark_main_page`).
    pub same_version: Dword,
    /// 0x51B9CC: `dword_51B9CC`: -u: the update run (`install::install_clio`).
    pub updating: Dword,
    /// 0x51B9D0: `dword_51B9D0`: -i: the paths were given on the command line.
    pub from_command: Dword,
    /// 0x51B9D4: `g_addStartItems`: the shortcuts are added.
    pub add_items: Dword,
    /// 0x51B9D8: `g_removeStartItems`: the shortcuts are removed (`install::uninstall`).
    pub remove_items: Dword,
    /// 0x51B9DC: `g_runDxSetup`: DirectX's set-up is run at the end.
    pub run_dxsetup: Dword,
    /// 0x51B9E0: `g_installing`: -p: installing (`install::check_at_start`).
    pub installing: Dword,
    /// 0x51B9E4: `dword_51B9E4`: the registration is offered.
    pub offer_registration: Dword,
    /// 0x51B9E8: `g_runRegistration`: the registration program is run at the end.
    pub run_registration: Dword,
    /// 0x51B9EC: `a4eregEregipExe`: the registration program's script: a count, "ereg\eregip.exe" from +2 (`install::run_after_exit`).
    pub ereg: Bytes<20>,
    /// 0x51BA00: `g_pathProblem`: 1 a bad folder, 2 no room.
    pub path_problem: Dword,
    /// 0x51BA04: `g_ignoreRoom`: the lack of room is ignored.
    pub ignore_room: Dword,
    /// 0x51BA08: `g_givenMediaMode`: -i's first digit.
    pub given_mode: Dword,
    /// 0x51BA0C: `g_givenLanguage`: -i's second digit.
    pub given_language: Dword,
    /// 0x51BA10: `dword_51BA10`: 26 of (found, free KB), by drive letter.
    pub drives: [Dword; 52],
    /// 0x51BAE0: `g_driveCount`: the drives found.
    pub drive_count: Dword,
    /// 0x51BAE4: `g_firstDrive`: the first letter looked at.
    pub first_drive: Dword,
    /// 0x51BAE8: `g_filesCopied`: the files copied so far.
    pub files_copied: Dword,
    /// 0x51BAEC: `g_filesToCopy`: the files to copy.
    pub files_to_copy: Dword,
    /// 0x51BAF0: `dword_51BAF0`: `install::install_files`' spare.
    pub copy_spare: Dword,
    /// 0x51BAF4: Not named: 4 bytes (a gap, or data nothing reads here).
    pub _gap_51baf4: Bytes<4>,
    /// 0x51BAF8: `g_roomNeededKb`: KB needed.
    pub room_needed: Dword,
    /// 0x51BAFC: `stru_51BAFC`: the installer's memory list (a `MemoryList` of 16 bytes, then its work).
    pub buffers: Bytes<64>,
    /// 0x51BB3C: `g_installBarGlyphs`: the progress bar's glyph table.
    pub bar_glyphs: Dword,
    /// 0x51BB40: `g_dirNames`: the folders' names (a 0x10000-byte buffer; 0 since phase 12:
    /// `install::name_tables::FOLDERS`).
    pub dirs: Dword,
    /// 0x51BB44: `g_fileNames`: the files' names (a 0x40000-byte buffer; 0 since phase 12:
    /// `install::name_tables::FILES`).
    pub files: Dword,
    /// 0x51BB48: `g_dirCount`: the folders listed.
    pub dir_count: Dword,
    /// 0x51BB4C: `g_fileCount`: the files listed.
    pub file_count: Dword,
    /// 0x51BB50: `g_dirScan`: the next folder to search (since phase 12 its number).
    pub dir_scan: Dword,
    /// 0x51BB54: `g_dirCursor`: the folders' table's end (since phase 12 its bytes).
    pub dir_cursor: Dword,
    /// 0x51BB58: `g_fileCursor`: the files' table's end (since phase 12 its bytes).
    pub file_cursor: Dword,
    /// 0x51BB5C: `phkResult`: the open registry key.
    pub reg_key: Dword,
}

pub static INSTALLER_RUN: InstallerRun = InstallerRun {
    _lead: Bytes::new([0; 8]),
    same_version: Dword::new(0x0),
    updating: Dword::new(0x0),
    from_command: Dword::new(0x0),
    add_items: Dword::new(0x0),
    remove_items: Dword::new(0x0),
    run_dxsetup: Dword::new(0x0),
    installing: Dword::new(0x0),
    offer_registration: Dword::new(0x0),
    run_registration: Dword::new(0x0),
    ereg: Bytes::new(with_runs::<20>(&[(0, b";4ereg\\eregip.exe")])),
    path_problem: Dword::new(0x0),
    ignore_room: Dword::new(0x0),
    given_mode: Dword::new(0x2),
    given_language: Dword::new(0x0),
    drives: [const { Dword::new(0x0) }; 52],
    drive_count: Dword::new(0x0),
    first_drive: Dword::new(0x63),
    files_copied: Dword::new(0x0),
    files_to_copy: Dword::new(0x0),
    copy_spare: Dword::new(0x0),
    _gap_51baf4: Bytes::new([0; 4]),
    room_needed: Dword::new(0x0),
    buffers: Bytes::new([0; 64]),
    bar_glyphs: Dword::new(0x0),
    dirs: Dword::new(0x0),
    files: Dword::new(0x0),
    dir_count: Dword::new(0x0),
    file_count: Dword::new(0x0),
    dir_scan: Dword::new(0x0),
    dir_cursor: Dword::new(0x0),
    file_cursor: Dword::new(0x0),
    reg_key: Dword::new(0x0),
};

impl InstallerRun {
    pub const START: usize = 0x51B9C8;
    pub const LEN: usize = 0x198;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&INSTALLER_RUN as *const InstallerRun) + 8) as usize
    }
}

/// The registry names, the installed record, the scripts and file names, the languages (IDA 0x51BB60 .. 0x51C407, the image's layout).
#[repr(C, align(16))]
pub struct InstallerNames {
    /// 0x51BB60: `aSoftwareMagnet`: "SOFTWARE\Magnetic Fields\RC99".
    pub key_name: Bytes<30>,
    /// 0x51BB7E: `aStatus`: "status".
    pub status_name: Bytes<7>,
    /// 0x51BB85: `aSource`: "source".
    pub source_name: Bytes<7>,
    /// 0x51BB8C: `g_installStatus`: the installed status byte (`install::check_installed_version`).
    pub status: Byte,
    /// 0x51BB8D: `g_installSource`: the folder installed from (0xFF bytes used).
    pub source: Bytes<259>,
    /// 0x51BC90: `dword_51BC90`: the size of the source folder's name.
    pub source_size: Dword,
    /// 0x51BC94: `Type`: the registry value's type.
    pub reg_type: Dword,
    /// 0x51BC98: Not named: 176 bytes (a gap, or data nothing reads here).
    pub _gap_51bc98: Bytes<176>,
    /// 0x51BD48: `g_countScript`: the count script (`install::install_files`).
    pub script_count: Bytes<200>,
    /// 0x51BE10: `byte_51BE10`: the uninstall script.
    pub script: Bytes<17>,
    /// 0x51BE21: `byte_51BE21`: a byte of the script (`install::finish_page`).
    pub script_mark: Byte,
    /// 0x51BE22: Not named: 30 bytes (a gap, or data nothing reads here).
    pub _gap_51be22: Bytes<30>,
    /// 0x51BE40: `path`: "ral.exe" (in ".\ral.exe").
    pub program: Bytes<8>,
    /// 0x51BE48: `aRalExe_0`: "ral.exe".
    pub ral_exe: Bytes<8>,
    /// 0x51BE50: Not named: 4 bytes (a gap, or data nothing reads here).
    pub _gap_51be50: Bytes<4>,
    /// 0x51BE54: `aRedistDirectxD`: "redist\directx\dxsetup.exe".
    pub dxsetup: Bytes<27>,
    /// 0x51BE6F: `aFilesReadmeRea`: "files\%readme\readme.txt".
    pub readme: Bytes<25>,
    /// 0x51BE88: `aFilesReadmeMan`: "files\%readme\manual.pdf".
    pub manual: Bytes<25>,
    /// 0x51BEA1: `aFilesDGfxInsta`: "files\d\gfx\install.bun".
    pub installer_data: Bytes<24>,
    /// 0x51BEB9: `aFilesDGfxInsta_0`: "files\d\gfx\install.pal".
    pub install_pal: Bytes<24>,
    /// 0x51BED1: `aFilesDGfxBackd`: "files\d\gfx\backdrop.pcx".
    pub backdrop: Bytes<25>,
    /// 0x51BEEA: `g_languageCount`: the languages found.
    pub language_count: Loose,
    /// 0x51BEEE: `g_languages`: the languages found, 0x80 bytes each (eight).
    pub languages: Bytes<1024>,
    /// 0x51C2EE: Not named: 128 bytes (a gap, or data nothing reads here).
    pub _gap_51c2ee: Bytes<128>,
    /// 0x51C36E: `aLangCfg`: "lang.cfg".
    pub lang_cfg: Bytes<10>,
    /// 0x51C378: Not named: 127 bytes (a gap, or data nothing reads here).
    pub _gap_51c378: Bytes<127>,
    /// 0x51C3F7: `g_roomByMode`: KB needed by installation mode.
    pub room_by_mode: Bytes<16>,
}

pub static INSTALLER_NAMES: InstallerNames = InstallerNames {
    key_name: Bytes::new(with_runs::<30>(&[(0, b"SOFTWARE\\Magnetic Fields\\RC99")])),
    status_name: Bytes::new(with_runs::<7>(&[(0, b"status")])),
    source_name: Bytes::new(with_runs::<7>(&[(0, b"source")])),
    status: Byte::new(0x0),
    source: Bytes::new([0; 259]),
    source_size: Dword::new(0x0),
    reg_type: Dword::new(0x0),
    _gap_51bc98: Bytes::new(with_runs::<176>(&[(20, b"\x18"), (44, b"PROGMAN\x00[CreateGroup(Rally Championship)]\x00[DeleteGroup(Rally Championship)]\x00[AddItem(\x00,Rally)]\x00,Uninstall Rally)]\x00,View Readme)]")])),
    script_count: Bytes::new(with_runs::<200>(&[(0, b"\x03ral.exe\x03*.txt\x03*.pdf\x08\x02\x01files\x04a\x04b\x04d\x08\x03\x04fmv\t\x01g\x04frnt\x04graphics\x08\x03\x04slides\t\x02\x04levels\x04%locale\x04m\x04maps\x08\x03\x04music\t\x04replay\x04s\x01%speech\x04m\x04f\x08\x03\x04cars\x04rally\t\x02\x02\t\x01savedata\x01carsetup\x02\x01replay\x02\x01savegame\x02\x01snapshot\x02\x02\x01var\x03ral.cfg\x02\x06\x05")])),
    script: Bytes::new(with_runs::<17>(&[(0, b"\x03*.*\x01files\x02\x04files")])),
    script_mark: Byte::new(0x1),
    _gap_51be22: Bytes::new(with_runs::<30>(&[(0, b"savedata\x02\x04savedata\x01var\x02\x04var\x05.\\")])),
    program: Bytes::new(with_runs::<8>(&[(0, b"ral.exe")])),
    ral_exe: Bytes::new(with_runs::<8>(&[(0, b"ral.exe")])),
    _gap_51be50: Bytes::new(with_runs::<4>(&[(0, b" -u")])),
    dxsetup: Bytes::new(with_runs::<27>(&[(0, b"redist\\directx\\dxsetup.exe")])),
    readme: Bytes::new(with_runs::<25>(&[(0, b"files\\%readme\\readme.txt")])),
    manual: Bytes::new(with_runs::<25>(&[(0, b"files\\%readme\\manual.pdf")])),
    installer_data: Bytes::new(with_runs::<24>(&[(0, b"files\\d\\gfx\\install.bun")])),
    install_pal: Bytes::new(with_runs::<24>(&[(0, b"files\\d\\gfx\\install.pal")])),
    backdrop: Bytes::new(with_runs::<25>(&[(0, b"files\\d\\gfx\\backdrop.pcx")])),
    language_count: Loose::new(0x0),
    languages: Bytes::new([0; 1024]),
    _gap_51c2ee: Bytes::new([0; 128]),
    lang_cfg: Bytes::new(with_runs::<10>(&[(0, b"lang.cfg")])),
    _gap_51c378: Bytes::new(with_runs::<127>(&[(0, b"lang1\x00\xee\xbeQ\x00\x7f\x00\x00\x00lang2\x00n\xbfQ\x00\x7f\x00\x00\x00lang3\x00\xee\xbfQ\x00\x7f\x00\x00\x00lang4\x00n\xc0Q\x00\x7f\x00\x00\x00lang5\x00\xee\xc0Q\x00\x7f\x00\x00\x00lang6\x00n\xc1Q\x00\x7f\x00\x00\x00lang7\x00\xee\xc1Q\x00\x7f\x00\x00\x00lang8\x00n\xc2Q\x00\x7f\x00\x00\x00lang9\x00\xee\xc2Q\x00\x7f")])),
    room_by_mode: Bytes::new(with_runs::<16>(&[(5, b"("), (10, b"\x05\x00\x00(\n")])),
};

impl InstallerNames {
    /// Whether IDA's `ida` is a dword of the languages' keys table (`moved::point_into_homes`
    /// passes it over). The table (0x51C378: `lang1` .. `lang9`, a slot's address each) is
    /// read by nothing: `install::read_languages` has the keys and the slots as constants.
    pub fn holds_handle(ida: usize) -> bool {
        (0x51C378..0x51C378 + 127).contains(&ida)
    }

    pub const START: usize = 0x51BB60;
    pub const LEN: usize = 0x8A7;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        crate::pointers::game_address(&INSTALLER_NAMES as *const InstallerNames) as usize
    }
}

/// The page: its pieces, the list menu's state, the fonts and the pages' tables (IDA 0x51C407 .. 0x51DE2C, the image's layout).
#[repr(C, align(16))]
pub struct InstallerPages {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<7>,
    /// 0x51C407: `g_pageLayout`: the page's pieces: a count, then 0x30 bytes each.
    pub layout: Bytes<4096>,
    /// 0x51D407: `g_listChoice`: the list's choice.
    pub choice: Loose,
    /// 0x51D40B: `g_listChoiceBefore`: the choice before the key.
    pub choice_before: Loose,
    /// 0x51D40F: `g_listRepeats`: how many repeats of the key.
    pub repeats: Loose,
    /// 0x51D413: `g_listLastKey`: the last key (`race::viewer` reads it too).
    pub last_key: Byte,
    /// 0x51D414: `byte_51D414`: a key was taken.
    pub key_taken: Byte,
    /// 0x51D415: `byte_51D415`: the list's answer.
    pub list_answer: Byte,
    /// 0x51D416: `g_listLeave`: the list is left.
    pub list_leave: Byte,
    /// 0x51D417: Not named: 4 bytes (a gap, or data nothing reads here).
    pub _gap_51d417: Bytes<4>,
    /// 0x51D41B: `dword_51D41B`: the key's clock.
    pub key_clock: Loose,
    /// 0x51D41F: `dword_51D41F`: the frame's clock.
    pub frame_clock: Loose,
    /// 0x51D423: `dword_51D423`: the move's clock.
    pub move_clock: Loose,
    /// 0x51D427: Not named: 12 bytes (a gap, or data nothing reads here).
    pub _gap_51d427: Bytes<12>,
    /// 0x51D433: `unk_51D433`: the copying page's two texts.
    pub texts: Bytes<32>,
    /// 0x51D453: `unk_51D453`: the error message.
    pub msg_error: Bytes<16>,
    /// 0x51D463: `unk_51D463`: the cancel message.
    pub msg_cancel: Bytes<16>,
    /// 0x51D473: `unk_51D473`: the removal page's strings.
    pub remove_strings: Bytes<16>,
    /// 0x51D483: `unk_51D483`: the bar's glyph list.
    pub bar_glyphs_list: Bytes<32>,
    /// 0x51D4A3: `dword_51D4A3`: the bar's x, y, width, height.
    pub bar: Bytes<16>,
    /// 0x51D4B3: `unk_51D4B3`: its frame.
    pub frame: Bytes<17>,
    /// 0x51D4C4: `dword_51D4C4`: the page's fonts (pointers).
    pub fonts: [Dword; 25],
    /// 0x51D528: `byte_51D528`: font by mark, 8 bytes a kind (`install::lay_out_pieces`, from +4).
    pub kind_fonts: Bytes<108>,
    /// 0x51D594: `off_51D594`: the pages of a normal run.
    pub normal_pages: Bytes<44>,
    /// 0x51D5C0: Not named: 24 bytes (a gap, or data nothing reads here).
    pub _gap_51d5c0: Bytes<24>,
    /// 0x51D5D8: `off_51D5D8`: the pages of an update run.
    pub update_pages: Bytes<12>,
    /// 0x51D5E4: Not named: 20 bytes (a gap, or data nothing reads here).
    pub _gap_51d5e4: Bytes<20>,
    /// 0x51D5F8: `g_driveMenu`: the drive page's menu.
    pub chosen_menu: Bytes<12>,
    /// 0x51D604: Not named: 292 bytes (a gap, or data nothing reads here).
    pub _gap_51d604: Bytes<292>,
    /// 0x51D728: `off_51D728`: the script's entry (`install::check_at_start`).
    pub script_entry: Dword,
    /// 0x51D72C: `g_readInstalledRecord`: the installed record was read.
    pub record_flag: Dword,
    /// 0x51D730: Not named: 16 bytes (a gap, or data nothing reads here).
    pub _gap_51d730: Bytes<16>,
    /// 0x51D740: `off_51D740`+0: the script entry's first byte (5; `install::check_at_start::set_script_entry`).
    pub entry_kind: Byte,
    /// 0x51D741: the script entry's second byte (0x0C).
    pub entry_marks: Byte,
    /// 0x51D742: the script entry's text id.
    pub entry_text: Word,
    /// 0x51D744: Not named: 14 bytes (a gap, or data nothing reads here).
    pub _gap_51d744: Bytes<14>,
    /// 0x51D752: `word_51D752`: 0xBD when the installer's script is set up (`system::main_loop`).
    pub script_set_up: Word,
    /// 0x51D754: Not named: 164 bytes (a gap, or data nothing reads here).
    pub _gap_51d754: Bytes<164>,
    /// 0x51D7F8: `off_51D7F8`: the 8-drive list menu.
    pub menu_8: Bytes<184>,
    /// 0x51D8B0: `off_51D8B0`: the 16-drive list menu.
    pub menu_16: Bytes<312>,
    /// 0x51D9E8: `off_51D9E8`: the longer list menu.
    pub menu_more: Bytes<440>,
    /// 0x51DBA0: Not named: 424 bytes (a gap, or data nothing reads here).
    pub _gap_51dba0: Bytes<424>,
    /// 0x51DD48: `off_51DD48`: the third page's action.
    pub step: Bytes<16>,
    /// 0x51DD58: Not named: 32 bytes (a gap, or data nothing reads here).
    pub _gap_51dd58: Bytes<32>,
    /// 0x51DD78: `unk_51DD78`: the title's item.
    pub title: Bytes<16>,
    /// 0x51DD88: Not named: 16 bytes (a gap, or data nothing reads here).
    pub _gap_51dd88: Bytes<16>,
    /// 0x51DD98: `unk_51DD98`: the title's box.
    pub title_box: Bytes<16>,
    /// 0x51DDA8: Not named: 112 bytes (a gap, or data nothing reads here).
    pub _gap_51dda8: Bytes<112>,
    /// 0x51DE18: `g_badFolderChars`: the characters a folder name may not hold.
    pub bad_folder_chars: Bytes<20>,
}

pub static INSTALLER_PAGES: InstallerPages = InstallerPages {
    _lead: Bytes::new([0; 7]),
    layout: Bytes::new([0; 4096]),
    choice: Loose::new(0x0),
    choice_before: Loose::new(0x0),
    repeats: Loose::new(0x1),
    last_key: Byte::new(0x0),
    key_taken: Byte::new(0x0),
    list_answer: Byte::new(0x0),
    list_leave: Byte::new(0x0),
    _gap_51d417: Bytes::new([0; 4]),
    key_clock: Loose::new(0x0),
    frame_clock: Loose::new(0x0),
    move_clock: Loose::new(0x0),
    _gap_51d427: Bytes::new([0; 12]),
    texts: Bytes::new(with_runs::<32>(&[(0, b"\xa9\x01\x00\x00\xaf\x00\x00\x00\xa4\xe8Q\x00\xaf\x00\x00\x00\xa9\x01\x00\x00\xfa\x00\x00\x00d\xe9Q\x00\xb0")])),
    msg_error: Bytes::new(with_runs::<16>(&[(0, b"\xa9\x01\x00\x00^\x01\x00\x00\xa4\xe8Q\x00\xb1")])),
    msg_cancel: Bytes::new(with_runs::<16>(&[(0, b"\xa9\x01\x00\x00\x90\x01\x00\x00\xa4\xe8Q\x00\xb3")])),
    remove_strings: Bytes::new(with_runs::<16>(&[(0, b"\xa9\x01\x00\x00,\x01\x00\x00\xa4\xe8Q\x00\xba")])),
    bar_glyphs_list: Bytes::new(with_runs::<32>(&[(0, b"\xe1\x00\x00\x00,\x01"), (12, b"t;O\x00\xa9\x01\x00\x00,\x01\x00\x00\x01\x00\x00\x00t;O")])),
    bar: Bytes::new(with_runs::<16>(&[(0, b"\xe0\x00\x00\x00+\x01\x00\x00\x92\x01\x00\x00\x1f")])),
    frame: Bytes::new(with_runs::<17>(&[(0, b"\xdf\x00\x00\x00*\x01\x00\x00\x94\x01\x00\x00!")])),
    fonts: [Dword::new(0x0), Dword::new(0x51e624), Dword::new(0x51e824), Dword::new(0x51e824), Dword::new(0x51ea44), Dword::new(0x51ea64), Dword::new(0x51ea24), Dword::new(0x51e9e4), Dword::new(0x51ea04), Dword::new(0x51e9c4), Dword::new(0x51e864), Dword::new(0x51e884), Dword::new(0x51e844), Dword::new(0x51e6c4), Dword::new(0x51e6e4), Dword::new(0x51e6a4), Dword::new(0x51e8c4), Dword::new(0x51e8e4), Dword::new(0x51e8a4), Dword::new(0x51e8c4), Dword::new(0x51e8e4), Dword::new(0x51e8a4), Dword::new(0x51e984), Dword::new(0x51e9a4), Dword::new(0x51e964)],
    kind_fonts: Bytes::new(with_runs::<108>(&[(0, b"\x12\x00\t\x0c\x00\x02\x00\x1e\x17\x00\x08\x0b\x01\x02\x00\x1e\n\x00\x07\x10\x02\x02\x00\x1e"), (32, b"\x12\x00\t\x0c\x00\x02\x00\x1e"), (64, b"\x0c\x00\t\x12\x00\x02\x00\x1e\x15\x00\x15\x15\x00\x03\x00,\x0c\x00\x0c\x0c\x00\x02\x00\x1e\x12\x00\x12\x12\x00\x02\x00\x1e\x18\x00\x18\x18\x00\x02\x00\x1e\t")])),
    normal_pages: Bytes::new(with_runs::<44>(&[(0, b"p\xd6Q"), (8, b"\xc0\xd5Q\x00\xc0\xd5Q\x00\xc0\xd5Q\x00\xc0\xd5Q\x00\xc0\xd5Q\x00\xc0\xd5Q\x00\xc0\xd5Q\x00\xc0\xd5Q\x00\xc0\xd5Q")])),
    _gap_51d5c0: Bytes::new(with_runs::<24>(&[(0, b"(\xd7Q"), (8, b"\xe4\xd5Q\x00T\xd6Q\x00\x10\xd6Q\x00`\xd6Q")])),
    update_pages: Bytes::new(with_runs::<12>(&[(0, b"\xb8\xdcQ"), (8, b"H\xd6Q")])),
    _gap_51d5e4: Bytes::new(with_runs::<20>(&[(0, b"\x90\xd7Q\x00\xc0\xd5Q\x00\xf8\xd5Q\x00\xf8\xd5Q\x00\xf8\xd5Q")])),
    chosen_menu: Bytes::new(with_runs::<12>(&[(0, b"\xf8\xd7Q\x00\xe4\xd5Q\x00\x04\xd6Q")])),
    _gap_51d604: Bytes::new(with_runs::<292>(&[(0, b"\xa0\xdbQ\x00\xf8\xd5Q\x00\x1c\xd6Q\x00\xb8\xdcQ\x00\xc0\xd5Q\x00H\xd6Q\x00\x10\xddQ\x00\x04\xd6Q"), (40, b"0\xd6Q\x00\x18\xdcQ\x00\xf8\xd5Q\x00\x1c\xd6Q\x00`\xdcQ"), (68, b"(\xddQ"), (80, b"@\xddQ"), (92, b"X\xddQ"), (104, b"\t"), (109, b"\n\x9b"), (116, b"\x99AA"), (132, b"\x01\x01+\x10"), (148, b"\x01\x02,\x10"), (164, b"\x01\x03-\x10"), (180, b"\x01\x04.\x10"), (196, b"\x01\x05/\x10"), (212, b"\x01\x060\x10"), (228, b"\x01\x071\x10"), (244, b"\x01\x082\x10"), (260, b"\x01\t3\x10"), (276, b"\x01\x00\x99")])),
    script_entry: Dword::new(0x950500),
    record_flag: Dword::new(0x0),
    _gap_51d730: Bytes::new(with_runs::<16>(&[(0, b"\xefAA")])),
    entry_kind: Byte::new(0x1),
    entry_marks: Byte::new(0x1),
    entry_text: Word::new(0x96),
    _gap_51d744: Bytes::new(with_runs::<14>(&[(12, b"\x01\x02")])),
    script_set_up: Word::new(0x97),
    _gap_51d754: Bytes::new(with_runs::<164>(&[(12, b"\x01\x03\x98"), (28, b"\x01\x04|\x05"), (44, b"\x01\x00\x99"), (61, b"\x05\x9a\x00\x02\x00\x00\x00+BA"), (84, b"\x05\x0c\xa1"), (96, b"\x04SA\x00\x01\x01\xa2"), (116, b"\x01\x02\xa3"), (132, b"\x01\x03\xa4"), (148, b"\x01\x00\xbf")])),
    menu_8: Bytes::new(with_runs::<184>(&[(1, b"\n\x9a\x00\x02\x00\x00\x00^BA"), (24, b"\x05\x0c\x9d\x00\xf8\xddQ"), (40, b"\x01\x00\xbf\x00\x08\xdeQ"), (56, b"\x01\x01\x11\x10\xb8\xddQ"), (72, b"\x01\x01\x12\x10\xb8\xddQ"), (88, b"\x01\x01\x13\x10\xb8\xddQ"), (104, b"\x01\x01\x14\x10\xb8\xddQ"), (120, b"\x01\x01\x15\x10\xb8\xddQ"), (136, b"\x01\x01\x16\x10\xb8\xddQ"), (152, b"\x01\x01\x17\x10\xb8\xddQ"), (168, b"\x01\x01\x18\x10\xb8\xddQ")])),
    menu_16: Bytes::new(with_runs::<312>(&[(1, b"\x12\x9a\x00\x02\x00\x00\x00^BA"), (24, b"\x05\x0c\x9d\x00\xf8\xddQ"), (40, b"\x01\x00\xbf\x00\x08\xdeQ"), (56, b"\x01\x01\x11\x10\xd8\xddQ"), (72, b"\x01\x01\x12\x10\xd8\xddQ"), (88, b"\x01\x01\x13\x10\xd8\xddQ"), (104, b"\x01\x01\x14\x10\xd8\xddQ"), (120, b"\x01\x01\x15\x10\xd8\xddQ"), (136, b"\x01\x01\x16\x10\xd8\xddQ"), (152, b"\x01\x01\x17\x10\xd8\xddQ"), (168, b"\x01\x01\x18\x10\xd8\xddQ"), (184, b"\x01\x01\x19\x10\xe8\xddQ"), (200, b"\x01\x01\x1a\x10\xe8\xddQ"), (216, b"\x01\x01\x1b\x10\xe8\xddQ"), (232, b"\x01\x01\x1c\x10\xe8\xddQ"), (248, b"\x01\x01\x1d\x10\xe8\xddQ"), (264, b"\x01\x01\x1e\x10\xe8\xddQ"), (280, b"\x01\x01\x1f\x10\xe8\xddQ"), (296, b"\x01\x01 \x10\xe8\xddQ")])),
    menu_more: Bytes::new(with_runs::<440>(&[(1, b"\x1a\x9a\x00\x02\x00\x00\x00^BA"), (24, b"\x05\x0c\x9d\x00\xf8\xddQ"), (40, b"\x01\x00\xbf\x00\x08\xdeQ"), (56, b"\x01\x01\x11\x10\xa8\xddQ"), (72, b"\x01\x01\x12\x10\xa8\xddQ"), (88, b"\x01\x01\x13\x10\xa8\xddQ"), (104, b"\x01\x01\x14\x10\xa8\xddQ"), (120, b"\x01\x01\x15\x10\xa8\xddQ"), (136, b"\x01\x01\x16\x10\xa8\xddQ"), (152, b"\x01\x01\x17\x10\xa8\xddQ"), (168, b"\x01\x01\x18\x10\xa8\xddQ"), (184, b"\x01\x01\x19\x10\xb8\xddQ"), (200, b"\x01\x01\x1a\x10\xb8\xddQ"), (216, b"\x01\x01\x1b\x10\xb8\xddQ"), (232, b"\x01\x01\x1c\x10\xb8\xddQ"), (248, b"\x01\x01\x1d\x10\xb8\xddQ"), (264, b"\x01\x01\x1e\x10\xb8\xddQ"), (280, b"\x01\x01\x1f\x10\xb8\xddQ"), (296, b"\x01\x01 \x10\xb8\xddQ"), (312, b"\x01\x01!\x10\xc8\xddQ"), (328, b"\x01\x01\"\x10\xc8\xddQ"), (344, b"\x01\x01#\x10\xc8\xddQ"), (360, b"\x01\x01$\x10\xc8\xddQ"), (376, b"\x01\x01%\x10\xc8\xddQ"), (392, b"\x01\x01&\x10\xc8\xddQ"), (408, b"\x01\x01'\x10\xc8\xddQ"), (424, b"\x01\x01(\x10\xc8\xddQ")])),
    _gap_51dba0: Bytes::new(with_runs::<424>(&[(1, b"\x06\x9a"), (8, b"\xf5BA"), (20, b"NSA\x00\x05\x0c\x9e"), (40, b"$\x01"), (48, b"\xfa\xeb\x95\x00p\xddQ\x00\x05\n\xaa"), (72, b"\x05\n\xac"), (88, b"\x05\n\xad"), (104, b"\x05\n\xab"), (121, b"\x03\x9a"), (128, b"\xc7BA"), (144, b"\x05\n\xad"), (156, b"\x04SA\x00\x01\x01\xae"), (168, b"\xdaBA"), (176, b"\x01\x00\xbf"), (193, b"\x04\x9a"), (200, b"/CA"), (216, b"\x05\x0b\xb2"), (228, b"\x04SA\x00\x05\x0c\xb4"), (244, b"\x04SA\x00\x01\x01l\x05"), (264, b"\x01\x00k\x05"), (281, b"\x04\xb6\x00\x02\x00\x00\x00GCA"), (304, b"\x05\n\xb7"), (316, b"\x04SA\x00\x01\x01\xb8"), (336, b"\x01\x02\xb9"), (352, b"\x01\x00\xbf"), (368, b"@"), (376, b"\xcd0A"), (392, b"@"), (400, b"\xdf4A"), (416, b"@")])),
    step: Bytes::new(with_runs::<16>(&[(0, b")6A")])),
    _gap_51dd58: Bytes::new(with_runs::<32>(&[(0, b"@"), (8, b"P6A"), (24, b"\x02|\xa0\x00\x0b\x00\xe9\x02")])),
    title: Bytes::new(with_runs::<16>(&[(0, b"\x05\t\x00\x00\x88\xddQ")])),
    _gap_51dd88: Bytes::new(with_runs::<16>(&[(0, b"2\x00\x00\x00\x96\x00\x00\x00\xee\x02\x00\x00,")])),
    title_box: Bytes::new(with_runs::<16>(&[(0, b"2\x00\x00\x00\x96\x00\x00\x00\xee\x02\x00\x00\x90\x01")])),
    _gap_51dda8: Bytes::new(with_runs::<112>(&[(0, b"}\x00\x00\x00\x96\x00\x00\x00\xc8\x00\x00\x00\x90\x01\x00\x00E\x01\x00\x00\x96\x00\x00\x00\xc8\x00\x00\x00\x90\x01\x00\x00\r\x02\x00\x00\x96\x00\x00\x00\xc8\x00\x00\x00\x90\x01\x00\x00\xe1\x00\x00\x00\x96\x00\x00\x00\xc8\x00\x00\x00\x90\x01\x00\x00\xa9\x01\x00\x00\x96\x00\x00\x00\xc8\x00\x00\x00\x90\x01\x00\x002\x00\x00\x00\xc8\x00\x00\x00\xee\x02\x00\x00\x1e\x00\x00\x002\x00\x00\x00\xef\x01\x00\x00\xee\x02\x00\x00\x1e")])),
    bad_folder_chars: Bytes::new(with_runs::<20>(&[(0, b"\"+*,;=/:<>?\\{}[]|\x7f")])),
};

impl InstallerPages {
    /// Where font `index` of the page's fonts is (phase 13: the table keeps IDA's addresses
    /// as names, `moved::point_into_homes` passes them over). A slot that names no font
    /// object (the unused slot 0, an index off the table) is refused.
    pub fn font_of(index: u32) -> Result<u32, crate::errors::BadInput> {
        let cell = INSTALLER_PAGES
            .fonts
            .get(index as usize)
            .ok_or(crate::errors::BadInput("a page font past the table"))?;
        crate::gfx::typefaces::FontId::try_at(cell.get() as usize)
            .map(crate::gfx::typefaces::FontId::addr)
            .ok_or(crate::errors::BadInput(
                "a page font that is no font object",
            ))
    }

    /// Whether IDA's `ida` is one of the fonts table's dwords (`moved::point_into_homes`).
    pub fn holds_handle(ida: usize) -> bool {
        (Self::FONTS_AT..Self::FONTS_AT + 25 * 4).contains(&ida)
            || PAGE_LINKS.binary_search(&(ida as u32)).is_ok()
    }

    /// What dword `value` read at game address `at` means: a slot of `PAGE_LINKS` keeps IDA's
    /// address as a name, resolved here to the place now (`installer::rd` reads them all).
    pub fn resolved(at: u32, value: u32) -> Result<u32, crate::errors::BadInput> {
        let off = (at as usize).wrapping_sub(Self::home());
        if value == 0 || off >= Self::LEN {
            return Ok(value);
        }
        if PAGE_LINKS
            .binary_search(&((Self::START + off) as u32))
            .is_err()
        {
            return Ok(value);
        }
        crate::moved::home_of_rva((value as usize).wrapping_sub(crate::framework::IMAGE_BASE))
            .map(|home| home as u32)
            .ok_or(crate::errors::BadInput("an installer link at no place"))
    }

    /// 0x51D4C4: `dword_51D4C4`, the page fonts' table.
    const FONTS_AT: usize = 0x51D4C4;
    pub const START: usize = 0x51C407;
    pub const LEN: usize = 0x1A25;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&INSTALLER_PAGES as *const InstallerPages) + 7) as usize
    }
}

/// The installer's path buffers (0x100 bytes each; the network's lobby edits two of them) (IDA 0x95E6F8 .. 0x95ECF8, the image's layout).
#[repr(C, align(16))]
pub struct InstallerPaths {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<8>,
    /// 0x95E6F8: `byte_95E6F8`: the folder installed into (also the first of `-i`'s paths).
    pub game_dir: Bytes<256>,
    /// 0x95E7F8: `byte_95E7F8`: the folder installed from (also `-i`'s second path).
    pub installed: Bytes<256>,
    /// 0x95E8F8: `byte_95E8F8`: a path being built (the lobby's number edited as text).
    pub path_a: Bytes<256>,
    /// 0x95E9F8: `byte_95E9F8`: another path being built.
    pub path_b: Bytes<256>,
    /// 0x95EAF8: `byte_95EAF8`: the script's path; an edit's backup (the lobby's text before editing).
    pub script_path: Bytes<256>,
    /// 0x95EBF8: `byte_95EBF8`: the folder typed on the page.
    pub typed: Bytes<256>,
}

pub static INSTALLER_PATHS: InstallerPaths = InstallerPaths {
    _lead: Bytes::new([0; 8]),
    game_dir: Bytes::new([0; 256]),
    installed: Bytes::new([0; 256]),
    path_a: Bytes::new([0; 256]),
    path_b: Bytes::new([0; 256]),
    script_path: Bytes::new([0; 256]),
    typed: Bytes::new([0; 256]),
};

impl InstallerPaths {
    pub const START: usize = 0x95E6F8;
    pub const LEN: usize = 0x600;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&INSTALLER_PATHS as *const InstallerPaths) + 8) as usize
    }
}

/// The installer's palette buffer (IDA 0x4F3874 .. 0x4F3B74, the image's layout).
#[repr(C, align(16))]
pub struct InstallerPalette {
    /// Not in the image: the bytes that keep every field the alignment its IDA address has (the struct is 16-byte aligned).
    pub _lead: Bytes<4>,
    /// 0x4F3874: `unk_4F3874`: the installer's palette file read here (0x300 bytes, `install::open_installer`).
    pub palette: Bytes<768>,
}

pub static INSTALLER_PALETTE: InstallerPalette = InstallerPalette {
    _lead: Bytes::new([0; 4]),
    palette: Bytes::new([0; 768]),
};

impl InstallerPalette {
    pub const START: usize = 0x4F3874;
    pub const LEN: usize = 0x300;
    /// Where the byte IDA calls `START` is now (`moved::MOVED`'s home).
    pub fn home() -> usize {
        (crate::pointers::game_address(&INSTALLER_PALETTE as *const InstallerPalette) + 4) as usize
    }
}

/// The installer pages' dwords that name data: the pages' links, their items' fonts and
/// variables (IDA's addresses, sorted). The code pointers beside them stay addresses.
const PAGE_LINKS: [u32; 105] = [
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
        let found: Vec<u32> = crate::relocations::PLACES
            .iter()
            .copied()
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
            let at = (InstallerPages::home() + p as usize - InstallerPages::START) as u32;
            let value = crate::install::installer::rd(at);
            assert_ne!(value, 0, "{p:#X}");
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
            assert_eq!(InstallerPages::font_of(n as u32).unwrap(), font.addr());
        }
    }

    #[test]
    fn installer_languages_keys_are_the_images() {
        // Nine keys, a slot's address each (0x51BEEE + 0x80 n), kept as the image's numbers.
        assert!(InstallerNames::holds_handle(0x51C37E) && InstallerNames::holds_handle(0x51C3EE));
        assert!(!InstallerNames::holds_handle(0x51C377) && !InstallerNames::holds_handle(0x51C3F7));
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
                crate::pointers::host_at(InstallerRun::home()) as *const u8,
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
                crate::pointers::host_at(InstallerNames::home()) as *const u8,
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
        let image = crate::moved::image_bytes(InstallerPages::START, InstallerPages::LEN);
        let mine = unsafe {
            std::slice::from_raw_parts(
                crate::pointers::host_at(InstallerPages::home()) as *const u8,
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
                crate::pointers::host_at(InstallerPaths::home()) as *const u8,
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
                crate::pointers::host_at(InstallerPalette::home()) as *const u8,
                InstallerPalette::LEN,
            )
        };
        assert_eq!(mine, &image[..]);
    }
}
