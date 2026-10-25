//! The installer that `RAL.EXE` also is (`sub_412DE0` and its tree, read 2026-10-19): the
//! variables its parts share, the calls they make, the functions its tables hold, the Win32
//! calls (`Api`).
//!
//! The installer runs only with its script set up (`-i`, `-u`, `-p`, or the update and Clio
//! discs: `install::check_at_start`, `check_clio_disc`). Its pages are list menus
//! (`sub_4144CF`): a list of 0x18 bytes (+0 flags, +1 items, +2 title, +4 the item chosen,
//! +8 the list's action with the state 0 at the start, 1 after a key, -1 at the end, +0x10
//! the neighbours for `menu::list_step`, +0x14 a key action) and its items of 0x10 bytes
//! from +0x18 (+0 kind and marks, +1 the next list or the choices, +2 the text, +8 and +0xC
//! the kind's pointers). `sub_4144AA` goes from list to list by the answer.
//!
//! Conversion phase 3 (2026-10-23): the parts call each other and the game's other
//! functions directly with typed arguments; the functions the tables hold (by address in
//! RAL.EXE's data) are run by `page_action`, `key_action`, `item_action_at`, `text_action`
//! and `draw_action`, each knowing the functions the game's tables put in that place.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::game_addr;
use crate::session::FLAGS;

/// A Win32 call by the address of its import the arguments in, eax out.
pub type Api<'a> = &'a mut dyn FnMut(u32, &[u32]) -> u32;

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
pub const FONTS: usize = 0x51D4C4; // dword_51D4C4: the page's fonts
pub const COLOURS: usize = 0x51D528; // byte_51D528: font by mark, 8 bytes a kind
pub const TEXT: usize = 0x93E430; // text: the shared text buffer
pub const SCRATCH: usize = 0x93E8B0; // g_pngInfo: 0x10000 bytes used as scratch
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

/// The game's address of IDA address `x` (where its variable lives now, if it has moved).
pub fn a(x: impl crate::store::Ida) -> u32 {
    x.here() as u32
}

/// The game's dword at IDA address `x`.
pub fn g(x: impl crate::store::Ida) -> u32 {
    rd(a(x))
}

pub fn p(x: impl crate::store::Ida, v: u32) {
    wd(a(x), v)
}

pub fn gb(x: impl crate::store::Ida) -> u8 {
    rb(a(x))
}

pub fn pb(x: impl crate::store::Ida, v: u8) {
    wb(a(x), v)
}

pub fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

/// The IDA address of a game address (for the dispatchers).
fn ida_of(place: u32) -> usize {
    (place as usize)
        .wrapping_sub(game_addr(0))
        .wrapping_add(0x40_0000)
}

// ---- pure text work ----

/// `strings::str_append`: `text` appended at `cursor`; the cursor past it.
pub fn str_append(text: u32, cursor: u32) -> u32 {
    unsafe { crate::strings::str_append::str_append(text as *const u8, cursor as *mut u8) as u32 }
}

/// `strings::str_append_indexed`: text `index` of `table` appended at `cursor`.
pub fn str_append_indexed(table: u32, index: u32, cursor: u32) -> u32 {
    unsafe {
        crate::strings::str_append_indexed::str_append_indexed(
            table as *const u8,
            index as i32,
            cursor as *mut u8,
        ) as u32
    }
}

/// `strings::append_nul` and the other one-byte cursor helpers.
pub fn append_byte(cursor: u32, b: u8) -> u32 {
    wb(cursor, b);
    cursor.wrapping_add(1)
}

/// `strings::append_string_by_id`: string `id` appended at `cursor`; the cursor past it.
pub fn append_string_by_id(id: u32, cursor: u32) -> u32 {
    let mut out = cursor as *mut u8;
    unsafe {
        crate::strings::append_string_by_id::append_string_by_id(id, &mut |from, n| {
            std::ptr::copy(from, out, n);
            out = out.add(n);
        });
    }
    out as u32
}

