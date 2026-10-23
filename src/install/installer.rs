//! The installer that `RAL.EXE` also is (`sub_412DE0` and its tree, read 2026-10-19): the
//! variables its parts share, the calls they make (`Part`, by the callee's address), the
//! Win32 calls (`Api`), and the tools of their self-tests.
//!
//! The installer runs only with its script set up (`-i`, `-u`, `-p`, or the update and Clio
//! discs: `install::check_at_start`, `check_clio_disc`). Its pages are list menus
//! (`sub_4144CF`): a list of 0x18 bytes (+0 flags, +1 items, +2 title, +4 the item chosen,
//! +8 the list's action with ebx 0 at the start, 1 after a key, -1 at the end, +0x10 the
//! neighbours for `menu::list_step`, +0x14 a key action) and its items of 0x10 bytes from
//! +0x18 (+0 kind and marks, +1 the next list or the choices, +2 the text, +8 and +0xC the
//! kind's pointers). `sub_4144AA` goes from list to list by the answer.
//!
//! The Rust versions call everything with effects through `part` (the IDA address of the
//! callee, or for a list's action the address it holds), so that a self-test can stand in
//! for it; pure text work (`str_append` and the like) is done in place. In the game,
//! `game_part` gives each call its Rust version (or, for the platform and a few drawing
//! functions, the game's hooked function through `call_game`). The Win32 calls go through
//! `api` (`game_api` in the game).
//!
//! The self-tests: a copy of the original whose calls with effects go to noting stand-ins
//! (`Copy`), and jumps to the fatal stubs to landings that note the code; `both` runs it
//! and the Rust version (with `test_part`) from the same state and compares the registers,
//! the flags, the calls (place, registers, and a text a call gets) and memory.

use crate::errors::{FatalError, Refused};
use crate::framework::{Regs, ZERO_FLAG, game_addr, rva};

/// A call by place: the callee's address in the game (or a stand-in's in a test).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;
/// A Win32 call by the address of its import (`jmp` thunk), the arguments in, eax out.
pub type Api<'a> = &'a mut dyn FnMut(u32, &[u32]) -> u32;

pub const ZF: u32 = ZERO_FLAG;

// ---- the variables (IDA addresses) ----

