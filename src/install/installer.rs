//! The installer that `RAL.EXE` also is (`install::run_installer` and its tree):
//! the variables its parts share, the calls they make, the functions its tables hold (its
//! registry calls: `install::read_registry::Registry`).
//!
//! The installer runs only with its script set up (`-i`, `-u`, `-p`, or the update and Clio
//! discs: `install::check_at_start`, `check_clio_disc`). Its pages are list menus
//! (`install::run_page`): a list of 0x18 bytes (+0 flags, +1 items, +2 title, +4 the item chosen,
//! +8 the list's action with the state 0 at the start, 1 after a key, -1 at the end, +0x10
//! the neighbours for `menu::list_step`, +0x14 a key action) and its items of 0x10 bytes
//! from +0x18 (+0 kind and marks, +1 the next list or the choices, +2 the text, +8 and +0xC
//! the kind's pointers). `install::run_pages` goes from list to list by the answer.
//!
//! The parts call each other and the game's other functions directly with typed arguments; the
//! functions the tables hold (by address in RAL.EXE's data) are run by `page_action`, `key_action`,
//! `item_action_at`, `text_action` and `draw_action`, each knowing the functions the game's tables
//! put in that place.

use crate::errors::{BadInput, FatalError, Refused};
use crate::menu::list_handle::ListMenu;
use crate::place::BytePlace;
use crate::system::owner::system;

// ---- the variables (IDA addresses) ----

/// IDA's address of the installer's texts.
pub const TEXTS_START: usize = 0x5295B4;
/// Their length.
pub const TEXTS_LEN: usize = 0x18;

/// The texts the installer (and `files::resolve_path`) use. Atomic cells: the audio
/// thread's file calls resolve paths against `base_b`.
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct InstallTexts {
    /// `unk_5295B4`: base directory B, an empty text.
    pub base_b: crate::session::Bytes<1>,
    /// `unk_5295B5`: the other directory, "\".
    pub other_dir: crate::session::Bytes<2>,
    /// `byte_5295B7`: ".".
    pub dot: crate::session::Bytes<2>,
    /// `asc_5295B9`: "..".
    pub dotdot: crate::session::Bytes<3>,
    /// `asc_5295BC`: "*.*".
    pub star: crate::session::Bytes<4>,
    /// `aVarRalZog` (`0x5295C0`): the installation's record.
    pub record_path: crate::session::Bytes<12>,
}

/// The installer's texts; the first values are the image's.
impl InstallTexts {
    pub const fn new() -> Self {
        InstallTexts {
            base_b: crate::session::Bytes::new(*b"\0"),
            other_dir: crate::session::Bytes::new(*b"\\\0"),
            dot: crate::session::Bytes::new(*b".\0"),
            dotdot: crate::session::Bytes::new(*b"..\0"),
            star: crate::session::Bytes::new(*b"*.*\0"),
            record_path: crate::session::Bytes::new(*b"var\\ral.zog\0"),
        }
    }
}

// g_baseDirectory, g_localeDirectory: `files::state::{base_directory, locale_directory}`

/// g_mediaMode (`files::state::MEDIA_MODE`).
pub fn media_mode() -> u32 {
    crate::files::state::MEDIA_MODE.get()
}

pub fn set_media_mode(mode: u32) {
    crate::files::state::MEDIA_MODE.set(mode);
}
// The installer's other variables (the update switch, the remove-items switch, the install.bun
// flag, the room by installation, the list variables, the third page's action, the file
// search record and its skip mask) are in `install::vars`.

// ---- memory by address ----

/// A game address the installer's code reads or writes lies outside every allocation: the
/// program ends naming it (the script and its pages have no way to carry the refusal back;
/// `GamePtr` logged the address).
pub fn refuse(e: crate::errors::BadInput) -> ! {
    crate::fatal::raise::end("install", e)
}

#[track_caller]
pub fn rb(at: impl Into<BytePlace>) -> u8 {
    at.into()
        .read(1, "an installer byte")
        .unwrap_or_else(|e| refuse(e))[0]
}

#[track_caller]
pub fn wb(at: impl Into<BytePlace>, v: u8) {
    at.into()
        .write(&[v], "an installer byte")
        .unwrap_or_else(|e| refuse(e));
}

