//! The installer (`install::run_installer` and its tree): the variables its parts share, the calls
//! they make and the functions its page tables hold.
//!
//! The installer runs only with its script set up (`-u`:
//! `install::check_at_start`). Its pages are `rc2k_core::install::model` pages
//! (`install::run_page`); `install::run_pages` goes from page to page by the answer.

use crate::errors::{FatalError, Refused};
use crate::memory::place::BytePlace;

/// The installer's constant texts.
#[derive(rc2k_derive::Cells)]
pub struct InstallTexts {
    /// ".".
    pub dot: crate::memory::session::Bytes<2>,
    /// "..".
    pub dotdot: crate::memory::session::Bytes<3>,
    /// "*.*".
    pub star: crate::memory::session::Bytes<4>,
    /// The installation's record.
    pub record_path: crate::memory::session::Bytes<12>,
}

/// The installer's texts.
impl InstallTexts {
    pub const fn new() -> Self {
        InstallTexts {
            dot: crate::memory::session::Bytes::new(*b".\0"),
            dotdot: crate::memory::session::Bytes::new(*b"..\0"),
            star: crate::memory::session::Bytes::new(*b"*.*\0"),
            record_path: crate::memory::session::Bytes::new(*b"var\\ral.zog\0"),
        }
    }
}

// The base and locale directories: `files::state::{base_directory, locale_directory}`.

/// The media mode of a full installation: every file is in the installed folder.
pub(super) const FULL_INSTALLATION: u32 = 3;

/// The installation is a full one (the only kind this installer makes): the game reads the
/// media mode (`files::state::MEDIA_MODE`) to know where its files are.
pub(super) fn set_full_installation() {
    crate::files::state::MEDIA_MODE.set(FULL_INSTALLATION);
}

// The installer's other variables (the update switch, the remove-items switch, the install.bun
// flag, the room by installation, the list variables, the third page's action, the file
// search record and its skip mask) are in `install::vars`.

/// A read or write of the installer's code is refused: the program ends naming it (the
/// script and its pages have no way to carry the refusal back).
pub fn refuse(e: crate::errors::BadInput) -> ! {
    crate::fatal::raise::end("install", e)
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

/// `bytes`, the whole text a builder made (its NULs too), written at `at`.
pub fn store_text(at: impl Into<BytePlace>, bytes: &[u8]) {
    at.into()
        .write(bytes, "an installer text")
        .unwrap_or_else(|e| refuse(e));
}

/// `strings::append_string_by_id`: string `id` appended to `out`.
pub(super) fn push_string_by_id(
    o: crate::strings::append_string_by_id::StringOwners<'static>,
    out: &mut Vec<u8>,
    id: u32,
) {
    let pushed = crate::strings::append_string_by_id::push_string(o, out, id);
    crate::menu::event_flow::refuse_here(pushed, "install::installer: a string");
}

/// Makes `path` the current directory; whether it worked.
pub(super) fn set_cd(path: &[u8]) -> bool {
    crate::files::directories::set_current_directory(path)
}

/// `files::remove_directory` (remove a directory): whether it worked.
pub fn remove_dir(path: &[u8]) -> bool {
    crate::files::directories::remove_directory(path)
}

/// Deletes the file `path`; whether it worked.
pub fn delete_file(path: &[u8]) -> bool {
    crate::files::directories::delete_file(path)
}

/// `path` in the installer's one form of a folder (with its NUL); None when the path is not
/// taken.
pub fn normalise(path: &[u8]) -> Result<Option<Vec<u8>>, Refused> {
    Ok(crate::install::normalise_path::normalised_bytes(
        path,
        &mut || crate::install::normalise_path::current_directory(),
    )?)
}

/// Copies the file `from` to `to`; whether it worked.
pub fn copy_file(from: &[u8], to: &[u8]) -> Result<bool, Refused> {
    Ok(crate::files::copy_file::copy_file(from, to)?)
}

/// The path the game opens (without its NUL).
pub fn resolve_path(path: &[u8]) -> Result<Vec<u8>, Refused> {
    Ok(crate::files::resolve_path::resolve(path)?)
}

/// Starts a file search on `pattern`, passing over the `skipped` attributes; whether a file was
/// found.
pub fn find_first(pattern: &[u8], skipped: u32) -> bool {
    crate::race::find_snapshot_number::find_first(pattern, skipped)
}

/// The search's next file not of the `skipped` attributes; whether one was found.
pub(super) fn find_next(skipped: u32) -> bool {
    crate::race::find_snapshot_number::find_next_wanted(skipped)
}

pub fn close_search() {
    crate::files::search::close();
}

/// A pause that a key ends, the window's events going on.
pub fn pause(o: crate::owners::Owners<'static>, milliseconds: u32) {
    crate::system::pause::pause(o, milliseconds);
}

/// A pause that keys do not end, the window's events going on.
pub fn hold(o: crate::owners::Owners<'static>, milliseconds: u32) {
    crate::system::pause::hold(o, milliseconds);
}

pub fn sleep(milliseconds: u32) {
    crate::clock::sleep::sleep(milliseconds);
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
    crate::shell::frame::show_frame_now(o)
}

pub fn pump(o: crate::owners::Owners<'static>) {
    crate::shell::frame::pump_messages_now(o);
}

/// A frame of the installer's for `screens::run_frame`; true when it failed.
pub type Frame<'a> = &'a dyn Fn() -> Result<bool, Refused>;

/// The window's messages, then (unless quitting) the frame; true when the frame failed.
pub fn run_frame(
    o: crate::owners::Owners<'static>,
    frame: Option<Frame<'_>>,
) -> Result<bool, Refused> {
    // The platform's run_frame with no frame of its own: the window's messages only.
    crate::shell::frame::run_frame_now(o, || ());
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
pub(super) fn components_to_pixels(
    from: &[crate::memory::session::PlainDword; 256],
    to: &[crate::memory::session::PlainDword; 256],
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
    crate::language::load_language::load_language(o, &mut || {
        crate::language::compile_lexicon::compile_lexicon(o.strings, &o.system.work().files)
    })
}

/// The key (0: none).
pub fn list_take_key(
    list_state: &crate::menu::list_state::ListState,
    input: &crate::input::owner::Input,
) -> Result<u8, Refused> {
    Ok(crate::menu::list_take_key::list_take_key(list_state, input))
}

/// How the install or uninstall ended, logged.
pub fn logged<T>(what: &str, done: Result<T, Refused>) -> Result<T, Refused> {
    crate::program_log::log(&format!(
        "install: the {what} ended: {:?}",
        done.as_ref().map(|_| ())
    ));
    done
}