pub const GAME_DIR: usize = 0x95E6F8; // byte_95E6F8: the folder installed into, 0x100
pub const INSTALLED: usize = 0x95E7F8; // byte_95E7F8: the folder installed from, 0x100
pub const PATH_A: usize = 0x95E8F8; // byte_95E8F8: a path being built, 0x100
pub const PATH_B: usize = 0x95E9F8; // byte_95E9F8: another, 0x100
pub const SCRIPT_PATH: usize = 0x95EAF8; // byte_95EAF8: the script's path; an edit's backup
pub const TYPED: usize = 0x95EBF8; // byte_95EBF8: the folder typed on the page, 0x100
pub const BASE_B: usize = 0x5295B4; // unk_5295B4: base directory B
// g_baseDirectory
pub const LOCALE: usize = 0x518AA8; // g_localeDirectory
pub const MEDIA_MODE: usize = 0x5189A4; // g_mediaMode
pub const STATUS: usize = 0x51BB8C; // g_installStatus
pub const SOURCE: usize = 0x51BB8D; // g_installSource, 0xFF bytes
pub const SOURCE_SIZE: usize = 0x51BC90; // dword_51BC90
pub const REG_KEY: usize = 0x51BB5C; // phkResult
pub const SAME_VERSION: usize = 0x51B9C8; // dword_51B9C8: the installation is this version
// dword_51B9CC (-u)
pub const FROM_COMMAND: usize = 0x51B9D0; // dword_51B9D0 (-i: the paths given)
pub const ADD_ITEMS: usize = 0x51B9D4; // g_addStartItems
// g_removeStartItems
pub const RUN_DXSETUP: usize = 0x51B9DC; // g_runDxSetup
pub const INSTALLING: usize = 0x51B9E0; // g_installing (-p)
pub const OFFER_REGISTRATION: usize = 0x51B9E4; // dword_51B9E4
pub const RUN_REGISTRATION: usize = 0x51B9E8; // g_runRegistration
pub const PATH_PROBLEM: usize = 0x51BA00; // dword_51BA00: 1 a bad folder, 2 no room
pub const IGNORE_ROOM: usize = 0x51BA04; // dword_51BA04
pub const GIVEN_MODE: usize = 0x51BA08; // dword_51BA08 (-i's first digit)
pub const GIVEN_LANGUAGE: usize = 0x51BA0C; // dword_51BA0C (-i's second digit)
pub const DRIVES: usize = 0x51BA10; // dword_51BA10: 26 of (found, free KB)
pub const DRIVE_COUNT: usize = 0x51BAE0; // dword_51BAE0
pub const FIRST_DRIVE: usize = 0x51BAE4; // dword_51BAE4: the first letter looked at
pub const FILES_COPIED: usize = 0x51BAE8; // dword_51BAE8
pub const FILES_TO_COPY: usize = 0x51BAEC; // dword_51BAEC
// dword_51BAF0
pub const ROOM_NEEDED: usize = 0x51BAF8; // dword_51BAF8: KB
pub const BUFFERS: usize = 0x51BAFC; // stru_51BAFC: the installer's memory list
// dword_51BB3C: install.bun
pub const DIRS: usize = 0x51BB40; // dword_51BB40: the folders' names, 0x10000
pub const FILES: usize = 0x51BB44; // dword_51BB44: the files' names, 0x40000
pub const DIR_COUNT: usize = 0x51BB48; // dword_51BB48
pub const FILE_COUNT: usize = 0x51BB4C; // dword_51BB4C
pub const DIR_SCAN: usize = 0x51BB50; // dword_51BB50: the next folder to search
pub const DIR_CURSOR: usize = 0x51BB54; // dword_51BB54
pub const FILE_CURSOR: usize = 0x51BB58; // dword_51BB58
// dword_51C3F7: KB by installation
pub const LAYOUT: usize = 0x51C407; // dword_51C407: the page's pieces, 0x30 bytes each after
// up to unk_51D407's list variables
pub const LIST_ANSWER: usize = 0x51D415; // byte_51D415
pub const KEY_TAKEN: usize = 0x51D414; // byte_51D414
pub const LIST_LEAVE: usize = 0x51D416; // g_listLeave
pub const CHOICE: usize = 0x51D407; // g_listChoice
pub const CHOICE_BEFORE: usize = 0x51D40B; // g_listChoiceBefore
pub const REPEATS: usize = 0x51D40F; // g_listRepeats
pub const KEY_CLOCK: usize = 0x51D41B; // dword_51D41B
pub const FRAME_CLOCK: usize = 0x51D41F; // dword_51D41F
pub const MOVE_CLOCK: usize = 0x51D423; // dword_51D423
pub const CLOCK: usize = 0x518998; // dword_518998
pub const FONTS: usize = 0x51D4C4; // dword_51D4C4: the page's fonts
pub const COLOURS: usize = 0x51D528; // byte_51D528: font by mark, 8 bytes a kind
pub const QUITTING: usize = 0x51837C; // g_quitting
pub const TEXT: usize = 0x93E430; // text: the shared text buffer
pub const SCRATCH: usize = 0x93E8B0; // g_pngInfo: 0x10000 bytes used as scratch
pub const DRIVE_LETTERS: usize = 0x52A30F; // byte_52A30F: the drives' texts, 0x10 each
// off_51DD48: the third page's action
pub const LANGUAGE_COUNT: usize = 0x51BEEA; // g_languageCount
// FindFileData
// FindFileData+13Eh: attributes passed over

// ---- memory by address ----

pub fn rb(at: u32) -> u8 {
    unsafe { (at as *const u8).read_volatile() }
}

pub fn wb(at: u32, v: u8) {
    unsafe { (at as *mut u8).write_volatile(v) }
}

pub fn rw(at: u32) -> u16 {
    unsafe { (at as *const u16).read_unaligned() }
}

pub fn ww(at: u32, v: u16) {
    unsafe { (at as *mut u16).write_unaligned(v) }
}

