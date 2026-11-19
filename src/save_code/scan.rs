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
const KEY_TAB: u8 = 0x0F;
const KEY_LEFT: u8 = 0x68;
const KEY_RIGHT: u8 = 0x69;

/// Where frames come from.
enum Source {
    /// An open camera, by its place in the list of cameras.
    Real(Camera, usize),
    /// A test's: the frame it sets (`remote`), as stand-in camera number so many.
    #[cfg(feature = "remote")]
    Fake(usize),
}

impl Source {
    fn frame(&mut self) -> Option<Frame> {
        match self {
            Source::Real(camera, _) => camera.frame(),
            #[cfg(feature = "remote")]
            Source::Fake(n) => crate::remote::fake_camera_frame().map(|mut f| {
                f.facing_user |= *n % 2 == 1;
                f
            }),
        }
    }

    fn permission(&self) -> Permission {
        match self {
            Source::Real(camera, _) => camera.permission(),
            #[cfg(feature = "remote")]
            Source::Fake(_) => Permission::Approved,
        }
    }

    /// Its place among the cameras.
    fn number(&self) -> usize {
        match self {
            Source::Real(_, n) => *n,
            #[cfg(feature = "remote")]
            Source::Fake(n) => *n,
        }
    }
}

/// How many cameras there are to choose from.
fn camera_count() -> usize {
    #[cfg(feature = "remote")]
    if crate::remote::fake_camera() {
        return crate::remote::fake_camera_count();
    }
    sdl_camera::devices().len()
}

/// The name of camera number `n`.
fn camera_name(n: usize) -> String {
    #[cfg(feature = "remote")]
    if crate::remote::fake_camera() {
        return format!("Test camera {}", n + 1);
    }
    sdl_camera::devices()
        .get(n)
        .map(|&id| sdl_camera::name(id))
        .unwrap_or_default()
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

/// Opens camera number `n` (the first is a back-facing one when there is one).
fn open_camera(n: usize) -> Option<Source> {
    #[cfg(feature = "remote")]
    if crate::remote::fake_camera() {
        return Some(Source::Fake(n));
    }
    let id = *sdl_camera::devices().get(n)?;
    sdl_camera::open(id).map(|camera| Source::Real(camera, n))
}

/// A save code's text (the whole code or one of its parts) found in a frame of grey levels, if
/// there is one, however the camera turned or mirrored the picture.
fn code_in(width: usize, height: usize, grey: &[u8]) -> Option<String> {
    qr::read_camera(width, height, grey)
        .map(|(text, _)| text)
        .filter(|t| {
            let t = t.trim();
            t.starts_with(START) || t.starts_with(parts::START)
        })
}

/// Looks for codes in frames on a thread of its own (finding one in a large picture takes
/// tenths of a second, which would stall the page): one frame at a time, the ones that come
/// while it works are not looked at.
struct Looker {
    frames: std::sync::mpsc::SyncSender<(usize, usize, Vec<u8>)>,
    found: std::sync::mpsc::Receiver<Option<String>>,
    busy: bool,
}

impl Looker {
    fn start() -> Self {
        let (frames, wanted) = std::sync::mpsc::sync_channel::<(usize, usize, Vec<u8>)>(1);
        let (sender, found) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            // Ends when the page drops `frames`.
            while let Ok((width, height, grey)) = wanted.recv() {
                if sender.send(code_in(width, height, &grey)).is_err() {
                    break;
                }
            }
        });
        Self {
            frames,
            found,
            busy: false,
        }
    }

    /// Whether it is free for another frame.
    fn idle(&self) -> bool {
        !self.busy
    }

    /// Hands it a frame to look at.
    fn look(&mut self, frame: &Frame) {
        let grey = rc2k_core::gfx::camera_frame::grey_of(&frame.rgba);
        if self
            .frames
            .try_send((frame.width, frame.height, grey))
            .is_ok()
        {
            self.busy = true;
        }
    }

    /// What it found since, if it has finished: a code's text.
    fn result(&mut self) -> Option<String> {
        let found = self.found.try_recv().ok()?;
        self.busy = false;
        found
    }
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
/// (`None`), the picture and a line saying what is going on drawn meanwhile. Left and Right (a
/// pad's D-pad and shoulder buttons, a touch screen's slides sideways, which the page's
/// `DirectSideways` is still holding) choose another camera. `Err` is the own text that
/// says why there is no looking.
pub fn run(o: Owners<'static>) -> Result<Option<String>, u32> {
    let count = camera_count();
    let Some(mut source) = open_camera(0) else {
        return Err(id::SAVE_CODE_NO_CAMERA);
    };
    crate::input::take_key::take_key(o.input);
    crate::input::menu_pad::take_answer(o.input);
    // The page that opened this one is taking the taps (`platform::touch_menu`): a tap here
    // leaves, as Enter does.
    #[cfg(touch_screen)]
    let _taps = crate::platform::touch_menu::InstallerPage::begin();
    let mut last: Option<Frame> = None;
    let mut looker = Looker::start();
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
            if looker.idle() {
                looker.look(&frame);
            }
            last = Some(frame);
        }
        if let Some(text) = looker.result() {
            match take(&mut held, &text) {
                Read::Whole(whole) => return Ok(Some(whole)),
                Read::Taken => note = None,
                Read::Note(n) => note = Some(n),
                Read::Nothing => {}
            }
        }
        if o.gfx.locked.surface.get() == 1 {
            crate::platform::unlock_surface(o.gfx);
        }
        if crate::platform::lock_surface(o.gfx, o.system) < 1 {
            return Ok(None);
        }
        let camera = (source.number(), count);
        draw_scan(o, last.as_ref(), &held, note, camera);
        crate::platform::unlock_surface(o.gfx);
        if !crate::platform::show_frame_now(o) {
            return Ok(None);
        }
        crate::input::menu_pad::take_answer(o.input);
        #[cfg(touch_screen)]
        if crate::platform::touch_menu::take_installer_tap().is_some() {
            return Ok(None);
        }
        let step = match crate::input::take_key::take_key(o.input) {
            KEY_ESCAPE | KEY_ENTER => return Ok(None),
            KEY_LEFT => count - 1,
            KEY_RIGHT | KEY_TAB => 1,
            _ => 0,
        };
        if step != 0 && count > 1 {
            // The next camera that opens, the way round from this one (this one again when
            // no other does).
            let from = source.number();
            // Closed before the next one opens (a camera is held by one user at a time); the
            // handheld's stand-in camera holds nothing to close.
            #[allow(
                clippy::drop_non_drop,
                reason = "a real camera closes on drop; the handheld's stand-in has no drop"
            )]
            drop(source);
            let Some(next) = (1..=count)
                .map(|k| (from + k * step) % count)
                .find_map(open_camera)
            else {
                return Err(id::SAVE_CODE_NO_CAMERA);
            };
            source = next;
            // The old camera's last picture and what it was being looked at for are gone.
            last = None;
        }
    }
}

