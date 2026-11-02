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

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

/// Direct3D's render states a command carries.
pub const STATES: usize = 0x40;

/// A texture's first level as noted: its memory and how it is laid out.
#[derive(Clone, Copy)]
pub struct TextureNote {
    /// The memory's address (a number: the tiles' threads read it).
    pub pixels: usize,
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
    pub pixels: usize,
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

/// Draws the commands on the backend that holds the frame.
pub fn draw(target: &Target, commands: Vec<Command>) {
    let in_race = crate::race::numbers_store::NUMBERS.in_race.get() != 0;
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
    software::Software.draw(target, commands);
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
/// canvas's written pixels over it.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn finish_frame(target: &Target) -> Option<*mut sdl3_sys::everything::SDL_GPUTexture> {
    if !GPU_HOLDS_FRAME.load(Relaxed) {
        return None;
    }
    if crate::race::numbers_store::NUMBERS.in_race.get() == 0 {
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
pub fn show_on_panel(target: &Target, frame: &[u8], width: usize, height: usize) -> bool {
    gpu::start_if_wanted();
    if GPU_HOLDS_FRAME.load(Relaxed) && crate::race::numbers_store::NUMBERS.in_race.get() == 0 {
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

/// The textures whose memory may have changed since a GPU last asked.
static CHANGED_TEXTURES: Mutex<Vec<usize>> = Mutex::new(Vec::new());

/// The texture at `pixels` is being written, or is gone.
pub fn texture_changed(pixels: usize) {
    let mut changed = CHANGED_TEXTURES.lock().unwrap();
    if !changed.contains(&pixels) {
        changed.push(pixels);
    }
}

pub(crate) fn take_changed_textures() -> Vec<usize> {
    std::mem::take(&mut *CHANGED_TEXTURES.lock().unwrap())
}

/// A texture's pixels as bytes of 0xAARRGGBB, its rows without gaps.
pub(crate) fn texture_as_bgra(note: &TextureNote) -> Vec<u8> {
    // SAFETY: as the rasteriser's read: a noted texture is not written while it is drawn from.
    let pixels = unsafe {
        crate::pointers::GamePtr::new(note.pixels as u32).slice(note.len, "a texture's pixels")
    };
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
    /// The screen's rows as bytes.
    ///
    /// # Safety
    /// Nothing else may touch them while the slice lives.
    pub unsafe fn bytes<'a>(&self) -> &'a mut [u8] {
        let len = self.pitch * self.height;
        crate::pointers::check_or_end(self.pixels as u32, len, "the screen's pixels");
        // SAFETY: the bytes checked above; the caller's promise.
        unsafe { std::slice::from_raw_parts_mut(crate::pointers::host_at(self.pixels), len) }
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