// ---- the game's other functions the installer calls ----

/// `files::set_current_directory` (0x403105): whether it worked.
pub fn set_cd(path: u32) -> bool {
    unsafe { crate::files::directories::set_current_directory(path as *const u8) != 0 }
}

/// `files::make_directory` (0x403113): whether it worked.
pub fn make_dir(path: u32) -> bool {
    unsafe { crate::files::directories::make_directory(path as *const u8) != 0 }
}

/// `sub_403123` (remove a directory): whether it worked.
pub fn remove_dir(path: u32) -> bool {
    unsafe { crate::files::directories::remove_directory(path as *const u8) != 0 }
}

/// `files::delete_file` (0x402E8C): whether it worked.
pub fn delete_file(path: u32) -> bool {
    unsafe { crate::files::directories::delete_file(path as *const u8) != 0 }
}

/// `install::normalise_path` (0x4139EC): `path` written at `to`; whether it was.
pub fn normalise(path: u32, to: u32) -> Result<bool, Refused> {
    Ok(
        unsafe {
            crate::install::normalise_path::normalise_path(path as *const u8, to as *mut u8)
        }?,
    )
}

/// `files::copy_file` (0x402C28): whether it worked.
pub fn copy_file(from: u32, to: u32) -> Result<bool, Refused> {
    Ok(unsafe { crate::files::copy_file::copy_file(from as *const u8, to as *const u8) }?)
}

/// `files::resolve_path` (0x403206): the path the game opens.
pub fn resolve_path(path: u32) -> Result<u32, Refused> {
    Ok(unsafe { crate::files::resolve_path::resolve_path_in_game(path as *const u8) }? as u32)
}

/// `files::find_first` (0x402FC2): whether a file was found.
pub fn find_first(pattern: u32) -> bool {
    unsafe { crate::race::find_snapshot_number::find_first(pattern) }
}

/// `files::find_next_wanted` (0x402FF6): whether a file was found.
pub fn find_next() -> bool {
    unsafe { crate::race::find_snapshot_number::find_next_wanted() }
}

/// `files::close_search` (0x40301F).
pub fn close_search() {
    unsafe { crate::race::find_snapshot_number::close_search() }
}

/// `system::sleep` (0x41100C).
pub fn sleep(milliseconds: u32) {
    crate::system::sleep::sleep(milliseconds)
}

/// `input::take_key` (0x407F95): the pending key (0: none).
pub fn take_key() -> u8 {
    crate::input::take_key::take_key()
}

/// `gfx::lock_surface` (0x4058AD): whether the surface is locked (the original's carry
/// clear).
pub fn lock() -> bool {
    crate::platform::lock_surface() >= 1
}

/// `gfx::unlock_surface` (0x405949).
pub fn unlock() {
    crate::platform::unlock_surface();
}

/// `gfx::show_frame` (0x4057D1): whether it was shown (the original's carry clear).
pub fn show() -> bool {
    crate::platform::show_frame_now()
}

/// `system::pump_messages` (0x410535).
pub fn pump() {
    crate::platform::pump_messages_now()
}

/// A frame of the installer's for `screens::run_frame`: its carry.
pub type Frame = fn() -> Result<bool, Refused>;

/// `screens::run_frame` (0x405A71): the window's messages, then (unless quitting) the
/// frame; the carry it leaves (the frame's, else clear).
pub fn run_frame(frame: Option<Frame>) -> Result<bool, Refused> {
    // The platform's run_frame with no frame of its own: the window's messages only.
    crate::platform::run_frame_now(|| ());
    match frame {
        Some(frame) if FLAGS.quitting.get() != 1 => frame(),
        _ => Ok(false),
    }
}

/// `gfx::copy_picture_to_surface` (0x4060E3).
pub fn copy_picture() -> Result<(), Refused> {
    unsafe {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface,
        )
    }?;
    Ok(())
}

