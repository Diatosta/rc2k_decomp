//! The installer (`install::run_installer` and its tree): the variables its parts share, the calls
//! they make and the functions its page tables hold (its registry calls:
//! `install::read_registry::Registry`).
//!
//! The installer runs only with its script set up (`-i`, `-u`, `-p`, or the update and Clio
//! discs: `install::check_at_start`, `check_clio_disc`). Its pages are list menus
//! (`install::run_page`): a list of 0x18 bytes (+0 flags, +1 items, +2 title, +4 the item chosen,
//! +8 the list's action with the state 0 at the start, 1 after a key, -1 at the end, +0x10 the
//! neighbours for `menu::list_step`, +0x14 a key action) and its items of 0x10 bytes from +0x18
//! (+0 kind and marks, +1 the next list or the choices, +2 the text, +8 and +0xC the kind's
//! pointers). `install::run_pages` goes from list to list by the answer. The functions the
//! tables hold are run by `page_action`, `key_action`, `item_action_at`, `text_action` and
//! `draw_action`.

use crate::errors::{BadInput, FatalError, Refused};
use crate::list_handle::ListMenu;
use crate::place::BytePlace;

/// The installer's constant texts.
#[derive(rc2k_derive::Cells)]
pub struct InstallTexts {
    /// The other directory, "\".
    pub other_dir: crate::session::Bytes<2>,
    /// ".".
    pub dot: crate::session::Bytes<2>,
    /// "..".
    pub dotdot: crate::session::Bytes<3>,
    /// "*.*".
    pub star: crate::session::Bytes<4>,
    /// The installation's record.
    pub record_path: crate::session::Bytes<12>,
}

/// The installer's texts.
impl InstallTexts {
    pub const fn new() -> Self {
        InstallTexts {
            other_dir: crate::session::Bytes::new(*b"\\\0"),
            dot: crate::session::Bytes::new(*b".\0"),
            dotdot: crate::session::Bytes::new(*b"..\0"),
            star: crate::session::Bytes::new(*b"*.*\0"),
            record_path: crate::session::Bytes::new(*b"var\\ral.zog\0"),
        }
    }
}

// The base and locale directories: `files::state::{base_directory, locale_directory}`.

/// The media mode (`files::state::MEDIA_MODE`).
pub fn media_mode() -> u32 {
    crate::files::state::MEDIA_MODE.get()
}

pub fn set_media_mode(mode: u32) {
    crate::files::state::MEDIA_MODE.set(mode);
}
// The installer's other variables (the update switch, the remove-items switch, the install.bun
// flag, the room by installation, the list variables, the third page's action, the file
// search record and its skip mask) are in `install::vars`.

/// A read or write of the installer's code is refused: the program ends naming it (the
/// script and its pages have no way to carry the refusal back).
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

#[track_caller]
pub fn wd(at: impl Into<BytePlace>, v: u32) {
    at.into()
        .write(&v.to_le_bytes(), "an installer dword")
        .unwrap_or_else(|e| refuse(e));
}

pub fn fatal(code: u32) -> Refused {
    FatalError { code, message: "" }.into()
}

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
pub fn push_string_by_id(
    o: crate::strings::append_string_by_id::StringOwners<'static>,
    out: &mut Vec<u8>,
    id: u32,
) {
    let pushed = crate::strings::append_string_by_id::push_string(o, out, id);
    crate::menu::event_flow::refuse_here(pushed, "install::installer: a string");
}

/// A path place read as text (a refused read ends the installer, as every wrapper here).
fn path_text(path: impl Into<BytePlace>) -> Vec<u8> {
    path.into().text("a path").unwrap_or_else(|e| refuse(e))
}

/// Makes `path` the current directory; whether it worked.
pub fn set_cd(path: impl Into<BytePlace>) -> bool {
    crate::files::directories::set_current_directory(&path_text(path))
}

/// Makes the folder `path`; whether it worked.
pub fn make_dir(path: impl Into<BytePlace>) -> bool {
    crate::files::directories::make_directory(&path_text(path))
}

/// `files::remove_directory` (remove a directory): whether it worked.
pub fn remove_dir(path: impl Into<BytePlace>) -> bool {
    crate::files::directories::remove_directory(&path_text(path))
}

/// Deletes the file `path`; whether it worked.
pub fn delete_file(path: impl Into<BytePlace>) -> bool {
    crate::files::directories::delete_file(&path_text(path))
}

