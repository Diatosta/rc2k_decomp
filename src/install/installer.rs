//! The installer that `RAL.EXE` also is (`sub_412DE0` and its tree, read 2026-10-19): the
//! variables its parts share, the calls they make, the functions its tables hold (its
//! registry calls: `install::read_registry::Registry`).
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
use crate::menu::list_handle::ListMenu;
use crate::place::Place;
use crate::session::FLAGS;

// ---- the variables (IDA addresses) ----

/// IDA's address of the installer's texts.
pub const TEXTS_START: usize = 0x5295B4;
/// Their length.
pub const TEXTS_LEN: usize = 0x18;

/// The texts the installer (and `files::resolve_path`) use.
#[repr(C)]
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
pub static INSTALL_TEXTS: InstallTexts = InstallTexts {
    base_b: crate::session::Bytes::new(*b"\0"),
    other_dir: crate::session::Bytes::new(*b"\\\0"),
    dot: crate::session::Bytes::new(*b".\0"),
    dotdot: crate::session::Bytes::new(*b"..\0"),
    star: crate::session::Bytes::new(*b"*.*\0"),
    record_path: crate::session::Bytes::new(*b"var\\ral.zog\0"),
};

/// Where the texts are (`moved::MOVED`'s home for IDA 0x5295B4).
pub fn texts_home() -> usize {
    crate::pointers::game_address(&raw const INSTALL_TEXTS) as usize
}

// g_baseDirectory, g_localeDirectory: `files::state::{base_directory, locale_directory}`

