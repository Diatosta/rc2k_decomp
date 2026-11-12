//! The renderer's boundary (GPU plan, `notes/improvements.md`): the frame's 3D as commands the
//! game's Direct3D layer (`platform_3d`) notes, and a `Backend` that draws them. Everything the
//! backends must agree on is here; each one's own way of drawing is in its file.

#![allow(unsafe_code)]

pub mod batches;
pub mod dump;
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub mod gles;
// The handheld has no Vulkan: its GPU is reached through `gles`, by the same name here.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub use gles as gpu;
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub mod gpu;
pub mod software;

use crate::race::owner_course::Course;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// Direct3D's render states a command carries.
pub const STATES: usize = 0x40;

/// A texture level of the Direct3D layer (`platform_3d`): the number of its surface and a
/// serial no other level, past or present, has. A level let go and made again under the same
/// surface number is another id, so what a cache holds under the old one is never taken for it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TextureId {
    pub surface: u32,
    pub serial: u32,
}

/// A texture's first level as noted: which it is and how it is laid out.
#[derive(Clone, Copy)]
pub struct TextureNote {
    /// The level, whose `len` bytes `platform_3d::level_bytes` hands out.
    pub pixels: TextureId,
    pub len: usize,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bytes: usize,
    pub masks: [u32; 4],
}

// A fan is by far the common command and its vertices live inline to spare an allocation each.
#[allow(clippy::large_enum_variant)]
pub enum Command {
    Fan {
        vertices: smallvec::SmallVec<[crate::rasteriser::Vertex; 6]>,
        /// Shared between fans with the same states, which most of a frame's are.
        states: std::sync::Arc<[u32; STATES]>,
        texture: Option<TextureNote>,
        /// The vertices' bounds on the screen: (left, top, right, bottom).
        bounds: (f32, f32, f32, f32),
    },
    /// The depth buffer back to 1.0 in a rectangle (left, top, right, bottom).
    Clear([usize; 4]),
}

/// What the commands draw into.
#[derive(Clone, Copy, PartialEq)]
pub struct Target {
    /// The handle of the pixels (`gfx::locked_pixels`).
    pub pixels: u32,
    pub pitch: usize,
    pub width: usize,
    pub height: usize,
    pub bits: u32,
}

/// A way of drawing the commands. It draws them all, in order, before it returns: the game's
/// code may read or change the target and the textures as soon as it has.
pub trait Backend: Send {
    fn draw(&mut self, target: &Target, commands: Vec<Command>);
}

/// Whether the GPU holds the frame: a race's on the GPU backend (the menus stay the rasteriser's:
/// they draw little 3D, and read the screen's memory).
static GPU_HOLDS_FRAME: AtomicBool = AtomicBool::new(false);

/// Draws the commands on the backend that holds the frame; `screen` is the screen's size (the
/// rasteriser's depth buffer is kept at it).
pub fn draw(in_race: bool, screen: (usize, usize), target: &Target, commands: Vec<Command>) {
    if target.bits == 32 && in_race {
        let mut gpu = gpu::GPU.lock().unwrap();
        if let Some(gpu) = gpu.as_mut() {
            // On the first one, the picture so far goes over as written pixels: the canvas is
            // whole.
            GPU_HOLDS_FRAME.store(true, Relaxed);
            Backend::draw(gpu, target, commands);
            return;
        }
    }
    give_frame_back(target);
    software::Software { screen }.draw(target, commands);
}

/// The GPU's picture back in the canvas, if it holds the frame and something wants the true one.
pub fn bring_back(target: &Target) {
    if GPU_HOLDS_FRAME.load(Relaxed)
        && let Some(gpu) = gpu::GPU.lock().unwrap().as_mut()
    {
        // SAFETY: the game's code waits; nothing else writes the canvas meanwhile.
        let canvas = unsafe { target.bytes() };
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        gpu.bring_back(canvas);
        #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
        if let Err(why) = gpu.bring_back(canvas) {
            crate::framework::log(&format!("render: the frame did not come back: {why}"));
        }
    }
}

/// The frame back for good: the rasteriser draws from here.
fn give_frame_back(target: &Target) {
    bring_back(target);
    GPU_HOLDS_FRAME.store(false, Relaxed);
}

/// The frame's end, before it is shown. The GPU's colour target when that is what to show, the
/// canvas's written pixels over it, with the number of its set of targets (`Gpu::colour_texture`).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn finish_frame(
    course: &Course,
    target: &Target,
) -> Option<(*mut sdl3_sys::everything::SDL_GPUTexture, u64)> {
    if !GPU_HOLDS_FRAME.load(Relaxed) {
        return None;
    }
    if !course.numbers.get().in_race.is_set() {
        give_frame_back(target);
        return None;
    }
    let mut gpu = gpu::GPU.lock().unwrap();
    let gpu = gpu.as_mut()?;
    // SAFETY: the game's code waits; nothing else writes the canvas meanwhile.
    if let Err(why) = gpu.finish_frame(unsafe { target.bytes() }) {
        crate::framework::log(&format!("render: the frame did not finish: {why}"));
    }
    Some(gpu.colour_texture())
}