/// `path` normalised and written at `to`; whether it was.
pub fn normalise(path: impl Into<BytePlace>, to: impl Into<BytePlace>) -> Result<bool, Refused> {
    Ok(crate::install::normalise_path::normalise_path(
        path.into(),
        to.into(),
    )?)
}

/// Copies the file `from` to `to`; whether it worked.
pub fn copy_file(
    system: &crate::system::owner::System,
    from: impl Into<BytePlace>,
    to: impl Into<BytePlace>,
) -> Result<bool, Refused> {
    Ok(crate::files::copy_file::copy_file(
        system,
        &from.into().text("a file to copy")?,
        &to.into().text("a copy's path")?,
    )?)
}

/// The path the game opens (without its NUL).
pub fn resolve_path(path: impl Into<BytePlace>) -> Result<Vec<u8>, Refused> {
    Ok(crate::files::resolve_path::resolve(
        &path.into().text("a path")?,
    )?)
}

/// Starts a file search on `pattern`; whether a file was found.
pub fn find_first(pattern: &[u8]) -> bool {
    crate::race::find_snapshot_number::find_first(pattern)
}

/// The search's next wanted file; whether one was found.
pub fn find_next() -> bool {
    crate::race::find_snapshot_number::find_next_wanted()
}

pub fn close_search() {
    crate::files::search::close();
}

pub fn sleep(milliseconds: u32) {
    crate::system::sleep::sleep(milliseconds);
}

/// The pending key (0: none).
pub fn take_key(input: &crate::input::owner::Input) -> u8 {
    crate::input::take_key::take_key(input)
}

/// Whether the surface is locked.
pub fn lock(gfx: &crate::gfx::owner::Gfx, system: &crate::system::owner::System) -> bool {
    crate::platform::lock_surface(gfx, system) >= 1
}

pub fn unlock(gfx: &crate::gfx::owner::Gfx) {
    crate::platform::unlock_surface(gfx);
}

/// Whether it was shown.
pub fn show(o: crate::owners::Owners<'static>) -> bool {
    crate::platform::show_frame_now(o)
}

pub fn pump(o: crate::owners::Owners<'static>) {
    crate::platform::pump_messages_now(o);
}

/// A frame of the installer's for `screens::run_frame`; true when it failed.
pub type Frame<'a> = &'a dyn Fn() -> Result<bool, Refused>;

/// The window's messages, then (unless quitting) the frame; true when the frame failed.
pub fn run_frame(
    o: crate::owners::Owners<'static>,
    frame: Option<Frame<'_>>,
) -> Result<bool, Refused> {
    // The platform's run_frame with no frame of its own: the window's messages only.
    crate::platform::run_frame_now(o, || ());
    match frame {
        Some(frame) if !o.system.session.flags.quitting.is_set() => frame(),
        _ => Ok(false),
    }
}

pub fn copy_picture(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface(gfx, system),
        )
    }?;
    Ok(())
}

/// String `id` in `font` at (x, y).
pub fn draw_string(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    crate::text::draw_string_by_id::draw_string_by_id_at(
        so,
        id,
        font,
        x as i32,
        y as i32,
        &mut GameCanvas::new(gfx),
    )?;
    Ok(())
}

/// `text` in `font` at (x, y); the font object written back when the drawing changed it.
pub fn draw_text(
    font: crate::gfx::typefaces::FontId,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    let at = font.object(&gfx.fonts.typefaces);
    let mut copy = at.get();
    crate::text::draw_text::draw_text(
        &mut copy,
        text,
        x as i32,
        y as i32,
        &mut GameCanvas::new(gfx),
    )?;
    if copy != at.get() {
        at.set(copy);
    }
    Ok(())
}