pub fn rd(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

pub fn wd(at: u32, v: u32) {
    unsafe { (at as *mut u32).write_unaligned(v) }
}

/// The game's address of IDA address `x`.
pub fn a(x: usize) -> u32 {
    game_addr(rva(x)) as u32
}

/// The game's dword at IDA address `x`.
pub fn g(x: usize) -> u32 {
    rd(a(x))
}

pub fn p(x: usize, v: u32) {
    wd(a(x), v)
}

pub fn gb(x: usize) -> u8 {
    rb(a(x))
}

pub fn pb(x: usize, v: u8) {
    wb(a(x), v)
}

/// The place of IDA address `x`: its address in the game.
pub fn at(x: usize) -> u32 {
    a(x)
}

/// The flags of `cmp x, y` as `jb`, `jz` (unsigned) read them.
pub fn cmp(r: &mut Regs, x: u32, y: u32) {
    r.set_carry(x < y);
    r.set_zero(x == y);
}

/// The flags of `test`/`or`/`and` on a result: carry clear, zero by the result.
pub fn logic(r: &mut Regs, result: u32) {
    r.set_carry(false);
    r.set_zero(result == 0);
}

pub fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

// ---- pure text work, as the originals leave the registers ----

/// `strings::str_append` (eax the text, ebx the cursor): ebx past it; the flags of the
/// NUL's `test`.
pub fn str_append(r: &mut Regs) {
    r.ebx = unsafe { crate::strings::str_append::str_append(r.eax as *const u8, r.ebx as *mut u8) }
        as u32;
    logic(r, 0);
}

/// `strings::str_append_indexed` (eax the table, ecx the index, ebx the cursor).
pub fn str_append_indexed(r: &mut Regs) {
    r.ebx = unsafe {
        crate::strings::str_append_indexed::str_append_indexed(
            r.eax as *const u8,
            r.ecx as i32,
            r.ebx as *mut u8,
        )
    } as u32;
    logic(r, 0);
}

/// `strings::append_nul` and the other one-byte cursor helpers.
pub fn append_byte(r: &mut Regs, b: u8) {
    wb(r.ebx, b);
    r.ebx = r.ebx.wrapping_add(1);
}

/// `strings::append_string_by_id` (eax the id, ebx the cursor): ebx past it, eax as the
/// original leaves it.
pub fn append_string_by_id(r: &mut Regs) {
    let mut out = r.ebx as *mut u8;
    r.eax = unsafe {
        crate::strings::append_string_by_id::append_string_by_id(r.eax, &mut |from, n| {
            std::ptr::copy(from, out, n);
            out = out.add(n);
        })
    };
    r.ebx = out as u32;
}

// ---- the callees, by IDA address ----

pub const READ_LANGUAGES: usize = 0x413908; // install::read_languages
pub const BUILD_FONTS: usize = 0x415563; // gfx::build_fonts
pub const FREE_FONTS: usize = 0x415C55; // gfx::free_fonts
pub const READ_RECORD: usize = 0x4110C9; // install::read_record
pub const READ_REGISTRY: usize = 0x413D14; // install::read_registry
pub const LOAD_LANGUAGE: usize = 0x40331F; // strings::load_language
pub const CHOOSE_LANGUAGE: usize = 0x41397A; // install::choose_language (eax)
pub const NORMALISE: usize = 0x4139EC; // install::normalise_path (esi to edx; carry)
pub const SET_CD: usize = 0x403105; // files::set_current_directory (edx; carry)
pub const MAKE_DIR: usize = 0x403113; // files::make_directory (edx; carry)
pub const REMOVE_DIR: usize = 0x403123; // sub_403123 (edx; carry)
pub const DELETE_FILE: usize = 0x402E8C; // files::delete_file (eax; carry)
pub const IS_CD: usize = 0x403310; // files::is_cd_drive (al; zero)
pub const DRIVE_TYPE: usize = 0x4031D8; // files::drive_type (al -> eax)
pub const FREE_SPACE: usize = 0x40317D; // files::drive_free_space_kb (al -> eax; carry)
pub const OPEN: usize = 0x402DE8; // files::open_by_mode (eax, ebx -> eax; carry)
pub const READ: usize = 0x402E3D; // files::read (eax, ebx, ecx -> eax; carry)
pub const CLOSE: usize = 0x402E23; // files::close (eax; carry)
pub const READ_FILE: usize = 0x402BB6; // files::read_file
pub const WRITE_FILE: usize = 0x402BFB; // files::write_file
pub const COPY_FILE: usize = 0x402C28; // files::copy_file (eax to ebx; carry)
pub const RESOLVE_PATH: usize = 0x403206; // files::resolve_path (eax -> eax)
pub const FIND_FIRST: usize = 0x402FC2; // files::find_first (eax; carry)
pub const FIND_NEXT: usize = 0x402FF6; // files::find_next_wanted (carry)
pub const CLOSE_SEARCH: usize = 0x40301F; // files::close_search
pub const ZERO_MEMORY: usize = 0x401829; // system::zero_memory (ebx, ecx)
pub const ALLOCATE: usize = 0x4028A7; // memory::allocate (eax, ebx -> eax; carry)
pub const FREE_LIST: usize = 0x4028F5; // memory::free_list (ebx)
pub const SLEEP: usize = 0x41100C; // system::sleep (eax)
pub const SLEEP_1MS: usize = 0x410FFE; // sleep_1ms
pub const TAKE_KEY: usize = 0x407F95; // input::take_key (-> al; zero)
pub const PUMP: usize = 0x410535; // system::pump_messages (platform)
pub const RUN_FRAME: usize = 0x405A71; // screens::run_frame (eax the frame; platform)
pub const LOCK: usize = 0x4058AD; // gfx::lock_surface (carry; platform)
pub const UNLOCK: usize = 0x405949; // gfx::unlock_surface (platform)
pub const SHOW: usize = 0x4057D1; // gfx::show_frame (platform)
pub const COPY_PICTURE: usize = 0x4060E3; // gfx::copy_picture_to_surface
pub const FILL_RECT: usize = 0x406869; // gfx::fill_rect
pub const FRAME_RECT: usize = 0x4066DF; // gfx::frame_rect
pub const BLIT_GLYPH: usize = 0x4069B2; // gfx::blit_glyph
pub const DRAW_STRING: usize = 0x40234A; // text::draw_string_by_id
pub const DRAW_TEXT: usize = 0x4023D5; // text::draw_text
pub const MEASURE_TEXT: usize = 0x40230B; // text::measure_text
pub const MEASURE_STRING: usize = 0x40232C; // text::measure_string_by_id
pub const KEY_TO_CHAR: usize = 0x407E43; // input::key_to_char
pub const SCREENSHOT: usize = 0x407897; // gfx::take_screenshot
pub const PALETTE_TO_DWORDS: usize = 0x405DDE; // gfx::palette_to_dwords
pub const COMPONENTS_TO_PIXELS: usize = 0x405E02; // gfx::components_to_pixels
pub const LOAD_PICTURE: usize = 0x411863; // gfx::load_picture
pub const CONVERT_PICTURE: usize = 0x406126; // gfx::convert_picture
pub const PICTURE_AS_SURFACE: usize = 0x405FE7; // gfx::picture_as_surface
pub const LIST_TAKE_KEY: usize = 0x41469A; // menu::list_take_key
pub const LIST_STEP: usize = 0x414A55; // menu::list_step
pub const LIST_CLEAR_MARKS: usize = 0x414AC9; // menu::list_clear_marks
pub const LIST_UNUSABLE: usize = 0x414C1B; // menu::list_item_unusable
pub const NEXT_EDIT: usize = 0x41530F; // install::next_edit
pub const LIST_FOLDER: usize = 0x413876; // install::list_folder
pub const LIST_FILES: usize = 0x4138B2; // install::list_files
pub const LIST_FOLDER_TREE: usize = 0x41371C; // install::list_folder_tree
pub const RUN_SCRIPT: usize = 0x41365D; // install::run_script
pub const MAKE_FOLDERS: usize = 0x413A97; // install::make_folders
pub const COPY_COUNTED: usize = 0x4139DA; // install::copy_counted

/// The flags of `cmp eax, 1` after a Win32 call that gives a BOOL in eax: carry when it
/// failed (0), zero when it is 1.
pub fn bool_flags(r: &mut Regs, result: u32) {
    r.eax = result;
    cmp(r, result, 1);
}

/// Whether the standalone's install is running (its calls are logged).
static INSTALL_TRACE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The calls of the installer's Rust versions in the game: each by its callee's address.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::install as i;
    let ida = (place as usize)
        .wrapping_sub(game_addr(0))
        .wrapping_add(0x40_0000);
    // The standalone notes each call of the install (`install_files`) in its log.
    if INSTALL_TRACE.load(std::sync::atomic::Ordering::Relaxed) {
        crate::framework::log(&format!(
            "install: call {ida:#x} eax={:#x} ebx={:#x} ecx={:#x} edx={:#x} esi={:#x} edi={:#x}",
            r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi
        ));
    }
    unsafe {
        match ida {
            // the installer's own (this module's neighbours)
            0x412F34 => {
                i::install_from_command::install_from_command(r, &mut game_part).map(|_| ())?
            }
            0x4130CD => {
                let trace = crate::framework::standalone();
                INSTALL_TRACE.store(trace, std::sync::atomic::Ordering::Relaxed);
                let done = i::install_files::install_files(r, &mut game_part);
                INSTALL_TRACE.store(false, std::sync::atomic::Ordering::Relaxed);
                if trace {
                    crate::framework::log(&format!(
                        "install: the install ended: {:?}",
                        done.as_ref().map(|_| ())
                    ));
                }
                done.map(|_| ())?
            }
            0x4134DF => {
                let trace = crate::framework::standalone();
                INSTALL_TRACE.store(trace, std::sync::atomic::Ordering::Relaxed);
                let done = i::uninstall::uninstall(r, &mut game_part);
                INSTALL_TRACE.store(false, std::sync::atomic::Ordering::Relaxed);
                if trace {
                    crate::framework::log(&format!(
                        "install: the uninstall ended: {:?}",
                        done.as_ref().map(|_| ())
                    ));
                }
                done.map(|_| ())?
            }
            0x413629 => *r = i::open_installed::open_installed(r, &mut game_part)?,
            0x41365D => i::run_script::run_script(r, &mut game_part).map(|_| ())?,
            0x41371C => i::list_folder_tree::list_folder_tree(r, &mut game_part).map(|_| ())?,
            0x413876 => i::list_folder::list_folder(r, &mut game_part).map(|_| ())?,
            0x4138B2 => i::list_files::list_files(r, &mut game_part).map(|_| ())?,
            0x413650 => *r = i::ask_registration::ask_registration(r),
            0x41370C => *r = i::script_name::script_name(r)?,
            0x4139DA => *r = i::copy_counted::copy_counted(r, &mut game_part)?,
            0x413A6C => *r = i::check_folder_name::check_folder_name(r),
            0x413A97 => *r = i::make_folders::make_folders(r, &mut game_part)?,
            0x413B0B => i::check_installed_version::check_installed_version(r, &mut game_part)
                .map(|_| ())?,
            0x413C27 => *r = i::list_drives::list_drives(r, &mut game_part)?,
            0x413DC5 => *r = i::write_registry::write_registry(r, &mut game_api)?,
            0x414199 => *r = i::language_page::language_page(r, &mut game_part)?,
            0x4141EF => *r = i::mark_main_page::mark_main_page(r),
            0x41422B => *r = i::mode_page::mode_page(r),
            0x41425E => *r = i::drive_page::drive_page(r),
            0x4142C7 => *r = i::room_page::room_page(r),
            0x4142DA => *r = i::ignore_room::ignore_room(r),
            0x4142F5 => *r = i::folder_page::folder_page(r, &mut game_part)?,
            0x41432F => *r = i::dxsetup_page::dxsetup_page(r),
            0x414347 => *r = i::finish_page::finish_page(r),
            0x414490 => *r = i::close_installer::close_installer(r, &mut game_part)?,
            0x414363 => i::open_installer::open_installer(r, &mut game_part).map(|_| ())?,
            0x4144AA => *r = i::run_pages::run_pages(r, &mut game_part)?,
            0x4144CF => *r = i::run_page::run_page(r, &mut game_part)?,
            0x4146E4 => *r = i::page_key::page_key(r, &mut game_part)?,
            0x4147FE => *r = i::item_enter::item_enter(r, &mut game_part)?,
            0x414784 => *r = i::key_up::key_up(r)?,
            0x414794 => *r = i::key_down::key_down(r)?,
            0x4147A4 => *r = i::key_left::key_left(r)?,
            0x4147C0 => *r = i::key_right::key_right(r)?,
            0x4147DC => *r = i::key_number::key_number(r)?,
            0x414826 => *r = i::choice_back::choice_back(r)?,
            0x414848 => *r = i::choice_on::choice_on(r)?,
            0x414881 => *r = i::value_down::value_down(r)?,
            0x41489C => *r = i::value_up::value_up(r)?,
            0x41486A => *r = i::choice_by_number::choice_by_number(r)?,
            0x4148B7 => *r = i::value_by_number::value_by_number(r)?,
            0x4148D6 => *r = i::start_edit::start_edit(r),
            0x4148F7 => *r = i::edit_key::edit_key(r)?,
            0x414A28 => *r = i::item_link::item_link(r, &mut game_part)?,
            0x414A4B => *r = i::item_action::item_action(r, &mut game_part)?,
            0x414B53 => *r = i::mark_page::mark_page(r),
            0x414C7E => *r = i::lay_out_pieces::lay_out_pieces(r)?,
            0x414DF5 => *r = i::measure_pieces::measure_pieces(r, &mut game_part)?,
            0x414ED1 => *r = i::place_pieces::place_pieces(r)?,
            0x414F11 => *r = i::place_row::place_row(r)?,
            0x415058 => *r = i::draw_pieces::draw_pieces(r, &mut game_part)?,
            0x41525D => *r = i::show_backdrop::show_backdrop(r, &mut game_part)?,
            0x415273 => *r = i::backdrop_frame::backdrop_frame(r, &mut game_part)?,
            0x415304 => {
                append_byte(r, b'\n');
                append_byte(r, 0);
            }
            0x41530F => *r = i::next_edit::next_edit(r),
            0x41534E => *r = i::folder_key::folder_key(r),
            0x419030 => *r = i::register_lobby::register_lobby(r, &mut game_part, &mut game_api)?,
            0x41675B => i::install_clio::install_clio(r, &mut game_part).map(|_| ())?,
            0x4190E9 => *r = i::unregister_lobby::unregister_lobby(r, &mut game_api)?,
            0x402370 => *r = i::draw_string_list::draw_string_list(r, &mut game_part)?,
            0x402628 => *r = i::blit_glyph_list::blit_glyph_list(r, &mut game_part)?,
            0x40317D => *r = crate::files::drive_free_space_kb::free_space_regs(r, &mut game_api),
            0x411186 => *r = i::write_record::write_record(r, &mut game_part)?,
            // the game's, Rust
            READ_LANGUAGES => i::read_languages::read_languages(a(0x51C36E) as *const u8)?,
            READ_RECORD => i::read_record::read_record(a(0x5295C0) as *const u8)?,
            // the registry, or in the standalone the portable record
            READ_REGISTRY => i::read_registry::read_registry_here(),
            CHOOSE_LANGUAGE => i::choose_language::choose_language(r.eax)?,
            NORMALISE => {
                let written =
                    i::normalise_path::normalise_path(r.esi as *const u8, r.edx as *mut u8)?;
                r.set_carry(!written);
            }
            SET_CD | MAKE_DIR | REMOVE_DIR | DELETE_FILE => {
                let path = if ida == DELETE_FILE { r.eax } else { r.edx };
                let done = crate::files::directories::by_place(ida, path as *const u8);
                bool_flags(r, done);
            }
            IS_CD => {
                let (carry, zero) = crate::files::is_cd_drive::is_cd_drive(r.eax as u8);
                r.set_zero(zero);
                r.set_carry(carry);
            }
            DRIVE_TYPE => r.eax = crate::files::drive_type::drive_type(r.eax as u8),
            OPEN => match crate::files::handle::open_by_mode(r.eax as *const u8, r.ebx)? {
                Some(h) => {
                    r.eax = h;
                    r.set_carry(false);
                }
                None => {
                    r.eax = crate::files::handle::NO_HANDLE;
                    r.set_carry(true);
                }
            },
            READ => {
                let (n, ok) = crate::files::handle::read(r.eax, r.ebx as *mut u8, r.ecx);
                r.eax = n;
                r.set_carry(!ok);
            }
            CLOSE => {
                let ok = crate::files::handle::close(r.eax);
                bool_flags(r, ok as u32);
            }
            COPY_FILE => {
                let done =
                    crate::files::copy_file::copy_file(r.eax as *const u8, r.ebx as *const u8)?;
                r.set_carry(!done);
            }
            FIND_FIRST => {
                let found = crate::race::find_snapshot_number::find_first(r.eax);
                r.set_carry(!found);
            }
            FIND_NEXT => {
                let found = crate::race::find_snapshot_number::find_next_wanted();
                r.set_carry(!found);
            }
            CLOSE_SEARCH => crate::race::find_snapshot_number::close_search(),
            ZERO_MEMORY => crate::system::zero_memory::zero_memory(r.ebx as *mut u8, r.ecx)?,
            ALLOCATE => {
                r.eax = crate::memory::allocate::allocate(r.ebx as *mut _, r.eax)? as u32;
                r.set_carry(false);
            }
            FREE_LIST => crate::memory::free_list::free_list(r.ebx as *mut _)?,
            FREE_FONTS => crate::gfx::free_fonts::free_fonts(
                game_addr(crate::gfx::glyph_tables::TABLES_LIST_RVA) as *mut _,
                game_addr(crate::gfx::build_fonts::PALETTES_RVA) as *mut u32,
            )?,
            SLEEP => crate::system::sleep::sleep(r.eax),
            SLEEP_1MS => crate::system::sleep::sleep(1),
            TAKE_KEY => {
                let key = crate::input::take_key::take_key();
                r.eax = r.eax & !0xFF | key as u32;
                logic(r, key as u32);
            }
            LIST_TAKE_KEY => {
                *r = crate::menu::list_take_key::list_take_key(
                    r,
                    &mut crate::menu::list_take_key::game_part,
                )?
            }
            LIST_STEP => *r = crate::menu::list_step::list_step(r),
            LIST_CLEAR_MARKS => crate::menu::list_clear_marks::list_clear_marks(r.edi),
            LIST_UNUSABLE => {
                let unusable = crate::menu::list_item_unusable::list_item_unusable(r.edi);
                r.set_carry(unusable);
            }
            SCREENSHOT => {
                *r = crate::gfx::take_screenshot::take_screenshot(
                    r,
                    &mut crate::gfx::take_screenshot::game_part,
                )?
            }
            KEY_TO_CHAR => r.eax = crate::input::key_to_char::key_to_char(r.eax)?,
            DRAW_TEXT => draw_text(r)?,
            DRAW_STRING => draw_string(r)?,
            MEASURE_TEXT => measure_text(r)?,
            MEASURE_STRING => measure_string(r)?,
            READ_FILE => read_file(r)?,
            WRITE_FILE => write_file(r)?,
            RESOLVE_PATH => {
                r.eax = crate::files::resolve_path::resolve_path_in_game(r.eax as *const u8)? as u32
            }
            PALETTE_TO_DWORDS => crate::gfx::palette_to_dwords::palette_to_dwords(
                r.esi as *const u8,
                r.edi as *mut u32,
            )?,
            COMPONENTS_TO_PIXELS => components_to_pixels(r),
            LOAD_LANGUAGE => crate::strings::load_language::load_language(&mut || {
                crate::strings::compile_lexicon::compile_lexicon()
            })?,
            // the platform, and drawing whose Rust versions take their own surfaces: the
            // game's hooked functions
            RUN_FRAME | LOCK | UNLOCK | SHOW | PUMP | BUILD_FONTS | COPY_PICTURE | FILL_RECT
            | FRAME_RECT | BLIT_GLYPH | LOAD_PICTURE | CONVERT_PICTURE | PICTURE_AS_SURFACE => {
                crate::menu::table_calls::call_any(place, r)
            }
            // a list's action that is not the installer's: the game's
            _ => crate::menu::table_calls::call_any(place, r),
        }
    }
    Ok(())
}

