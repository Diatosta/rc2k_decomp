//! Reading a code from a camera: the camera is opened, its frames are shown as a small live
//! picture and each is looked at for a QR code, until one that holds a save code is found or
//! the player goes back.

use crate::owners::Owners;
use crate::platform::ffi::sdl_camera::{self, Camera, Frame, Permission};
use crate::save_code::page_draw as draw;
use rc2k_core::saves::code::{START, qr};
use rc2k_core::strings::own::id;

const KEY_ESCAPE: u8 = 0x01;
const KEY_ENTER: u8 = 0x1C;

/// How many frames pass between one look for a code and the next (finding one costs more than
/// drawing the picture).
const LOOK_EVERY: u32 = 3;

/// Where frames come from.
enum Source {
    Real(Camera),
    /// A test's: the frame it sets (`remote`).
    #[cfg(feature = "remote")]
    Fake,
}

impl Source {
    fn frame(&mut self) -> Option<Frame> {
        match self {
            Source::Real(camera) => camera.frame(),
            #[cfg(feature = "remote")]
            Source::Fake => crate::remote::fake_camera_frame(),
        }
    }

    fn permission(&self) -> Permission {
        match self {
            Source::Real(camera) => camera.permission(),
            #[cfg(feature = "remote")]
            Source::Fake => Permission::Approved,
        }
    }
}

/// Whether a camera can be asked for: SDL sees one (the camera subsystem is started on the
/// first ask).
pub fn available() -> bool {
    #[cfg(feature = "remote")]
    if crate::remote::fake_camera() {
        return true;
    }
    static SEEN: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *SEEN.get_or_init(|| sdl_camera::init() && !sdl_camera::devices().is_empty())
}

fn source() -> Option<Source> {
    #[cfg(feature = "remote")]
    if crate::remote::fake_camera() {
        return Some(Source::Fake);
    }
    let first = *sdl_camera::devices().first()?;
    sdl_camera::open(first).map(Source::Real)
}

fn grey(frame: &Frame) -> Vec<u8> {
    frame
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8) as u8)
        .collect()
}

/// A save code found in `frame`, if there is one.
pub fn code_in(frame: &Frame) -> Option<String> {
    qr::read(frame.width, frame.height, &grey(frame)).filter(|t| t.trim().starts_with(START))
}

/// Looks through the camera until a save code is read (its text) or the player goes back
/// (`None`), the picture and a line saying what is going on drawn meanwhile. `Err` is the own
/// text that says why there is no looking.
pub fn run(o: Owners<'static>) -> Result<Option<String>, u32> {
    let Some(mut source) = source() else {
        return Err(id::SAVE_CODE_NO_CAMERA);
    };
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    let mut last: Option<Frame> = None;
    let mut frames = 0u32;
    loop {
        crate::platform::pump_messages_now(o);
        if o.system.session.flags.quitting.is_set() {
            return Ok(None);
        }
        if source.permission() == Permission::Denied {
            return Err(id::SAVE_CODE_CAMERA_DENIED);
        }
        if let Some(frame) = source.frame() {
            frames += 1;
            if frames.is_multiple_of(LOOK_EVERY)
                && let Some(text) = code_in(&frame)
            {
                return Ok(Some(text));
            }
            last = Some(frame);
        }
        if o.gfx.locked.surface.get() == 1 {
            crate::platform::unlock_surface(o.gfx);
        }
        if crate::platform::lock_surface(o.gfx, o.system) < 1 {
            return Ok(None);
        }
        draw_scan(o, last.as_ref());
        crate::platform::unlock_surface(o.gfx);
        if !crate::platform::show_frame_now(o) {
            return Ok(None);
        }
        crate::input::menu_pad::take_answer(o.input);
        if matches!(
            crate::input::take_key::take_key(o.input),
            KEY_ESCAPE | KEY_ENTER
        ) {
            return Ok(None);
        }
    }
}

fn draw_scan(o: Owners<'static>, frame: Option<&Frame>) {
    let own = |n| {
        rc2k_core::files::ansi::ansi_text(&crate::strings::append_string_by_id::own_text(
            o.strings, n,
        ))
    };
    let (w, h) = draw::screen_size(o);
    crate::input::osk_draw::fill(o, [0, 0, w, h], draw::SCREEN);
    draw::text(
        o,
        &own(id::SAVE_CODE_TITLE),
        draw::MARGIN,
        draw::MARGIN + draw::LINE_H / 2,
    );
    let room = (w - 2 * draw::MARGIN).min((h - 5 * draw::LINE_H) * 4 / 3);
    let x = (w - room) / 2;
    let y = draw::MARGIN + 2 * draw::LINE_H;
    crate::input::osk_draw::fill(o, [x, y, room, room * 3 / 4], draw::PANEL);
    if let Some(frame) = frame {
        draw::preview(o, frame, x, y, room);
    }
    let measure = |s: &str| draw::width(o, s);
    let top = h - draw::MARGIN - 2 * draw::LINE_H;
    for (n, text) in
        rc2k_core::fatal::wrap_lines(&own(id::SAVE_CODE_LOOKING), w - 2 * draw::MARGIN, &measure)
            .iter()
            .take(2)
            .enumerate()
    {
        draw::centred(
            o,
            text,
            0,
            w,
            top + n as i32 * draw::LINE_H + draw::LINE_H / 2,
        );
    }
}