#[track_caller]
pub fn rw(at: impl Into<BytePlace>) -> u16 {
    let b = at
        .into()
        .read(2, "an installer word")
        .unwrap_or_else(|e| refuse(e));
    u16::from_le_bytes([b[0], b[1]])
}

#[track_caller]
pub fn ww(at: impl Into<BytePlace>, v: u16) {
    at.into()
        .write(&v.to_le_bytes(), "an installer word")
        .unwrap_or_else(|e| refuse(e));
}

#[track_caller]
pub fn rd(at: impl Into<BytePlace>) -> u32 {
    let b = at
        .into()
        .read(4, "an installer dword")
        .unwrap_or_else(|e| refuse(e));
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

/// The link (`page_links::PageLink`) the dword at `at` holds.
#[track_caller]
pub fn rl(at: impl Into<BytePlace>) -> crate::install::page_links::PageLink {
    crate::install::page_links::PageLink::from_raw(rd(at))
}

#[track_caller]
pub fn wd(at: impl Into<BytePlace>, v: u32) {
    at.into()
        .write(&v.to_le_bytes(), "an installer dword")
        .unwrap_or_else(|e| refuse(e));
}

pub fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

// ---- pure text work ----

/// The NUL-ended text at `place` (a refused read ends the installer, as every wrapper here).
pub fn text_at(place: impl Into<BytePlace>) -> Vec<u8> {
    place
        .into()
        .text("an installer text")
        .unwrap_or_else(|e| refuse(e))
}

/// The text at `place` with its NUL, as the text drawers and measurers read it (to its NUL).
pub fn text_with_nul(place: impl Into<BytePlace>) -> Vec<u8> {
    let mut text = text_at(place);
    text.push(0);
    text
}

/// `bytes`, the whole text a builder made (its NULs too), written at `at`.
pub fn store_text(at: impl Into<BytePlace>, bytes: &[u8]) {
    at.into()
        .write(bytes, "an installer text")
        .unwrap_or_else(|e| refuse(e));
}

/// `strings::append_string_by_id`: string `id` appended to `out`.
pub fn push_string_by_id(out: &mut Vec<u8>, id: u32) {
    let pushed = crate::strings::append_string_by_id::push_string(out, id);
    crate::menu::event_flow::refuse_here(pushed, "install::installer: a string");
}

// ---- the game's other functions the installer calls ----

/// A path place read as text (a refused read ends the installer, as every wrapper here).
fn path_text(path: impl Into<BytePlace>) -> Vec<u8> {
    path.into().text("a path").unwrap_or_else(|e| refuse(e))
}

/// `files::set_current_directory` (0x403105): whether it worked.
pub fn set_cd(path: impl Into<BytePlace>) -> bool {
    crate::files::directories::set_current_directory(&path_text(path))
}

/// `files::make_directory` (0x403113): whether it worked.
pub fn make_dir(path: impl Into<BytePlace>) -> bool {
    crate::files::directories::make_directory(&path_text(path))
}

/// `files::remove_directory` (remove a directory): whether it worked.
pub fn remove_dir(path: impl Into<BytePlace>) -> bool {
    crate::files::directories::remove_directory(&path_text(path))
}

/// `files::delete_file` (0x402E8C): whether it worked.
pub fn delete_file(path: impl Into<BytePlace>) -> bool {
    crate::files::directories::delete_file(&path_text(path))
}

/// `install::normalise_path` (0x4139EC): `path` written at `to`; whether it was.
pub fn normalise(path: impl Into<BytePlace>, to: impl Into<BytePlace>) -> Result<bool, Refused> {
    Ok(crate::install::normalise_path::normalise_path(
        path.into(),
        to.into(),
    )?)
}

/// `files::copy_file` (0x402C28): whether it worked.
pub fn copy_file(
    system: &crate::system::owner::System,
    from: impl Into<BytePlace>,
    to: impl Into<BytePlace>,
) -> Result<bool, Refused> {
    Ok(crate::files::copy_file::copy_file_in(
        system,
        &from.into().text("a file to copy")?,
        &to.into().text("a copy's path")?,
    )?)
}

/// `files::resolve_path` (0x403206): the path the game opens (without its NUL).
pub fn resolve_path(path: impl Into<BytePlace>) -> Result<Vec<u8>, Refused> {
    Ok(crate::files::resolve_path::resolve(
        &path.into().text("a path")?,
    )?)
}

/// `files::find_first` (0x402FC2): whether a file was found.
pub fn find_first(pattern: &[u8]) -> bool {
    crate::race::find_snapshot_number::find_first(pattern)
}

/// `files::find_next_wanted` (0x402FF6): whether a file was found.
pub fn find_next() -> bool {
    crate::race::find_snapshot_number::find_next_wanted()
}

/// `files::close_search` (0x40301F).
pub fn close_search() {
    crate::files::search::close();
}

/// `system::sleep` (0x41100C).
pub fn sleep(milliseconds: u32) {
    crate::system::sleep::sleep(milliseconds);
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
    crate::platform::pump_messages_now();
}

/// A frame of the installer's for `screens::run_frame`: its carry.
pub type Frame = fn() -> Result<bool, Refused>;

/// `screens::run_frame` (0x405A71): the window's messages, then (unless quitting) the
/// frame; the carry it leaves (the frame's, else clear).
pub fn run_frame(frame: Option<Frame>) -> Result<bool, Refused> {
    // The platform's run_frame with no frame of its own: the window's messages only.
    crate::platform::run_frame_now(|| ());
    match frame {
        Some(frame) if !system().session.flags.quitting.is_set() => frame(),
        _ => Ok(false),
    }
}

/// `gfx::copy_picture_to_surface` (0x4060E3).
pub fn copy_picture() -> Result<(), Refused> {
    {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface,
        )
    }?;
    Ok(())
}