/// `gfx::components_to_pixels` (0x405E02, not hooked: gfx's): 0x100 colours of esi made
/// pixels of the format ebx into edi; every register kept.
pub unsafe fn components_to_pixels(r: &Regs) {
    let format = unsafe { &*(r.ebx as *const crate::gfx::pixel_format::PixelFormat) };
    for n in 0..0x100u32 {
        let components = rd(r.esi + 4 * n);
        let pixel =
            unsafe { crate::gfx::components_to_pixel::components_to_pixel(format, components) };
        wd(r.edi + 4 * n, pixel);
    }
}

unsafe fn draw_text(r: &mut Regs) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    unsafe {
        let font = r.eax as *mut Font;
        let mut copy = font.read_unaligned();
        let out = crate::text::draw_text::draw_text(
            &mut copy,
            r.ebx as *const u8,
            r.ecx as i32,
            r.edx as i32,
            &mut GameCanvas,
        )?;
        if copy != font.read_unaligned() {
            font.write_unaligned(copy);
        }
        (r.eax, r.ebx, r.ecx, r.edx) = (0, out.text as u32, out.x as u32, out.y as u32);
        r.set_carry(out.too_wide);
        r.set_zero(!out.too_wide);
    }
    Ok(())
}

unsafe fn draw_string(r: &mut Regs) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    let out = unsafe {
        crate::text::draw_string_by_id::draw_string_by_id_at(
            r.eax,
            r.ebx as *mut Font,
            r.ecx as i32,
            r.edx as i32,
            &mut GameCanvas,
        )
    }?;
    (r.eax, r.ebx, r.ecx, r.edx) = (0, out.text as u32, out.x as u32, out.y as u32);
    r.set_carry(out.too_wide);
    r.set_zero(!out.too_wide);
    Ok(())
}