/// The frame on the panel, by the GPU when it is running: the target during a race, else
/// `frame`, the canvas's picture. False: it is not running.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn show_on_panel(
    course: &Course,
    target: &Target,
    frame: &[u8],
    width: usize,
    height: usize,
) -> bool {
    gpu::start_if_wanted();
    if GPU_HOLDS_FRAME.load(Relaxed) && !course.numbers.get().in_race.is_set() {
        give_frame_back(target);
    }
    let mut gpu = gpu::GPU.lock().unwrap();
    let Some(gpu) = gpu.as_mut() else {
        return false;
    };
    if GPU_HOLDS_FRAME.load(Relaxed) {
        // SAFETY: the game's code waits; nothing else writes the canvas meanwhile.
        gpu.finish_and_show(unsafe { target.bytes() });
    } else {
        gpu.show_picture(frame, width, height);
    }
    true
}

// ---- textures' changes ----

/// The textures changed since a GPU last asked, by their ids.
#[derive(Default)]
pub(crate) struct TextureChanges {
    /// Written: what a GPU holds of them is to be brought up again.
    pub written: HashSet<TextureId>,
    /// Gone (their memory given back): what a GPU holds of them is to be let go.
    pub gone: HashSet<TextureId>,
}

impl TextureChanges {
    fn note(&mut self, pixels: TextureId, gone: bool) {
        if gone {
            self.written.remove(&pixels);
            self.gone.insert(pixels);
        } else if !self.gone.contains(&pixels) {
            self.written.insert(pixels);
        }
    }
}

static CHANGED_TEXTURES: Mutex<Option<TextureChanges>> = Mutex::new(None);

/// Whether a GPU backend has started: only then are the textures' changes kept (nothing
/// else asks for them).
static GPU_STARTED: AtomicBool = AtomicBool::new(false);

/// A GPU backend has started and keeps textures: their changes are kept from now on.
pub(crate) fn gpu_started() {
    GPU_STARTED.store(true, Relaxed);
}

fn note_texture_change(pixels: TextureId, gone: bool) {
    if GPU_STARTED.load(Relaxed) {
        CHANGED_TEXTURES
            .lock()
            .unwrap()
            .get_or_insert_default()
            .note(pixels, gone);
    }
}

/// The texture `pixels` is being written.
pub fn texture_written(pixels: TextureId) {
    note_texture_change(pixels, false);
}

/// The texture `pixels` is gone: its memory has been given back.
pub fn texture_gone(pixels: TextureId) {
    note_texture_change(pixels, true);
}

pub(crate) fn take_changed_textures() -> TextureChanges {
    CHANGED_TEXTURES.lock().unwrap().take().unwrap_or_default()
}

/// A noted texture's bytes (`note.len` of them). A level that is gone while a fan still names it
/// ends the program.
pub(crate) fn noted_bytes<'a>(note: &TextureNote) -> &'a [u8] {
    // SAFETY: as the rasteriser's read: a noted texture is not written while it is drawn from
    // (locking it flushes the noted fans first).
    unsafe { crate::platform_3d::level_bytes(note.pixels) }.unwrap_or_else(|| {
        crate::fatal::raise::end(
            "a texture's pixels",
            crate::errors::BadInput("a noted texture's level is gone"),
        )
    })
}

/// A texture's pixels as bytes of 0xAARRGGBB, its rows without gaps.
pub(crate) fn texture_as_bgra(note: &TextureNote) -> Vec<u8> {
    let pixels = noted_bytes(note);
    let mut out = Vec::with_capacity(note.width * note.height * 4);
    for y in 0..note.height {
        for x in 0..note.width {
            let at = y * note.pitch + x * note.bytes;
            let mut raw = [0u8; 4];
            if let Some(from) = pixels.get(at..at + note.bytes) {
                raw[..note.bytes].copy_from_slice(from);
            }
            let argb = crate::rasteriser::to_argb(u32::from_le_bytes(raw), &note.masks);
            out.extend(argb.to_le_bytes());
        }
    }
    out
}

impl Target {
    /// `f` on the screen's rows: for the backends, which draw while the game's code waits for
    /// the call (nothing else writes the canvas meanwhile).
    pub fn with_canvas<R>(&self, f: impl FnOnce(&mut [u8]) -> R) -> R {
        // SAFETY: the game's code waits for the backend's call; the slice lives through `f` only.
        f(unsafe { self.bytes() })
    }