/// g_mediaMode (`files::state::MEDIA_MODE`).
pub fn media_mode() -> u32 {
    crate::files::state::MEDIA_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn set_media_mode(mode: u32) {
    crate::files::state::MEDIA_MODE.store(mode, std::sync::atomic::Ordering::Relaxed);
}
// dword_51B9CC (-u)
// g_removeStartItems
// dword_51BAF0
// dword_51BB3C: install.bun
// dword_51C3F7: KB by installation
// up to unk_51D407's list variables
// off_51DD48: the third page's action
// FindFileData
// FindFileData+13Eh: attributes passed over

// ---- memory by address ----

/// A game address the installer's code reads or writes lies outside every allocation: the
/// program ends naming it (the script and its pages have no way to carry the refusal back;
/// `GamePtr` logged the address).
pub fn refuse(e: crate::errors::BadInput) -> ! {
    crate::fatal::raise::end("install", e)
}

pub fn rb(at: impl Into<Place>) -> u8 {
    at.into()
        .read(1, "an installer byte")
        .unwrap_or_else(|e| refuse(e))[0]
}

pub fn wb(at: impl Into<Place>, v: u8) {
    at.into()
        .write(&[v], "an installer byte")
        .unwrap_or_else(|e| refuse(e));
}

pub fn rw(at: impl Into<Place>) -> u16 {
    let b = at
        .into()
        .read(2, "an installer word")
        .unwrap_or_else(|e| refuse(e));
    u16::from_le_bytes([b[0], b[1]])
}

pub fn ww(at: impl Into<Place>, v: u16) {
    at.into()
        .write(&v.to_le_bytes(), "an installer word")
        .unwrap_or_else(|e| refuse(e));
}

pub fn rd(at: impl Into<Place>) -> u32 {
    let at = at.into();
    let b = at
        .read(4, "an installer dword")
        .unwrap_or_else(|e| refuse(e));
    let value = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
    crate::install::vars::InstallerPages::resolved(at.game(), value).unwrap_or_else(|e| refuse(e))
}

pub fn wd(at: impl Into<Place>, v: u32) {
    at.into()
        .write(&v.to_le_bytes(), "an installer dword")
        .unwrap_or_else(|e| refuse(e));
}

pub fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

// ---- pure text work ----

/// `strings::str_append`: `text` appended at `cursor`; the cursor past it.
pub fn str_append(text: impl Into<Place>, cursor: impl Into<Place>) -> Place {
    let (text, cursor) = (text.into(), cursor.into());
    let bytes = text.text("an installer text").unwrap_or_else(|e| refuse(e));
    let len = bytes.len() as u32;
    cursor
        .write(&bytes, "an installer text")
        .unwrap_or_else(|e| refuse(e));
    cursor.add(len)
}

/// `strings::append_nul` and the other one-byte cursor helpers.
pub fn append_byte(cursor: impl Into<Place>, b: u8) -> Place {
    let cursor = cursor.into();
    wb(cursor, b);
    cursor.add(1)
}

/// `strings::append_string_by_id`: string `id` appended at `cursor`; the cursor past it.
pub fn append_string_by_id(id: u32, cursor: impl Into<Place>) -> Place {
    let cursor = cursor.into();
    let mut text = Vec::new();
    let written = crate::strings::append_string_by_id::push_string(&mut text, id)
        .and_then(|()| cursor.write(&text, "a text"));
    crate::menu::event_flow::refuse_here(written, "install::installer: a string");
    cursor.add(text.len() as u32)
}

// ---- the game's other functions the installer calls ----

/// `files::set_current_directory` (0x403105): whether it worked.
pub fn set_cd(path: impl Into<Place>) -> bool {
    crate::files::directories::set_current_directory(path.into().game_ptr())
        .unwrap_or_else(|e| refuse(e))
        != 0
}

/// `files::make_directory` (0x403113): whether it worked.
pub fn make_dir(path: impl Into<Place>) -> bool {
    crate::files::directories::make_directory(path.into().game_ptr()).unwrap_or_else(|e| refuse(e))
        != 0
}

/// `sub_403123` (remove a directory): whether it worked.
pub fn remove_dir(path: impl Into<Place>) -> bool {
    crate::files::directories::remove_directory(path.into().game_ptr())
        .unwrap_or_else(|e| refuse(e))
        != 0
}

/// `files::delete_file` (0x402E8C): whether it worked.
pub fn delete_file(path: impl Into<Place>) -> bool {
    crate::files::directories::delete_file(path.into().game_ptr()).unwrap_or_else(|e| refuse(e))
        != 0
}

/// `install::normalise_path` (0x4139EC): `path` written at `to`; whether it was.
pub fn normalise(path: impl Into<Place>, to: impl Into<Place>) -> Result<bool, Refused> {
    Ok(crate::install::normalise_path::normalise_path(
        path.into(),
        to.into(),
    )?)
}

/// `files::copy_file` (0x402C28): whether it worked.
pub fn copy_file(from: impl Into<Place>, to: impl Into<Place>) -> Result<bool, Refused> {
    Ok(crate::files::copy_file::copy_file(
        &from.into().text("a file to copy")?,
        &to.into().text("a copy's path")?,
    )?)
}

/// `files::resolve_path` (0x403206): the path the game opens.
pub fn resolve_path(path: impl Into<Place>) -> Result<crate::pointers::GamePtr, Refused> {
    Ok(crate::files::resolve_path::resolve_path_at(
        path.into().game_ptr(),
    )?)
}

/// `files::find_first` (0x402FC2): whether a file was found.
pub fn find_first(pattern: impl Into<Place>) -> bool {
    crate::race::find_snapshot_number::find_first(pattern.into().game_ptr())
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
        Some(frame) if FLAGS.quitting.get() != 1 => frame(),
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
pub fn draw_string(id: u32, font: crate::pointers::GamePtr, x: u32, y: u32) -> Result<(), Refused> {
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
    font: u32,
    text: impl Into<crate::place::Place>,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    let text = text.into().game_ptr();
    use crate::text::draw_text::GameCanvas;
    use crate::text::font::Font;
    let at = crate::pointers::GamePtr::new(font);
    let mut copy = at.read::<Font>("a font object")?;
    crate::text::draw_text::draw_text(&mut copy, text, x as i32, y as i32, &mut GameCanvas)?;
    if copy != at.read::<Font>("a font object")? {
        at.write(copy, "a font object")?;
    }
    Ok(())
}

/// `text::measure_text` (0x40230B), ecx `x`: (the widest line, the last line's y).
pub fn measure_text(
    font: u32,
    text: impl Into<crate::place::Place>,
    x: u32,
) -> Result<(u32, u32), Refused> {
    let text = text.into().game_ptr();
    use crate::text::draw_text::GameCanvas;
    let m = {
        crate::text::measure_text::measure_text_at(
            crate::pointers::GamePtr::new(font),
            text,
            x as i32,
            &mut GameCanvas,
        )
    }?;
    Ok((m.width, m.outcome.y as u32))
}

/// `text::measure_string_by_id` (0x40232C), ecx `x`: (the widest line, the last line's y).
pub fn measure_string(id: u32, font: u32, x: u32) -> Result<(u32, u32), Refused> {
    use crate::text::draw_text::GameCanvas;
    let m = {
        crate::text::measure_string_by_id::measure_string_by_id_at(
            id,
            crate::pointers::GamePtr::new(font),
            x as i32,
            &mut GameCanvas,
        )
    }?;
    Ok((m.width, m.outcome.y as u32))
}

/// `gfx::components_to_pixels` (0x405E02): 0x100 colours of `from` made pixels of the
/// surface's format into `to`.
pub fn components_to_pixels(
    from: &crate::moved::Place<[u32; 256]>,
    to: &crate::moved::Place<[u32; 256]>,
) {
    let what = "a pixel format";
    let format = crate::gfx::pixel_format::surface_format()
        .unwrap_or_else(|e| crate::fatal::raise::end(what, e));
    let (from, to) = (
        crate::pointers::GamePtr::to(from),
        crate::pointers::GamePtr::to(to),
    );
    for n in 0..0x100u32 {
        let components = from
            .offset(4 * n)
            .read::<u32>("an installer dword")
            .unwrap_or_else(|e| refuse(e));
        let pixel = crate::gfx::components_to_pixel::components_to_pixel(&format, components);
        to.offset(4 * n)
            .write(pixel, "an installer dword")
            .unwrap_or_else(|e| refuse(e));
    }
}

/// `strings::load_language` (0x40331F).
pub fn load_language() -> Result<(), Refused> {
    crate::strings::load_language::load_language(&mut || {
        crate::strings::compile_lexicon::compile_lexicon()
    })
}

/// `menu::list_take_key` (0x41469A): the key (0: none).
pub fn list_take_key() -> Result<u8, Refused> {
    Ok(crate::menu::list_take_key::list_take_key())
}

/// `menu::list_step` (0x414A55): `key` moving the highlight of `list` by the neighbours
/// table `table`; whether it moved (the original's zero).
pub fn list_step(key: u32, table: u32, list: ListMenu) -> Result<bool, crate::errors::BadInput> {
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
pub fn page_action(place: u32, state: u32, list: ListMenu) -> Result<(), Refused> {
    use crate::install as i;
    match place {
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
pub fn key_action(place: u32, key: u32, list: ListMenu) -> Result<bool, Refused> {
    match place {
        0x41534E => Ok(crate::install::folder_key::folder_key(key, list)),
        _ => Err(BadInput("a key action that is not one of the installer's").into()),
    }
}

/// A link's or an action item's action (+8); the original's zero after it. The game's items
/// hold only the not-enough-room page's "go on anyway".
pub fn item_action_at(place: u32) -> Result<bool, Refused> {
    match place {
        0x4142DA => {
            crate::install::ignore_room::ignore_room();
            Ok(true)
        }
        _ => Err(BadInput("an item action that is not one of the installer's").into()),
    }
}

/// A text item's action (+0xC), eax its +8: more text at `cursor`; the cursor after. The
/// game's items hold only `install::end_text_line`.
pub fn text_action(place: u32, cursor: impl Into<Place>) -> Result<Place, Refused> {
    match place {
        0x415304 => Ok(crate::install::end_text_line::end_text_line(cursor.into())),
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
            &INSTALL_TEXTS,
            TEXTS_START,
            TEXTS_START + TEXTS_LEN,
        );
        assert_eq!(
            crate::moved::place(TEXTS_START),
            INSTALL_TEXTS.base_b.addr() as usize
        );
        assert_eq!(
            crate::moved::place(0x5295C0),
            INSTALL_TEXTS.record_path.addr() as usize
        );
    }
}