/// Draws the scan screen; `camera` is the number of the camera in use and how many there are.
fn draw_scan(
    o: Owners<'static>,
    frame: Option<&Frame>,
    held: &Assembly,
    note: Option<u32>,
    camera: (usize, usize),
) {
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
    let (across, down) = (w - 2 * draw::MARGIN, h - 8 * draw::LINE_H - draw::MARGIN);
    let (fw, fh) = frame.map_or((4, 3), |f| (f.width as i32, f.height as i32));
    let room = across.min(down * fw / fh.max(1)).max(1);
    let x = (w - room) / 2;
    let y = draw::MARGIN + 2 * draw::LINE_H;
    crate::input::osk_draw::fill(o, [x, y, room, room * fh / fw.max(1)], draw::PANEL);
    if let Some(frame) = frame {
        draw::preview(o, frame, [x, y], room);
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
    // Above the notes: which camera this is, and the way to choose another.
    let (number, count) = camera;
    let name = own(id::SAVE_CODE_CAMERA_NAME)
        .replacen("{}", &(number + 1).to_string(), 1)
        .replacen("{}", &count.to_string(), 1)
        .replacen("{}", &camera_name(number), 1);
    let mut line = top - 2 * draw::LINE_H;
    draw::centred(
        o,
        &draw::fit(o, &name, w - 2 * draw::MARGIN),
        0,
        w,
        line + draw::LINE_H / 2,
    );
    if count > 1 {
        // On a touch screen the page's sideways slides give Left and Right (`save_code::page`).
        let switch_hint = if cfg!(touch_screen) {
            id::SAVE_CODE_CAMERA_SWITCH_TOUCH
        } else {
            id::SAVE_CODE_CAMERA_SWITCH
        };
        line += draw::LINE_H;
        draw::centred(
            o,
            &draw::fit(o, &own(switch_hint), w - 2 * draw::MARGIN),
            0,
            w,
            line + draw::LINE_H / 2,
        );
    }
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
