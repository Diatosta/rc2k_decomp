//! Reading a code from a camera: the camera is opened, its frames are shown as a small live
//! picture and each is looked at for a QR code, until a whole save code is read or the player
//! goes back. A code in several parts is read part by part in any order: the page says how many
//! are read and which, and the whole code is given once every part is in.

use crate::owners::Owners;
use crate::platform::ffi::sdl_camera::{self, Camera, Frame, Permission};
use crate::save_code::page_draw as draw;
use rc2k_core::saves::code::parts::{self, Assembly, Part, Taken};
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

/// A save code's text (the whole code or one of its parts) found in `frame`, if there is one.
pub fn code_in(frame: &Frame) -> Option<String> {
    qr::read(frame.width, frame.height, &grey(frame)).filter(|t| {
        let t = t.trim();
        t.starts_with(START) || t.starts_with(parts::START)
    })
}

/// What reading a frame's code did to the parts held.
enum Read {
    /// A whole code, or the last part: the whole text.
    Whole(String),
    /// A part of the code being collected, taken in.
    Taken,
    /// Something to say (the own text).
    Note(u32),
    Nothing,
}

/// Takes what `text` is into `held`.
fn take(held: &mut Assembly, text: &str) -> Read {
    let text = text.trim();
    if text.starts_with(START) {
        return Read::Whole(text.to_owned());
    }
    let Some(part) = Part::read(text) else {
        return Read::Nothing;
    };
    match held.take(part) {
        Taken::Other => return Read::Note(id::SAVE_CODE_PART_OTHER),
        Taken::Again => return Read::Nothing,
        Taken::New => {}
    }
    match held.whole() {
        None => Read::Taken,
        Some(Ok(whole)) => Read::Whole(whole),
        // The parts do not make the code their check number names: start over.
        Some(Err(_)) => {
            *held = Assembly::default();
            Read::Nothing
        }
    }
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
    let mut held = Assembly::default();
    let mut note = None;
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
                match take(&mut held, &text) {
                    Read::Whole(whole) => return Ok(Some(whole)),
                    Read::Taken => note = None,
                    Read::Note(n) => note = Some(n),
                    Read::Nothing => {}
                }
            }
            last = Some(frame);
        }
        if o.gfx.locked.surface.get() == 1 {
            crate::platform::unlock_surface(o.gfx);
        }
        if crate::platform::lock_surface(o.gfx, o.system) < 1 {
            return Ok(None);
        }
        draw_scan(o, last.as_ref(), &held, note);
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

fn draw_scan(o: Owners<'static>, frame: Option<&Frame>, held: &Assembly, note: Option<u32>) {
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
    // The picture's box: the frame's own proportions (4:3 before the first frame), as large as
    // the screen leaves room for between the title and the notes.
    let (across, down) = (w - 2 * draw::MARGIN, h - 6 * draw::LINE_H);
    let (fw, fh) = frame.map_or((4, 3), |f| (f.width as i32, f.height as i32));
    let room = across.min(down * fw / fh.max(1)).max(1);
    let x = (w - room) / 2;
    let y = draw::MARGIN + 2 * draw::LINE_H;
    crate::input::osk_draw::fill(o, [x, y, room, room * fh / fw.max(1)], draw::PANEL);
    if let Some(frame) = frame {
        draw::preview(o, frame, x, y, room);
    }
    // Under the title: which parts of a code in parts are read, or that a part is another code's.
    if held.count() > 0 {
        let numbers: Vec<String> = held.numbers().iter().map(usize::to_string).collect();
        let progress = format!(
            "{}.  {}",
            own(id::SAVE_CODE_PARTS_READ)
                .replacen("{}", &numbers.len().to_string(), 1)
                .replacen("{}", &held.count().to_string(), 1),
            own(id::SAVE_CODE_PARTS_HELD).replace("{}", &numbers.join(", "))
        );
        draw::centred(
            o,
            &draw::fit(o, &progress, w - 2 * draw::MARGIN),
            0,
            w,
            draw::MARGIN + draw::LINE_H + draw::LINE_H / 2,
        );
    }
    let measure = |s: &str| draw::width(o, s);
    let top = h - draw::MARGIN - 3 * draw::LINE_H;
    let said = note.map(own).unwrap_or_else(|| own(id::SAVE_CODE_LOOKING));
    for (n, text) in rc2k_core::fatal::wrap_lines(&said, w - 2 * draw::MARGIN, &measure)
        .iter()
        .take(3)
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