/// `text::draw_string_by_id` (0x40234A): string `id` in `font` at (x, y).
pub fn draw_string(
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    crate::text::draw_string_by_id::draw_string_by_id_at(
        id,
        font,
        x as i32,
        y as i32,
        &mut GameCanvas,
    )?;
    Ok(())
}

/// `text::draw_text` (0x4023D5): `text` in `font` at (x, y); the font object written back
/// when the drawing changed it.
pub fn draw_text(
    font: crate::gfx::typefaces::FontId,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    let at = font.object();
    let mut copy = at.get();
    crate::text::draw_text::draw_text(&mut copy, text, x as i32, y as i32, &mut GameCanvas)?;
    if copy != at.get() {
        at.set(copy);
    }
    Ok(())
}

/// `text::measure_text` (0x40230B), at `x`: (the widest line, the last line's y).
pub fn measure_text(
    font: crate::gfx::typefaces::FontId,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: u32,
) -> Result<(u32, u32), Refused> {
    use crate::text::draw_text::GameCanvas;
    let m = { crate::text::measure_text::measure_text_at(font, text, x as i32, &mut GameCanvas) }?;
    Ok((m.width, m.outcome.y as u32))
}

/// `text::measure_string_by_id` (0x40232C), at `x`: (the widest line, the last line's y).
pub fn measure_string(
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
) -> Result<(u32, u32), Refused> {
    use crate::text::draw_text::GameCanvas;
    let m = {
        crate::text::measure_string_by_id::measure_string_by_id_at(
            id,
            font,
            x as i32,
            &mut GameCanvas,
        )
    }?;
    Ok((m.width, m.outcome.y as u32))
}

/// `gfx::components_to_pixels` (0x405E02): 0x100 colours of `from` made pixels of the
/// surface's format into `to`.
pub fn components_to_pixels(
    from: &[crate::session::PlainDword; 256],
    to: &[crate::session::PlainDword; 256],
) {
    let what = "a pixel format";
    let format = crate::gfx::pixel_format::surface_format()
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
    for (from, to) in from.iter().zip(to) {
        to.set(crate::gfx::components_to_pixel::components_to_pixel(
            &format,
            from.get(),
        ));
    }
}

/// `strings::load_language` (0x40331F).
pub fn load_language() -> Result<(), Refused> {
    let o = crate::owners::boundary();
    crate::strings::load_language::load_language(o, &mut || {
        crate::strings::compile_lexicon::compile_lexicon(o.strings, o.system)
    })
}