/// `text::draw_string_by_id` (0x40234A): string `id` in `font` at (x, y).
pub fn draw_string(id: u32, font: u32, x: u32, y: u32) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    unsafe {
        crate::text::draw_string_by_id::draw_string_by_id_at(
            id,
            font as *mut crate::text::font::Font,
            x as i32,
            y as i32,
            &mut GameCanvas,
        )
    }?;
    Ok(())
}

/// `text::draw_text` (0x4023D5): `text` in `font` at (x, y); the font object written back
/// when the drawing changed it.
pub fn draw_text(font: u32, text: u32, x: u32, y: u32) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    unsafe {
        let font = font as *mut Font;
        let mut copy = font.read_unaligned();
        crate::text::draw_text::draw_text(
            &mut copy,
            text as *const u8,
            x as i32,
            y as i32,
            &mut GameCanvas,
        )?;
        if copy != font.read_unaligned() {
            font.write_unaligned(copy);
        }
    }
    Ok(())
}

/// `text::measure_text` (0x40230B), ecx `x`: (the widest line, the last line's y).
pub fn measure_text(font: u32, text: u32, x: u32) -> Result<(u32, u32), Refused> {
    use crate::text::draw_text::GameCanvas;
    let m = unsafe {
        crate::text::measure_text::measure_text_at(
            font as *mut crate::text::font::Font,
            text as *const u8,
            x as i32,
            &mut GameCanvas,
        )
    }?;
    Ok((m.width, m.outcome.y as u32))
}

/// `text::measure_string_by_id` (0x40232C), ecx `x`: (the widest line, the last line's y).
pub fn measure_string(id: u32, font: u32, x: u32) -> Result<(u32, u32), Refused> {
    use crate::text::draw_text::GameCanvas;
    let m = unsafe {
        crate::text::measure_string_by_id::measure_string_by_id_at(
            id,
            font as *mut crate::text::font::Font,
            x as i32,
            &mut GameCanvas,
        )
    }?;
    Ok((m.width, m.outcome.y as u32))
}

/// `gfx::components_to_pixels` (0x405E02): 0x100 colours of `from` made pixels of the
/// format `format` into `to`.
pub unsafe fn components_to_pixels(from: u32, to: u32, format: u32) {
    let format = unsafe { &*(format as *const crate::gfx::pixel_format::PixelFormat) };
    for n in 0..0x100u32 {
        let components = rd(from + 4 * n);
        let pixel =
            unsafe { crate::gfx::components_to_pixel::components_to_pixel(format, components) };
        wd(to + 4 * n, pixel);
    }
}

/// `strings::load_language` (0x40331F).
pub fn load_language() -> Result<(), Refused> {
    unsafe {
        crate::strings::load_language::load_language(&mut || {
            crate::strings::compile_lexicon::compile_lexicon()
        })
    }
}

/// `menu::list_take_key` (0x41469A): the key (0: none).
pub fn list_take_key() -> Result<u8, Refused> {
    Ok(crate::menu::list_take_key::list_take_key().key)
}

/// `menu::list_step` (0x414A55): `key` moving the highlight of `list` by the neighbours
/// table `table`; whether it moved (the original's zero).
pub fn list_step(key: u32, table: u32, list: u32) -> bool {
    unsafe { crate::menu::list_step::list_step(key as u8, table, list) }.zero
}

/// `menu::list_clear_marks` (0x414AC9).
pub fn list_clear_marks(list: u32) {
    unsafe { crate::menu::list_clear_marks::list_clear_marks(list) }
}

/// `menu::list_item_unusable` (0x414C1B): whether the highlighted item cannot be chosen.
pub fn list_item_unusable(list: u32) -> bool {
    unsafe { crate::menu::list_item_unusable::list_item_unusable(list) }
}

/// Whether the standalone's install or uninstall ended, logged.
pub fn logged(what: &str, done: Result<(), Refused>) -> Result<(), Refused> {
    crate::framework::log(&format!(
        "install: the {what} ended: {:?}",
        done.as_ref().map(|_| ())
    ));
    done
}