unsafe fn measure_text(r: &mut Regs) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    let m = unsafe {
        crate::text::measure_text::measure_text_at(
            r.eax as *mut Font,
            r.ebx as *const u8,
            r.ecx as i32,
            &mut GameCanvas,
        )
    }?;
    (r.eax, r.ebx, r.ecx, r.edx) = (0, m.lines, m.width, m.outcome.y as u32);
    r.set_carry(m.outcome.too_wide);
    r.set_zero(!m.outcome.too_wide);
    Ok(())
}

unsafe fn measure_string(r: &mut Regs) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    let m = unsafe {
        crate::text::measure_string_by_id::measure_string_by_id_at(
            r.eax,
            r.ebx as *mut Font,
            r.ecx as i32,
            &mut GameCanvas,
        )
    }?;
    (r.eax, r.ebx, r.ecx, r.edx) = (0, m.lines, m.width, m.outcome.y as u32);
    r.set_carry(m.outcome.too_wide);
    r.set_zero(!m.outcome.too_wide);
    Ok(())
}

unsafe fn read_file(r: &mut Regs) -> Result<(), Refused> {
    let read = unsafe {
        crate::files::read_file::read_file(r.eax as *const u8, r.ebx as *mut u8, r.ecx, r.edx)
    }?;
    apply_file_result(r, (read.ok, read.bytes_read));
    Ok(())
}