/// `menu::list_take_key` (0x41469A): the key (0: none).
pub fn list_take_key() -> Result<u8, Refused> {
    Ok(crate::menu::list_take_key::list_take_key())
}

/// `menu::list_step` (0x414A55): `key` moving the highlight of `list` by the neighbours
/// table `table`; whether it moved (the original's zero).
pub fn list_step(
    key: u32,
    table: Option<crate::place::BytePlace>,
    list: ListMenu,
) -> Result<bool, crate::errors::BadInput> {
    crate::menu::list_step::list_step(key as u8, table, list)
}

/// `menu::list_clear_marks` (0x414AC9).
pub fn list_clear_marks(list: ListMenu) {
    crate::menu::list_clear_marks::list_clear_marks(list);
}

/// `menu::list_item_unusable` (0x414C1B): whether the highlighted item cannot be chosen.
pub fn list_item_unusable(list: ListMenu) -> bool {
    crate::menu::list_item_unusable::list_item_unusable(list)
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
pub fn page_action(
    install: &'static crate::install::owner::Install,
    o: crate::owners::Owners<'static>,
    place: u32,
    state: u32,
    list: ListMenu,
) -> Result<(), Refused> {
    use crate::install as i;
    match place {
        0x414199 => i::language_page::language_page(install, state, list)?,
        0x4141EF => i::mark_main_page::mark_main_page(install, list),
        0x41422B => i::mode_page::mode_page(install, state),
        0x41425E => i::drive_page::drive_page(install, state, list),
        0x4142C7 => i::room_page::room_page(install, state),
        0x4142F5 => i::folder_page::folder_page(install, state, list),
        0x41432F => i::dxsetup_page::dxsetup_page(install, state),
        0x414347 => i::finish_page::finish_page(install, state),
        0x4130CD => logged("install", i::install_files::install_files(install, o))?,
        0x4134DF => logged("uninstall", i::uninstall::uninstall(install))?,
        0x413629 => i::open_installed::open_installed(install)?,
        0x413650 => i::ask_registration::ask_registration(install),
        0x41675B => i::install_clio::install_clio(install, o.system)?,
        _ => return Err(BadInput("a list action that is not one of the installer's").into()),
    }
    Ok(())
}

/// A list's key action (+0x14): `key`; whether it took the
/// key (the original's zero). The game's lists hold only the folder page's.
pub fn key_action(
    install: &'static crate::install::owner::Install,
    place: u32,
    key: u32,
    list: ListMenu,
) -> Result<bool, Refused> {
    match place {
        0x41534E => Ok(crate::install::folder_key::folder_key(install, key, list)),
        _ => Err(BadInput("a key action that is not one of the installer's").into()),
    }
}

/// A link's or an action item's action (+8); the original's zero after it. The game's items
/// hold only the not-enough-room page's "go on anyway".
pub fn item_action_at(
    install: &'static crate::install::owner::Install,
    place: u32,
) -> Result<bool, Refused> {
    match place {
        0x4142DA => {
            crate::install::ignore_room::ignore_room(install);
            Ok(true)
        }
        _ => Err(BadInput("an item action that is not one of the installer's").into()),
    }
}

/// A text item's action (+0xC), given its +8: more text at `cursor`; the cursor after. The
/// game's items hold only `install::end_text_line`.
pub fn text_action(place: u32, out: &mut Vec<u8>) -> Result<(), Refused> {
    match place {
        0x415304 => {
            crate::install::end_text_line::end_text_line(out);
            Ok(())
        }
        _ => Err(BadInput("a text action that is not one of the installer's").into()),
    }
}

/// An action item's drawing (+0xC): the game's lists have no action items.
pub fn draw_action(_place: u32) -> Result<(), Refused> {
    Err(BadInput("an action item's drawing (the game's lists have none)").into())
}

#[cfg(test)]
mod text_tests {
    use super::*;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(
            &crate::install::owner::install().texts,
            TEXTS_START,
            TEXTS_START + TEXTS_LEN,
        );
        assert_eq!(
            crate::moved::place(TEXTS_START),
            crate::install::owner::install().texts.base_b.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x5295C0),
            crate::install::owner::install().texts.record_path.addr() as usize
        );
    }
}