// ---- the functions the installer's tables hold, by place ----

/// A list's action (+8): the page told where it is (`state` 0 at the start, 1 after a key,
/// -1 at the end). The game's lists hold the pages' actions and, in lists of nothing else
/// (flags 0x40), the install, the uninstall, the installed copy's start (or the Clio
/// disc's install) and the registration.
pub fn page_action(place: u32, state: u32, list: u32) -> Result<(), Refused> {
    use crate::install as i;
    match ida_of(place) {
        0x414199 => i::language_page::language_page(state, list)?,
        0x4141EF => i::mark_main_page::mark_main_page(list),
        0x41422B => i::mode_page::mode_page(state),
        0x41425E => i::drive_page::drive_page(state, list),
        0x4142C7 => i::room_page::room_page(state),
        0x4142F5 => i::folder_page::folder_page(state, list),
        0x41432F => i::dxsetup_page::dxsetup_page(state),
        0x414347 => i::finish_page::finish_page(state),
        0x4130CD => logged("install", i::install_files::install_files())?,
        0x4134DF => logged("uninstall", i::uninstall::uninstall())?,
        0x413629 => i::open_installed::open_installed()?,
        0x413650 => i::ask_registration::ask_registration(),
        0x41675B => i::install_clio::install_clio()?,
        _ => return Err(BadInput("a list action that is not one of the installer's").into()),
    }
    Ok(())
}

/// A list's key action (+0x14): `key` (eax: al the key, ah its bits); whether it took the
/// key (the original's zero). The game's lists hold only the folder page's.
pub fn key_action(place: u32, key: u32, list: u32) -> Result<bool, Refused> {
    match ida_of(place) {
        0x41534E => Ok(crate::install::folder_key::folder_key(key, list)),
        _ => Err(BadInput("a key action that is not one of the installer's").into()),
    }
}

/// A link's or an action item's action (+8); the original's zero after it. The game's items
/// hold only the not-enough-room page's "go on anyway".
pub fn item_action_at(place: u32) -> Result<bool, Refused> {
    match ida_of(place) {
        0x4142DA => {
            crate::install::ignore_room::ignore_room();
            Ok(true)
        }
        _ => Err(BadInput("an item action that is not one of the installer's").into()),
    }
}

/// A text item's action (+0xC), eax its +8: more text at `cursor`; the cursor after. The
/// game's items hold only `install::end_text_line`.
pub fn text_action(place: u32, cursor: u32) -> Result<u32, Refused> {
    match ida_of(place) {
        0x415304 => Ok(unsafe { crate::install::end_text_line::end_text_line(cursor) }),
        _ => Err(BadInput("a text action that is not one of the installer's").into()),
    }
}

/// An action item's drawing (+0xC): the game's lists have no action items.
pub fn draw_action(_place: u32) -> Result<(), Refused> {
    Err(BadInput("an action item's drawing (the game's lists have none)").into())
}

// ---- Win32 by import ----

pub const REG_CREATE_KEY: u32 = 0x4D7CA4; // RegCreateKeyA (3)
pub const REG_SET_VALUE: u32 = 0x4D7CB6; // RegSetValueExA (6)
pub const REG_CLOSE_KEY: u32 = 0x4D7CB0; // RegCloseKey (1)
pub const REG_DELETE_KEY: u32 = 0x4D7C98; // RegDeleteKeyA (2)
pub const GET_DISK_FREE: u32 = 0x4D5D2E; // GetDiskFreeSpaceA (5)

/// The installer's Win32 calls in the game: the platform's, made from Rust. The DirectPlay
/// registry key, the disk-free query and the registration-record registry values; the DDE
/// to the Program Manager and `CreateProcess` stay the game's (`install::run_after_exit`).
pub fn game_api(place: u32, x: &[u32]) -> u32 {
    use windows_sys::Win32::System::Registry as reg;
    // The standalone is portable: no registry (`install::portable`).
    if let Some(answer) = crate::install::portable::api(place, x) {
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