    /// The screen's rows as bytes.
    ///
    /// # Safety
    /// Nothing else may touch them while the slice lives.
    pub unsafe fn bytes<'a>(&self) -> &'a mut [u8] {
        let len = self.pitch * self.height;
        let held = crate::gfx::locked_pixels::bytes_of(self.pixels).unwrap_or_else(|| {
            crate::fatal::raise::end(
                "the screen's pixels",
                crate::errors::BadInput("a screen with no pixels"),
            )
        });
        if len > held.len() {
            crate::fatal::raise::end(
                "the screen's pixels",
                crate::errors::BadInput("the screen's rows past its pixels"),
            );
        }
        // SAFETY: the pixels' bytes (the platform keeps them as long as the screen), `len`
        // of them checked above; the caller's promise.
        unsafe { std::slice::from_raw_parts_mut(held.as_mut_ptr(), len) }
    }

    /// The screen's pixels for the rasteriser, rows `pitch / (bits / 8)` pixels apart: its
    /// bytes (`bytes`) as 16- or 32-bit pixels. A screen not aligned to its pixels, or whose
    /// pitch is not whole pixels, ends the program.
    ///
    /// # Safety
    /// As `bytes`.
    pub unsafe fn pixels<'a>(&self) -> crate::rasteriser::Pixels<'a> {
        let size = (self.bits / 8) as usize;
        // SAFETY: the caller's promise.
        let bytes = unsafe { self.bytes() };
        let at = bytes.as_mut_ptr();
        assert!(
            (size == 2 || size == 4)
                && (at as usize).is_multiple_of(size)
                && self.pitch.is_multiple_of(size),
            "the screen's pixels not whole aligned pixels"
        );
        let len = bytes.len() / size;
        // SAFETY: the same bytes (borrowed for 'a with them), aligned and whole pixels (checked
        // above); every bit pattern is a pixel.
        unsafe {
            if size == 2 {
                crate::rasteriser::Pixels::Bits16(std::slice::from_raw_parts_mut(at.cast(), len))
            } else {
                crate::rasteriser::Pixels::Bits32(std::slice::from_raw_parts_mut(at.cast(), len))
            }
        }
    }
}

/// For the standalone's timing line (`platform::show`): fans drawn and the microseconds
/// spent drawing them (at the flushes).
pub static FANS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static RASTER_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The flushes that drew something and the pixels written (for `platform::report_timing`).
/// The parallel drawing's wall time and the threads' time inside the tiles (summed), in
/// microseconds: how well the tiles keep the threads busy.
pub static PARALLEL_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static TILE_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static FLUSHES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static PIXELS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The GPU's frames read back into the canvas, and the textures (and their bytes) sent up to
/// it (for `platform::report_timing`): the costs that vary along a stage on the handheld.
pub static READ_BACKS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static UPLOADS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static UPLOAD_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// The GPU's draw calls (a run of triangles drawn alike each): what the handheld's driver
/// spends its time on.
pub static GPU_DRAWS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// The GL thread's time at its jobs, and the game's waiting to hand it one (its queue full),
/// in microseconds: which of the two the frame waits for.
pub static GL_BUSY_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static GL_WAIT_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Of the GL thread's time: making the batches from the commands, and showing the target
/// (its swap) (microseconds).
pub static GL_BATCHING_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static GL_SHOW_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(test)]
mod tests {
    use super::{TextureChanges, TextureId};

    #[test]
    fn a_texture_gone_is_not_also_written() {
        let p = |surface: u32| TextureId { surface, serial: 0 };
        let mut changes = TextureChanges::default();
        changes.note(p(1), false);
        changes.note(p(1), false);
        changes.note(p(2), false);
        changes.note(p(2), true);
        // Gone, then written again under the same id: still to be let go.
        changes.note(p(3), true);
        changes.note(p(3), false);
        let mut written: Vec<_> = changes.written.iter().map(|t| t.surface).collect();
        let mut gone: Vec<_> = changes.gone.iter().map(|t| t.surface).collect();
        written.sort();
        gone.sort();
        assert_eq!(written, [1]);
        assert_eq!(gone, [2, 3]);
    }

    #[test]
    fn a_level_made_again_under_a_surface_is_another_id() {
        let first = crate::platform_3d::test_level(vec![1; 4]);
        let second = TextureId {
            serial: first.serial.wrapping_add(1_000_000),
            ..first
        };
        // SAFETY: nothing writes the test level.
        unsafe {
            assert_eq!(crate::platform_3d::level_bytes(first), Some(&[1u8; 4][..]));
            assert_eq!(crate::platform_3d::level_bytes(second), None);
        }
    }
}