unsafe fn write_file(r: &mut Regs) -> Result<(), Refused> {
    let written = unsafe {
        crate::files::write_file::write_file(r.eax as *const u8, r.ebx as *const u8, r.ecx)
    }?;
    apply_file_result(r, (written.ok, written.bytes_written));
    Ok(())
}

/// What `read_file` and `write_file` leave: (whether it worked, the bytes if the read or
/// write was reached).
fn apply_file_result(r: &mut Regs, (ok, bytes): (bool, Option<u32>)) {
    if let Some(n) = bytes {
        r.eax = n;
    }
    r.set_carry(!ok);
    r.set_zero(ok);
}

// ---- Win32 by import ----

pub const REG_CREATE_KEY: u32 = 0x4D7CA4; // RegCreateKeyA (3)
pub const REG_SET_VALUE: u32 = 0x4D7CB6; // RegSetValueExA (6)
pub const REG_CLOSE_KEY: u32 = 0x4D7CB0; // RegCloseKey (1)
pub const REG_DELETE_KEY: u32 = 0x4D7C98; // RegDeleteKeyA (2)
pub const GET_DISK_FREE: u32 = 0x4D5D2E; // GetDiskFreeSpaceA (5)

/// The installer's Win32 calls in the game: the platform's, made from Rust. The DirectPlay
/// registry key, the disk-free query and the registration-record registry values; the DDE
/// to the Program Manager and `CreateProcess` stay the game's (`install::run_after_exit`,
/// section 15).
pub fn game_api(place: u32, x: &[u32]) -> u32 {
    use windows_sys::Win32::System::Registry as reg;
    // The standalone is portable: no registry (`install::portable`).
    if crate::framework::standalone()
        && let Some(answer) = crate::install::portable::api(place, x)
    {
        return answer;
    }
    unsafe {
        match place {
            REG_CREATE_KEY => reg::RegCreateKeyA(x[0] as _, x[1] as _, x[2] as _) as u32,
            REG_SET_VALUE => {
                reg::RegSetValueExA(x[0] as _, x[1] as _, x[2], x[3], x[4] as _, x[5]) as u32
            }
            REG_CLOSE_KEY => reg::RegCloseKey(x[0] as _) as u32,
            REG_DELETE_KEY => reg::RegDeleteKeyA(x[0] as _, x[1] as _) as u32,
            GET_DISK_FREE => windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceA(
                x[0] as _, x[1] as _, x[2] as _, x[3] as _, x[4] as _,
            ) as u32,
            _ => 0,
        }
    }
}

// ---- self-tests ----
