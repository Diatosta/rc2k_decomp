//! The 2D drawing of a frame as typed commands. The funnels that write the screen's pixels
//! (`gfx::blit_glyph::blit_glyph_on_surface`, `gfx::rect`, `gfx::copy_picture_to_surface`,
//! `gfx::rescale_to_surface`, `gfx::clear_back_surface`) note what they draw as a `Command` while
//! recording is on, and `replay` draws a frame's commands into a canvas through the same drawing
//! code. Recording is off unless `RC2K_2D_CHECK` is set or a test turns it on: nothing consumes the
//! list yet except the check (`check`), which replays each frame and compares it with what the
//! funnels wrote.

pub mod check;
pub mod replay;

use crate::gfx::blit_glyph::{BlitCall, Scratch};
use rc2k_core::gfx::raster::{Clip, Surface};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// What a command draws on: the screen's bytes, laid out by `surface`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Canvas {
    pub surface: Surface,
    pub clip: Clip,
}

/// One glyph or sprite blit.
#[derive(Clone, Debug)]
pub struct Glyph {
    pub call: BlitCall,
    pub canvas: Canvas,
    /// The palette as it was when the glyph was drawn: an index into `Frame::palettes`; none for
    /// a sprite.
    pub palette: Option<usize>,
    /// The blitter's work variables before and after the blit.
    pub before: Scratch,
    pub after: Scratch,
}

/// The shape of a solid drawing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Fill([i32; 4]),
    HLine { x: i32, y: i32, length: i32 },
    Frame { rect: [i32; 4], dx: i32, dy: i32 },
}

/// A solid rectangle, outline or line.
#[derive(Clone, Debug)]
pub struct Rect {
    pub shape: Shape,
    pub canvas: Canvas,
    /// The pixel's bytes at the surface's depth: empty when the shape sets no pixel.
    pub pixel: Vec<u8>,
}

/// The picture buffer copied over the screen, row by row.
#[derive(Clone, Copy, Debug)]
pub struct Background {
    pub buffer: u32,
    /// The picture buffer's change count when the picture was drawn.
    pub generation: u64,
    pub rows: u32,
    pub row_bytes: usize,
    pub pitch: u32,
}

/// Rows of pixels written as they are (the loading picture rescaled to the screen).
#[derive(Clone, Debug)]
pub struct Image {
    pub row_len: usize,
    pub pitch: u32,
    pub pixels: Vec<u8>,
}

/// The screen's rows zeroed.
#[derive(Clone, Copy, Debug)]
pub struct Clear {
    pub row_len: usize,
    pub pitch: u32,
    pub rows: u32,
}

#[derive(Clone, Debug)]
pub enum Command {
    Glyph(Glyph),
    Rect(Rect),
    Background(Background),
    Image(Image),
    Clear(Clear),
}

/// A frame's commands, in the order they were drawn, with the palettes they used.
#[derive(Default, Debug)]
pub struct Frame {
    pub commands: Vec<Command>,
    /// The dwords of each palette state a glyph drew with (up to 256).
    pub palettes: Vec<Vec<u8>>,
}

struct Recorder {
    frame: Frame,
    /// Each palette's latest state in `frame.palettes`, by the palette's handle.
    latest: std::collections::BTreeMap<u32, usize>,
    /// What the funnels wrote on the screen this frame, byte for byte (0xAA where nothing).
    written: Vec<u8>,
    /// A write past the screen's bytes.
    outside: bool,
}

/// What `written` holds where nothing was written.
pub const UNWRITTEN: u8 = 0xAA;

static ENABLED: AtomicBool = AtomicBool::new(false);

static RECORDER: Mutex<Recorder> = Mutex::new(Recorder {
    frame: Frame {
        commands: Vec::new(),
        palettes: Vec::new(),
    },
    latest: std::collections::BTreeMap::new(),
    written: Vec::new(),
    outside: false,
});

fn recorder() -> std::sync::MutexGuard<'static, Recorder> {
    RECORDER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The picture buffer's change count; the handheld's build keeps none, so it notes no background.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn picture_generation(handle: u32) -> Option<u64> {
    crate::gfx::picture::generation(handle)
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn picture_generation(_handle: u32) -> Option<u64> {
    None
}

/// Whether the funnels note their drawing.
pub fn enabled() -> bool {
    ENABLED.load(Relaxed)
}

/// Turns recording on (off with `false`).
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Relaxed);
}

/// Turns recording on when `RC2K_2D_CHECK` is set.
pub fn enable_from_environment() {
    if std::env::var_os("RC2K_2D_CHECK").is_some() {
        set_enabled(true);
    }
}

/// Whether drawing on `target` is the screen's: the only target recorded.
pub fn is_screen(target: Option<crate::gfx::surface::Target>) -> bool {
    target
        == Some(crate::gfx::surface::Target::Locked(
            crate::gfx::locked_pixels::SCREEN,
        ))
}

impl Recorder {
    /// `written` made as long as the screen's bytes.
    fn make_written(&mut self) {
        if self.written.is_empty() {
            match crate::gfx::locked_pixels::bytes_of(crate::gfx::locked_pixels::SCREEN) {
                Some(screen) => self.written = vec![UNWRITTEN; screen.len()],
                None => self.outside = true,
            }
        }
    }
}

/// Notes a write of the funnels on the screen: `bytes` at `offset` from its start.
pub fn mirror(offset: u32, bytes: &[u8]) {
    let mut rec = recorder();
    rec.make_written();
    let at = offset as usize;
    match rec.written.get_mut(at..at.wrapping_add(bytes.len())) {
        Some(place) => place.copy_from_slice(bytes),
        None => rec.outside = true,
    }
}

/// Notes a command.
pub fn push(command: Command) {
    let mut rec = recorder();
    rec.make_written();
    rec.frame.commands.push(command);
}

/// Palette `handle` as its 256 dwords are now, as a state of the frame: the index in
/// `Frame::palettes`; none when it cannot be read.
pub fn palette_state(fonts: &crate::gfx::owner::FontStores, handle: u32) -> Option<usize> {
    // A palette is up to 256 dwords; a font's own may be shorter.
    let bytes = crate::gfx::glyph_store::read_bytes(fonts, handle, 0, 1024, "a palette")
        .or_else(|_| {
            let held = crate::gfx::glyph_store::bytes(fonts, handle, "a palette")?;
            Ok::<_, crate::errors::BadInput>(
                (0..1024)
                    .map_while(|at| rc2k_core::strings::bytes::Bytes::byte(&held, at))
                    .collect(),
            )
        })
        .ok()?;
    let mut rec = recorder();
    if let Some(&at) = rec.latest.get(&handle)
        && rec.frame.palettes[at] == bytes
    {
        return Some(at);
    }
    rec.frame.palettes.push(bytes);
    let at = rec.frame.palettes.len() - 1;
    rec.latest.insert(handle, at);
    Some(at)
}

/// The frame so far and what the funnels wrote, taken: the next frame starts empty.
pub fn take_frame() -> (Frame, Vec<u8>, bool) {
    let mut rec = recorder();
    let frame = std::mem::take(&mut rec.frame);
    rec.latest.clear();
    let outside = std::mem::take(&mut rec.outside);
    let written = std::mem::take(&mut rec.written);
    (frame, written, outside)
}