/// `text::measure_text`, at `x`: (the widest line, the last line's y).
pub fn measure_text(
    font: crate::gfx::typefaces::FontId,
    text: &(impl rc2k_core::strings::bytes::Bytes + ?Sized),
    x: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(u32, u32), Refused> {
    use crate::text::draw_text::GameCanvas;
    let m = {
        crate::text::measure_text::measure_text_at(font, text, x as i32, &mut GameCanvas::new(gfx))
    }?;
    Ok((m.width, m.outcome.y as u32))
}

/// `text::measure_string_by_id`, at `x`: (the widest line, the last line's y).
pub fn measure_string(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(u32, u32), Refused> {
    use crate::text::draw_text::GameCanvas;
    let m = {
        crate::text::measure_string_by_id::measure_string_by_id_at(
            so,
            id,
            font,
            x as i32,
            &mut GameCanvas::new(gfx),
        )
    }?;
    Ok((m.width, m.outcome.y as u32))
}

/// 0x100 colours of `from` made pixels of the surface's format into `to`.
pub fn components_to_pixels(
    from: &[crate::session::PlainDword; 256],
    to: &[crate::session::PlainDword; 256],
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) {
    let format = crate::gfx::pixel_format::surface_format(system);
    for (from, to) in from.iter().zip(to) {
        to.set(crate::gfx::components_to_pixel::components_to_pixel(
            &format,
            from.get(),
            gfx,
        ));
    }
}

pub fn load_language(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::strings::load_language::load_language(o, &mut || {
        crate::strings::compile_lexicon::compile_lexicon(o.strings, o.system)
    })
}

/// The key (0: none).
pub fn list_take_key(
    list_state: &crate::menu::list_state::ListState,
    input: &crate::input::owner::Input,
) -> Result<u8, Refused> {
    Ok(crate::menu::list_take_key::list_take_key(list_state, input))
}

/// `key` moving the highlight of `list` by the neighbours table `table`; whether it moved.
pub fn list_step(
    list_state: &crate::menu::list_state::ListState,
    key: u32,
    table: Option<crate::place::BytePlace>,
    list: ListMenu,
) -> Result<bool, crate::errors::BadInput> {
    crate::menu::list_step::list_step(list_state, key as u8, table, list)
}

pub fn list_clear_marks(install: &'static crate::install::owner::Install, list: ListMenu) {
    crate::menu::list_clear_marks::list_clear_marks(Some(install), list);
}

/// Whether the highlighted item cannot be chosen.
pub fn list_item_unusable(list_state: &crate::menu::list_state::ListState, list: ListMenu) -> bool {
    crate::menu::list_item_unusable::list_item_unusable(list_state, list)
}

/// Whether the standalone's install or uninstall ended, logged.
pub fn logged(what: &str, done: Result<(), Refused>) -> Result<(), Refused> {
    crate::framework::log(&format!(
        "install: the {what} ended: {:?}",
        done.as_ref().map(|_| ())
    ));
    done
}

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
    let list_state = &o.menus.list;
    use crate::install as i;
    match place {
        0x414199 => i::language_page::language_page(install, o, state, list)?,
        0x4141EF => i::mark_main_page::mark_main_page(install, list),
        0x41422B => i::mode_page::mode_page(install, list_state, state),
        0x41425E => i::drive_page::drive_page(install, list_state, state, list),
        0x4142C7 => i::room_page::room_page(install, state),
        0x4142F5 => i::folder_page::folder_page(install, list_state, state, list),
        0x41432F => i::dxsetup_page::dxsetup_page(install, list_state, state),
        0x414347 => i::finish_page::finish_page(install, list_state, state),
        0x4130CD => logged("install", i::install_files::install_files(install, o))?,
        0x4134DF => logged("uninstall", i::uninstall::uninstall(install, o))?,
        0x413629 => i::open_installed::open_installed(install)?,
        0x413650 => i::ask_registration::ask_registration(install),
        0x41675B => i::install_clio::install_clio(install, o)?,
        _ => return Err(BadInput("a list action that is not one of the installer's").into()),
    }
    Ok(())
}

/// A list's key action (+0x14): `key`; whether it took the key. The lists hold only the
/// folder page's.
pub fn key_action(
    list_state: &crate::menu::list_state::ListState,
    place: u32,
    key: u32,
    list: ListMenu,
) -> Result<bool, Refused> {
    match place {
        0x41534E => Ok(crate::install::folder_key::folder_key(
            list_state, key, list,
        )),
        _ => Err(BadInput("a key action that is not one of the installer's").into()),
    }
}

/// A link's or an action item's action (+8). The items hold only the not-enough-room page's
/// "go on anyway".
pub fn item_action_at(
    install: &crate::install::owner::Install,
    list_state: &crate::menu::list_state::ListState,
    place: u32,
) -> Result<bool, Refused> {
    match place {
        0x4142DA => {
            crate::install::ignore_room::ignore_room(install, list_state);
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

/// An action item's drawing (+0xC): the lists have no action items.
pub fn draw_action(_place: u32) -> Result<(), Refused> {
    Err(BadInput("an action item's drawing (the game's lists have none)").into())
}
