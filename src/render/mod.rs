//! The renderer's boundary: the frame's 3D as commands the game's Direct3D layer (`platform::direct3d`)
//! notes, and a `Backend` that draws them. Everything the backends must agree on is here; each
//! one's own way of drawing is in its file.

#![allow(unsafe_code)]

pub mod background;
pub mod batches;
pub mod canvas_rows;
pub mod choice;
pub mod dump;
#[cfg(any(test, not(target_os = "emscripten")))]
pub mod gl_batching;
#[cfg(not(target_os = "emscripten"))]
pub mod gles;
#[cfg(target_os = "emscripten")]
#[path = "gles_web.rs"]
pub mod gles;
pub mod wash;
// Built for tests on every machine; only the handheld's GL thread uses it.
pub mod gpu_cut;
#[cfg(any(test, not(target_os = "emscripten")))]
pub mod gpu_time;
// The handheld has no Vulkan: its GPU is reached through `gles`, by the same name here.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub use gles as gpu;
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub mod gpu;
pub mod rasteriser;
#[cfg(target_arch = "aarch64")]
pub mod rasteriser_neon;
pub mod seethru;
pub mod small_frame;
pub mod software;

use crate::race::owner_course::Course;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};

/// Direct3D's render states a command carries.
pub const STATES: usize = 0x40;

/// A map keyed by texture ids: the ids are two small integers, which the std hasher's SipHash
/// costs far more to hash than the lookup is worth (one per run of fans in a flush).
pub type TextureMap<V> =
    std::collections::HashMap<TextureId, V, std::hash::BuildHasherDefault<TextureIdHasher>>;

/// Hashes a `TextureId`'s two integers by folding each into the state with a multiply (the Fx
/// hash); it is for keys of that kind only.
#[derive(Default)]
pub struct TextureIdHasher(u64);

impl std::hash::Hasher for TextureIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.write_u32(u32::from(*byte));
        }
    }

    fn write_u32(&mut self, value: u32) {
        const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
        self.0 = (self.0.rotate_left(5) ^ u64::from(value)).wrapping_mul(SEED);
    }
}

/// A texture level of the Direct3D layer (`platform::direct3d`): the number of its surface and a
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
    /// The level, whose `len` bytes `platform::direct3d::level_bytes` hands out.
    pub pixels: TextureId,
    pub len: usize,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub bytes: usize,
    pub masks: [u32; 4],
}

/// A fan's vertices: where they start in its list's `vertices` (`CommandList`) and how many.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct VertexSpan {
    first: u32,
    count: u32,
}

impl VertexSpan {
    /// How many vertices the fan has.
    pub fn len(self) -> usize {
        self.count as usize
    }

    fn range(self) -> std::ops::Range<usize> {
        self.first as usize..self.first as usize + self.count as usize
    }
}

pub enum Command {
    Fan {
        /// In the list's `vertices`: a fan has no vertices of its own (an allocation each).
        vertices: VertexSpan,
        /// Shared between fans with the same states, which most of a frame's are.
        states: std::sync::Arc<[u32; STATES]>,
        texture: Option<TextureNote>,
        /// The vertices' bounds on the screen: (left, top, right, bottom).
        bounds: (f32, f32, f32, f32),
    },
    /// The depth buffer back to 1.0 in a rectangle (left, top, right, bottom).
    Clear([usize; 4]),
}

/// Commands in the order they were noted, with the vertices their fans name (`VertexSpan`),
/// the fans' one after another: one unit, moved and joined as one.
#[derive(Default)]
pub struct CommandList {
    pub commands: Vec<Command>,
    pub vertices: Vec<crate::render::rasteriser::Vertex>,
    /// The textures the fans read, each once, in the order they are first read: what a GPU
    /// brings up for the list is looked up once a texture, not once a fan.
    textures: Vec<TextureNote>,
}

impl CommandList {
    pub const fn new() -> Self {
        Self {
            commands: Vec::new(),
            vertices: Vec::new(),
            textures: Vec::new(),
        }
    }

    /// An empty list with room for as many commands, vertices and textures as `like` holds.
    pub fn with_room_of(like: &CommandList) -> Self {
        Self {
            commands: Vec::with_capacity(like.commands.capacity()),
            vertices: Vec::with_capacity(like.vertices.capacity()),
            textures: Vec::with_capacity(like.textures.capacity()),
        }
    }

    /// The textures the fans read, each once, in the order they are first read (for the
    /// handheld's GL, which brings them up).
    #[cfg(any(test, not(target_os = "emscripten")))]
    pub fn textures(&self) -> &[TextureNote] {
        &self.textures
    }

    /// Whether a fan reads the texture `pixels`.
    pub fn reads_texture(&self, pixels: TextureId) -> bool {
        self.textures.iter().any(|t| t.pixels == pixels)
    }

    /// `texture` among the textures read, when it is new.
    fn note_texture(&mut self, texture: &TextureNote) {
        // Most fans run on in the last one's texture: that is looked at first.
        if self
            .textures
            .last()
            .is_none_or(|t| t.pixels != texture.pixels)
            && !self.reads_texture(texture.pixels)
        {
            self.textures.push(*texture);
        }
    }

    /// The commands noted.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Everything taken out, the room kept.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.vertices.clear();
        self.textures.clear();
    }

    /// A fan of `vertices` noted after the rest.
    pub fn push_fan(
        &mut self,
        vertices: &[crate::render::rasteriser::Vertex],
        states: std::sync::Arc<[u32; STATES]>,
        texture: Option<TextureNote>,
        bounds: (f32, f32, f32, f32),
    ) {
        let span = VertexSpan {
            first: self.vertices.len() as u32,
            count: vertices.len() as u32,
        };
        self.vertices.extend_from_slice(vertices);
        if let Some(t) = &texture {
            self.note_texture(t);
        }
        self.commands.push(Command::Fan {
            vertices: span,
            states,
            texture,
            bounds,
        });
    }

    /// A depth clear noted after the rest.
    pub fn push_clear(&mut self, rect: [usize; 4]) {
        self.commands.push(Command::Clear(rect));
    }

    /// A fan's vertices (`span` one of this list's fans').
    pub fn fan_vertices(&self, span: VertexSpan) -> &[crate::render::rasteriser::Vertex] {
        &self.vertices[span.range()]
    }

    /// `other`'s commands after this list's, its fans' vertices after this list's, its textures
    /// after this list's (those not read here already); `other` is left empty with its room.
    pub fn append(&mut self, other: &mut CommandList) {
        for t in other.textures.drain(..) {
            self.note_texture(&t);
        }
        let base = self.vertices.len() as u32;
        self.vertices.append(&mut other.vertices);
        self.commands.reserve(other.commands.len());
        for mut command in other.commands.drain(..) {
            if let Command::Fan { vertices, .. } = &mut command {
                vertices.first += base;
            }
            self.commands.push(command);
        }
    }
}

/// Drawn lists handed back (`give_back`), their room kept for the next flushes: a list grown to
/// a frame's fans each flush shows in a profile.
static SPARE_LISTS: Mutex<Vec<CommandList>> = Mutex::new(Vec::new());

/// Lists kept at most: the noted ones in flight (the GL's queues, `gles::JOBS_WAITING` and
/// `gles::BATCHED_WAITING`, and the one being batched) and the parts'.
const SPARE_LISTS_KEPT: usize = 8;

/// An empty list: a drawn one handed back, else one with `like`'s room.
pub fn spare_list(like: &CommandList) -> CommandList {
    SPARE_LISTS
        .lock()
        .unwrap()
        .pop()
        .unwrap_or_else(|| CommandList::with_room_of(like))
}

/// A list drawn: emptied and kept for `spare_list`.
pub fn give_back(mut list: CommandList) {
    if list.commands.capacity() == 0 && list.vertices.capacity() == 0 {
        return;
    }
    list.clear();
    let mut spare = SPARE_LISTS.lock().unwrap();
    if spare.len() < SPARE_LISTS_KEPT {
        spare.push(list);
    }
}

/// A command with vertices of its own, for building lists in tests (`list`).
#[cfg(test)]
#[derive(Clone)]
pub enum TestCommand {
    Fan {
        vertices: Vec<crate::render::rasteriser::Vertex>,
        states: std::sync::Arc<[u32; STATES]>,
        texture: Option<TextureNote>,
        bounds: (f32, f32, f32, f32),
    },
    Clear([usize; 4]),
}

/// The list of `commands`, in their order.
#[cfg(test)]
pub fn list(commands: impl IntoIterator<Item = TestCommand>) -> CommandList {
    let mut list = CommandList::new();
    for command in commands {
        match command {
            TestCommand::Fan {
                vertices,
                states,
                texture,
                bounds,
            } => list.push_fan(&vertices, states, texture, bounds),
            TestCommand::Clear(rect) => list.push_clear(rect),
        }
    }
    list
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
/// code may read or change the target and the textures as soon as it has. The list drawn goes
/// back to `give_back`, for the next flushes' noting.
pub trait Backend: Send {
    fn draw(&mut self, target: &Target, commands: CommandList);
}

/// The race's 3D drawn at this percent of the screen's size (`rc2k_core::gfx::render_scale`).
static RENDER_SCALE: AtomicU32 = AtomicU32::new(rc2k_core::gfx::render_scale::DEFAULT);

/// The percent of the screen's size the race's 3D is drawn at.
pub fn render_scale() -> u32 {
    RENDER_SCALE.load(Relaxed)
}

/// Sets the 3D's share (the options' load, the graphics page).
pub fn set_render_scale(percent: u32) {
    RENDER_SCALE.store(rc2k_core::gfx::render_scale::nearest(percent), Relaxed);
}

/// The share one choice on (`up`) or back.
pub fn step_render_scale(up: bool) {
    set_render_scale(rc2k_core::gfx::render_scale::step(render_scale(), up));
}

/// The percent the flush being drawn draws its 3D at: the chosen share in a race, the whole
/// screen elsewhere (a menu's 3D is drawn larger when the window shows the frame larger, see
/// `menu_high_percent`).
static FLUSH_SCALE: AtomicU32 = AtomicU32::new(100);

/// The percent of the screen's size the 3D of the flush being drawn takes.
pub fn flush_scale() -> u32 {
    FLUSH_SCALE.load(Relaxed)
}

/// Whether the flush being drawn is a race's.
static FLUSH_RACING: AtomicBool = AtomicBool::new(false);

/// The percent of the menus' frame the window shows it at (the frame's place in the window,
/// times the 3D's share), set by the platform when it is above 100; 0 when the frame is shown
/// at its own size or less.
static MENU_HIGH: AtomicU32 = AtomicU32::new(0);

/// Sets the size the window shows a menu's frame at (`menu_high_percent`).
pub fn set_menu_high(percent: u32) {
    MENU_HIGH.store(percent, Relaxed);
}

/// The percent of the frame's size the 3D of the menu flush being drawn is drawn at, when that
/// is more than the frame's own size: the menus' models then keep the window's sharpness while
/// the 2D stays the frame's pixels enlarged.
pub fn menu_high_percent() -> Option<u32> {
    let percent = MENU_HIGH.load(Relaxed);
    (percent > 100 && !FLUSH_RACING.load(Relaxed)).then_some(percent)
}

/// Whether the GPU holds the frame: the 3D of a race or of a menu on the GPU backend.
static GPU_HOLDS_FRAME: AtomicBool = AtomicBool::new(false);

/// Whether the GPU drew 3D since the last frame's end. A menu frame with none (a fade, a list
/// of texts) goes back to the rasteriser's picture at its end, for 2D code that reads the
/// screen's memory back.
static GPU_DREW_THIS_FRAME: AtomicBool = AtomicBool::new(false);

/// Draws the commands on the backend that holds the frame; `screen` is the screen's size (the
/// rasteriser's depth buffer is kept at it), `racing` whether a race is on. A menu's 3D goes to
/// the GPU as a race's does: its 2D before and after the 3D is the canvas's written pixels, put
/// over the target at each flush.
pub fn draw(screen: (usize, usize), racing: bool, target: &Target, commands: CommandList) {
    FLUSH_SCALE.store(if racing { render_scale() } else { 100 }, Relaxed);
    FLUSH_RACING.store(racing, Relaxed);
    if target.bits == 32 {
        let mut gpu = gpu::GPU.lock().unwrap();
        if let Some(gpu) = gpu.as_mut() {
            // On the first one, the picture so far goes over as written pixels: the canvas is
            // whole.
            if !GPU_HOLDS_FRAME.swap(true, Relaxed) {
                // The rasteriser's picture, written in place, unnoted.
                target.note_all_rows_written();
            }
            GPU_DREW_THIS_FRAME.store(true, Relaxed);
            Backend::draw(gpu, target, commands);
            return;
        }
    }
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    if target.bits == 32
        && let Some(gl) = gles::GPU.lock().unwrap().as_mut()
    {
        if !GPU_HOLDS_FRAME.swap(true, Relaxed) {
            target.note_all_rows_written();
        }
        GPU_DREW_THIS_FRAME.store(true, Relaxed);
        Backend::draw(gl, target, commands);
        return;
    }
    give_frame_back(target);
    // Whatever was noted for a GPU is on the canvas before the rasteriser draws.
    crate::gfx::draw2d::quads::before_canvas_write();
    software::Software { screen }.draw(target, commands);
}

/// Whether the GPU holds the frame now (the canvas is then only what the game wrote since the
/// last flush).
pub fn frame_on_gpu() -> bool {
    GPU_HOLDS_FRAME.load(Relaxed)
}

/// The GPU's picture back in the canvas, if it holds the frame and something wants the true one.
pub fn bring_back(target: &Target) {
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    if GPU_HOLDS_FRAME.load(Relaxed)
        && let Some(gl) = gles::GPU.lock().unwrap().as_mut()
    {
        // SAFETY: as below: the game's thread waits for this call and no other slice of the
        // canvas lives.
        gl.bring_back(target, unsafe { target.bytes() });
        target.note_all_rows_written();
        return;
    }
    if GPU_HOLDS_FRAME.load(Relaxed)
        && let Some(gpu) = gpu::GPU.lock().unwrap().as_mut()
    {
        // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
        // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
        // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
        let canvas = unsafe { target.bytes() };
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        gpu.bring_back(target, canvas);
        #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
        {
            let rows = canvas_rows::written_rows(target, canvas);
            if let Err(why) = gpu.bring_back(Some(rows), canvas) {
                crate::logfile::program_log::log(&format!(
                    "render: the frame did not come back: {why}"
                ));
            }
        }
        // The picture written in place, unnoted.
        target.note_all_rows_written();
    }
}

/// Whether the frame ending now stays on the GPU: a race's always, a menu's when it drew 3D or
/// kept its background on the GPU (`background`). Starts the next frame's count.
fn frame_stays_on_gpu(course: &Course) -> bool {
    let drew = GPU_DREW_THIS_FRAME.swap(false, Relaxed);
    let kept_background = background::recorded_this_frame();
    drew || kept_background || course.numbers.get().in_race.is_set()
}

/// The frame back for good: the rasteriser draws from here.
fn give_frame_back(target: &Target) {
    bring_back(target);
    GPU_HOLDS_FRAME.store(false, Relaxed);
}

/// The frame's end, before it is shown. The GPU's colour target when that is what to show, the
/// canvas's written pixels over it, with the number of its set of targets and its size
/// (`Gpu::colour_texture`).
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn finish_frame(
    course: &Course,
    target: &Target,
) -> Option<(*mut sdl3_sys::everything::SDL_GPUTexture, u64, (u32, u32))> {
    if !GPU_HOLDS_FRAME.load(Relaxed) {
        return None;
    }
    // The OpenGL backend's window shows its own frames (`show_on_gl_window`); a frame that
    // comes here is read back into the canvas.
    let stays = frame_stays_on_gpu(course);
    if !stays || gles::GPU.lock().unwrap().is_some() {
        give_frame_back(target);
        if stays {
            seethru::frame_end();
        }
        return None;
    }
    let mut gpu = gpu::GPU.lock().unwrap();
    let gpu = gpu.as_mut()?;
    // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
    // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
    // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
    let canvas = unsafe { target.bytes() };
    let rows = canvas_rows::written_rows(target, canvas);
    if let Err(why) = gpu.finish_frame(Some(rows), canvas) {
        crate::logfile::program_log::log(&format!("render: the frame did not finish: {why}"));
    }
    if let Err(why) = gpu.finish_high() {
        crate::logfile::program_log::log(&format!(
            "render: the menu's larger 3D did not finish: {why}"
        ));
    }
    Some(gpu.colour_texture())
}

/// The frame on the panel, by the GPU when it is running: the target during a race, else
/// the canvas's picture, which `with_frame` hands over (taken only after the GPU's writes to the
/// canvas here, so that no slice of it is held across them). False: it is not running.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn show_on_panel(
    course: &Course,
    target: &Target,
    with_frame: impl FnOnce(&mut dyn FnMut(&[u8])),
    width: usize,
    height: usize,
) -> bool {
    gpu::start_if_wanted();
    if GPU_HOLDS_FRAME.load(Relaxed) && !frame_stays_on_gpu(course) {
        give_frame_back(target);
    }
    let mut gpu = gpu::GPU.lock().unwrap();
    let Some(gpu) = gpu.as_mut() else {
        return false;
    };
    if GPU_HOLDS_FRAME.load(Relaxed) {
        // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
        // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
        // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
        // The handheld's panel is the frame's own size: no space beside it to fill.
        gpu.finish_and_show(target, unsafe { target.bytes() }, false);
    } else {
        with_frame(&mut |frame| gpu.show_picture(frame, width, height, false));
    }
    true
}

/// The frame on the OpenGL backend's own window, which shows it without the frame's coming back
/// through the canvas: the target during a race (and in the menus that keep their background on
/// the GPU), else the canvas's picture, which `with_frame` hands over. False: the backend does
/// not show a window (it is not running), and the caller shows the frame.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub fn show_on_gl_window(
    course: &Course,
    target: &Target,
    with_frame: impl FnOnce(&mut dyn FnMut(&[u8])),
    width: usize,
    height: usize,
    menu: bool,
) -> bool {
    if gles::window().is_none() || gles::GPU.lock().unwrap().is_none() {
        return false;
    }
    if GPU_HOLDS_FRAME.load(Relaxed) && !frame_stays_on_gpu(course) {
        give_frame_back(target);
    }
    let mut gl = gles::GPU.lock().unwrap();
    let Some(gl) = gl.as_mut() else {
        return false;
    };
    if menu
        && let Some(window) = gles::window().and_then(crate::platform::ffi::sdl::Window::pixel_size)
    {
        // Made and sent only when the background or the window's size has changed.
        if let Some(image) = wash::image_for(window) {
            gl.set_wash(&image);
        }
    }
    if GPU_HOLDS_FRAME.load(Relaxed) {
        // SAFETY: `Target::bytes`' promise: on the game's thread, which waits for this call, so
        // nothing else reaches the canvas; no other slice of it lives (the frame's showing takes
        // its own after this returns, `platform::with_frame`); the screen keeps its pixels.
        gl.finish_and_show(target, unsafe { target.bytes() }, menu);
    } else {
        with_frame(&mut |frame| gl.show_picture(frame, width, height, menu));
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
    crate::gfx::draw2d::quads::gpu_changed();
    GPU_STARTED.store(true, Relaxed);
}

/// A GPU backend that had started is given up (the screen's targets could not be made): its
/// texture changes are no longer kept.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub(crate) fn gpu_stopped() {
    GPU_STARTED.store(false, Relaxed);
    crate::gfx::draw2d::quads::gpu_changed();
    background::clear();
    *CHANGED_TEXTURES.lock().unwrap() = None;
}

/// Whether the desktop's OpenGL backend is the one running (the handheld's GL ES is its GPU
/// renderer, `gpu_running`).
pub fn gl_running() -> bool {
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "aarch64"),
        target_os = "emscripten"
    )))]
    {
        gles::GPU.lock().unwrap().is_some()
    }
    #[cfg(any(
        all(target_os = "linux", target_arch = "aarch64"),
        target_os = "emscripten"
    ))]
    {
        false
    }
}

/// Whether a GPU backend is running: what started, not what was asked for.
pub fn gpu_running() -> bool {
    GPU_STARTED.load(Relaxed)
}

/// The renderer that draws the 3D now, as a wish names it: OpenGL, the GPU API (on the handheld
/// its GL ES), or the software rasteriser.
pub fn running_wish() -> choice::Wish {
    if gl_running() {
        choice::Wish::Gl
    } else if gpu_running() {
        choice::Wish::GpuFirst
    } else {
        choice::Wish::Software
    }
}

/// Whether the GPU's frame may be given up: a frame the GPU holds is brought back into the
/// canvas, and the rasteriser draws from here. For the renderer's swap.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
pub(crate) fn release_frame(target: Option<&Target>) {
    if let Some(target) = target {
        give_frame_back(target);
    }
    GPU_HOLDS_FRAME.store(false, Relaxed);
    GPU_DREW_THIS_FRAME.store(false, Relaxed);
}

/// Whether the software renderer keeps smaller levels of textures (`software`): only then are
/// the textures' changes kept for it (they tell it which levels are stale).
static LEVELS_KEPT: AtomicBool = AtomicBool::new(false);

static LEVEL_CHANGES: Mutex<Vec<TextureId>> = Mutex::new(Vec::new());

/// From now on the textures' changes are kept for the software renderer's smaller levels.
pub(crate) fn keep_level_changes() {
    LEVELS_KEPT.store(true, Relaxed);
}

/// The textures written or gone since the last call (when `keep_level_changes` was called).
pub(crate) fn take_level_changes() -> Vec<TextureId> {
    std::mem::take(&mut *LEVEL_CHANGES.lock().unwrap())
}

fn note_texture_change(pixels: TextureId, gone: bool) {
    if LEVELS_KEPT.load(Relaxed) {
        LEVEL_CHANGES.lock().unwrap().push(pixels);
    }
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
    // SAFETY: `level_bytes`' promise. The noted fans are drawn (the rasteriser's tiles, the
    // GPU's uploads, the dumps) only inside `platform::direct3d::flush`'s drawing, on the game's thread
    // with its drawing threads joined before it returns, and the slices are dropped with that
    // drawing (`'a` does not bind them: the callers' discipline). Every write to a level's
    // pixels and every letting go of one flushes the noted drawing that reads it first
    // (`lock_surface`, `hold_texture`, `copy_screen_to_system_texture`, `allocate_textures_now`
    // before `make_texture` replaces a level, `release_all_textures`), so neither meets a
    // slice that is still drawn from.
    unsafe { crate::platform::direct3d::level_bytes(note.pixels) }.unwrap_or_else(|| {
        crate::fatal::raise::end(
            "a texture's pixels",
            crate::errors::BadInput("a noted texture's level is gone"),
        )
    })
}

/// A texture's pixels as bytes of 0xAARRGGBB, its rows without gaps.
pub(crate) fn texture_as_bgra(note: &TextureNote) -> Vec<u8> {
    pixels_as_bgra(noted_bytes(note), note)
}

/// `pixels`, laid out as `note` says, as bytes of 0xAARRGGBB, its rows without gaps. A pixel
/// past the end of `pixels` counts as zero bits.
fn pixels_as_bgra(pixels: &[u8], note: &TextureNote) -> Vec<u8> {
    let mut out = Vec::with_capacity(note.width * note.height * 4);
    // How a pixel's raw bits become 0xAARRGGBB (`rasteriser::to_argb`), worked out once.
    enum Convert {
        /// A 16-bit format: its table.
        Table(std::sync::Arc<Vec<u32>>),
        /// 32 bits already laid out as 0xAARRGGBB (or with no alpha, which is made full).
        Same {
            alpha: u32,
        },
        Masks,
    }
    let m = note.masks;
    let convert = if note.bytes == 2 {
        Convert::Table(crate::render::rasteriser::table_for(&m))
    } else if note.bytes == 4
        && m[0] == 0xFF
        && m[1] == 0xFF00
        && m[2] == 0xFF_0000
        && (m[3] == 0 || m[3] == 0xFF00_0000)
    {
        Convert::Same {
            alpha: if m[3] == 0 { 0xFF00_0000 } else { 0 },
        }
    } else {
        Convert::Masks
    };
    for y in 0..note.height {
        let start = y * note.pitch;
        // A whole row inside the pixels, in one of the two usual layouts: converted as a run.
        let row = pixels.get(start..start + note.width * note.bytes);
        match (row, &convert) {
            (Some(row), Convert::Table(table)) => {
                for pixel in row.as_chunks::<2>().0 {
                    let argb = table[u16::from_le_bytes(*pixel) as usize & 0xFFFF];
                    out.extend_from_slice(&argb.to_le_bytes());
                }
            }
            (Some(row), Convert::Same { alpha }) => {
                for pixel in row.as_chunks::<4>().0 {
                    let argb = u32::from_le_bytes(*pixel) | alpha;
                    out.extend_from_slice(&argb.to_le_bytes());
                }
            }
            // Any other layout, or a row that runs past the pixels: pixel by pixel.
            _ => {
                for x in 0..note.width {
                    let at = start + x * note.bytes;
                    let mut raw = [0u8; 4];
                    if let Some(from) = pixels.get(at..at + note.bytes) {
                        raw[..note.bytes].copy_from_slice(from);
                    }
                    let pixel = u32::from_le_bytes(raw);
                    let argb = match &convert {
                        Convert::Table(table) => table[pixel as usize & 0xFFFF],
                        Convert::Same { alpha } => pixel | alpha,
                        Convert::Masks => crate::render::rasteriser::to_argb(pixel, &m),
                    };
                    out.extend_from_slice(&argb.to_le_bytes());
                }
            }
        }
    }
    out
}

impl Target {
    /// `f` on the screen's rows: for the backends, which draw while the game's code waits for
    /// the call (nothing else writes the canvas meanwhile).
    pub fn with_canvas<R>(&self, f: impl FnOnce(&mut [u8]) -> R) -> R {
        // SAFETY: `bytes`' promise: the backends call this on the game's thread, which waits for
        // the call, and `f` (their drawing) takes no other slice of the canvas; the slice lives
        // through `f` only, while the screen keeps its pixels.
        f(unsafe { self.bytes() })
    }

    /// The rows written through the pixels' handle since the last call (`PoolBytes::
    /// take_written_rows`), one flag a row; `None` when the pixels do not note them at the
    /// target's pitch.
    pub fn take_written_rows(&self) -> Option<Vec<bool>> {
        crate::gfx::locked_pixels::bytes_of(self.pixels)?.take_written_rows(self.pitch)
    }

    /// Every row noted as written, after a write in place (`bytes`, `pixels`).
    pub fn note_all_rows_written(&self) {
        if let Some(held) = crate::gfx::locked_pixels::bytes_of(self.pixels) {
            held.note_all_rows_written();
        }
    }

    /// The screen's rows as bytes.
    ///
    /// # Safety
    /// Nothing else may touch them while the slice lives: no other thread, no read or write
    /// through the pixels' handle (`gfx::locked_pixels`), no second slice from `bytes` or
    /// `pixels`; and the screen keeps its pixels meanwhile (the slice is not bound to them).
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
        // SAFETY: the pixels' bytes, `len` of them (checked above), alive as long as the screen
        // keeps them (`held` is dropped here; the screen's own handle keeps the block, the
        // caller's promise), and touched by nothing else while the slice lives (the same).
        unsafe { std::slice::from_raw_parts_mut(held.as_mut_ptr(), len) }
    }

    /// The screen's pixels for the rasteriser, rows `pitch / 4` pixels apart: its bytes
    /// (`bytes`) as 32-bit pixels. A target that is not 32 bits a pixel, not aligned to its
    /// pixels, or whose pitch is not whole pixels, ends the program.
    ///
    /// # Safety
    /// As `bytes`.
    pub unsafe fn pixels<'a>(&self) -> &'a mut [u32] {
        let size = (self.bits / 8) as usize;
        // SAFETY: the caller's promise.
        let bytes = unsafe { self.bytes() };
        let at = bytes.as_mut_ptr();
        assert!(
            size == 4 && (at as usize).is_multiple_of(size) && self.pitch.is_multiple_of(size),
            "the screen's pixels not whole aligned pixels"
        );
        let len = bytes.len() / size;
        // SAFETY: the same bytes (borrowed for 'a with them), aligned and whole pixels (checked
        // above); every bit pattern is a pixel.
        unsafe { std::slice::from_raw_parts_mut(at.cast(), len) }
    }
}

/// For the timing line (`platform::show`): fans drawn and the microseconds
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
/// The bytes of the canvas and of the retained background sent up to the PC's GPU backend.
pub static CANVAS_UPLOAD_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// The PC's GPU backend's flushes, and how many of them sent canvas rows up.
pub static GPU_FLUSHES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static CANVAS_UPLOADS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static BACKGROUND_UPLOAD_BYTES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
/// The GPU's draw calls (a run of triangles drawn alike each): what the handheld's driver
/// spends its time on.
pub static GPU_DRAWS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Of them, the draws by a scene program with no alpha test (`Batch::discard_free`), and
/// the times the GL thread went from one of its scene programs to another within a flush.
pub static GPU_FREE_DRAWS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static GPU_PROGRAM_SWITCHES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
/// The GL thread's time at its jobs, and the game's waiting to hand it one (its queue full),
/// in microseconds: which of the two the frame waits for.
pub static GL_BUSY_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static GL_WAIT_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// Making the batches from the commands (the batching thread's time beside the GL's,
/// `gl_batching`), and of the GL thread's time showing the target (its swap) (microseconds).
pub static GL_BATCHING_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
pub static GL_SHOW_MICROS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(test)]
mod tests {
    use super::{Command, CommandList, TestCommand, TextureChanges, TextureId, TextureMap, list};

    /// A scene of `count` commands as the game notes them: fans of 3 to 12 corners (most past
    /// six), runs of shared states, textures and clears between, from a fixed seed.
    fn scene(count: usize) -> Vec<TestCommand> {
        let mut state = 0x2468_ACE1u32;
        let mut next = move || {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            state
        };
        let mut states = std::sync::Arc::new([0u32; super::STATES]);
        (0..count)
            .map(|n| {
                if next() % 29 == 0 {
                    return TestCommand::Clear([n, n + 1, n + 40, n + 30]);
                }
                if next() % 7 == 0 {
                    let mut changed = *states;
                    changed[0x1B] = next() % 2;
                    changed[0x0E] = next() % 2;
                    states = std::sync::Arc::new(changed);
                }
                let corners = 3 + (next() % 10) as usize;
                let vertices = (0..corners)
                    .map(|_| {
                        let mut v = [0u32; 8];
                        v[0] = ((next() % 640) as f32).to_bits();
                        v[1] = ((next() % 480) as f32).to_bits();
                        v[2] = (next() as f32 / u32::MAX as f32).to_bits();
                        v[3] = 1.0f32.to_bits();
                        for d in &mut v[4..] {
                            *d = next();
                        }
                        v
                    })
                    .collect();
                let texture = (next() % 3 == 0).then(|| super::TextureNote {
                    pixels: TextureId {
                        surface: next() % 4,
                        serial: 0,
                    },
                    len: 4,
                    width: 1,
                    height: 1,
                    pitch: 4,
                    bytes: 4,
                    masks: [0xFF_0000, 0xFF00, 0xFF, 0xFF00_0000],
                });
                TestCommand::Fan {
                    vertices,
                    states: std::sync::Arc::clone(&states),
                    texture,
                    bounds: (0.0, 0.0, 640.0, 480.0),
                }
            })
            .collect()
    }

    /// Whether `noted` reads back as `scene`, command by command: each fan its own vertices in
    /// order, its states (the same shared ones), texture and bounds.
    fn reads_back_as(noted: &CommandList, scene: &[TestCommand]) {
        assert_eq!(noted.len(), scene.len());
        for (n, (got, want)) in noted.commands.iter().zip(scene).enumerate() {
            match (got, want) {
                (
                    Command::Fan {
                        vertices,
                        states,
                        texture,
                        bounds,
                    },
                    TestCommand::Fan {
                        vertices: want_vertices,
                        states: want_states,
                        texture: want_texture,
                        bounds: want_bounds,
                    },
                ) => {
                    assert_eq!(noted.fan_vertices(*vertices), &want_vertices[..], "{n}");
                    assert!(std::sync::Arc::ptr_eq(states, want_states), "{n}");
                    assert_eq!(
                        texture.map(|t| t.pixels),
                        want_texture.map(|t| t.pixels),
                        "{n}"
                    );
                    assert_eq!(bounds, want_bounds, "{n}");
                }
                (Command::Clear(got), TestCommand::Clear(want)) => assert_eq!(got, want, "{n}"),
                _ => panic!("command {n} of another kind"),
            }
        }
    }

    /// A scene noted in one list and the same scene noted by parts and joined in their order
    /// (`platform::direct3d::put_noted`) read back as the scene, and give the same batches, plain and
    /// with second passes joined: the fans' vertices in their list draw what each fan's own did.
    #[test]
    fn a_scene_noted_whole_or_by_parts_reads_back_and_batches_the_same() {
        let scene = scene(600);
        let whole = list(scene.iter().cloned());
        reads_back_as(&whole, &scene);
        let mut joined = CommandList::new();
        let mut rest = &scene[..];
        for cut in [0, 1, 150, 7, 200] {
            let (part, after) = rest.split_at(cut.min(rest.len()));
            let mut part = list(part.iter().cloned());
            joined.append(&mut part);
            assert!(part.is_empty() && part.vertices.is_empty());
            rest = after;
        }
        joined.append(&mut list(rest.iter().cloned()));
        reads_back_as(&joined, &scene);
        assert_eq!(joined.vertices, whole.vertices);
        // The textures read, each once, in the order the fans first read them, whole or joined.
        let mut first_read: Vec<TextureId> = Vec::new();
        for c in &scene {
            if let TestCommand::Fan {
                texture: Some(t), ..
            } = c
                && !first_read.contains(&t.pixels)
            {
                first_read.push(t.pixels);
            }
        }
        let ids =
            |l: &CommandList| -> Vec<TextureId> { l.textures().iter().map(|t| t.pixels).collect() };
        assert_eq!(ids(&whole), first_read);
        assert_eq!(ids(&joined), first_read);
        assert!(first_read.len() > 1 && whole.reads_texture(first_read[1]));
        assert!(!whole.reads_texture(TextureId {
            surface: 9,
            serial: 0
        }));
        let mut cleared = list(scene.iter().cloned());
        cleared.clear();
        assert!(cleared.textures().is_empty());
        let opaque = |t: &super::TextureNote| t.pixels.surface == 1;
        for opaque in [None, Some(&opaque as &dyn Fn(&super::TextureNote) -> bool)] {
            let a = super::batches::Batches::of(&whole, 640, 480, opaque);
            let b = super::batches::Batches::of(&joined, 640, 480, opaque);
            assert_eq!(a.vertices, b.vertices);
            assert_eq!(a.batches, b.batches);
            assert_eq!(a.seconds, b.seconds);
            assert!(a.batches.len() > 1 && a.vertices.len() > 3 * 600);
        }
    }

    /// Every id of a grid of surfaces and serials is found again under its own value, and
    /// the same hashes as a std map's lookups give the same answers.
    #[test]
    fn texture_map_finds_each_id() {
        let mut map = TextureMap::default();
        let mut reference = std::collections::HashMap::new();
        for surface in 0..0x140u32 {
            for serial in 0..8u32 {
                let id = TextureId { surface, serial };
                map.insert(id, surface * 8 + serial);
                reference.insert(id, surface * 8 + serial);
            }
        }
        assert_eq!(map.len(), reference.len());
        for (id, value) in &reference {
            assert_eq!(map.get(id), Some(value));
        }
        assert_eq!(
            map.get(&TextureId {
                surface: 0x140,
                serial: 0
            }),
            None
        );
    }

    /// The upload's conversion gives what `rasteriser::to_argb` gives for every pixel, in
    /// each layout the fast paths take and one they do not.
    #[test]
    fn texture_conversion_matches_the_masks() {
        let layouts: [(usize, [u32; 4]); 5] = [
            (2, [0x001F, 0x07E0, 0xF800, 0]),
            (2, [0x000F, 0x00F0, 0x0F00, 0xF000]),
            (4, [0xFF, 0xFF00, 0xFF_0000, 0xFF00_0000]),
            (4, [0xFF, 0xFF00, 0xFF_0000, 0]),
            (4, [0xFF, 0xFF00, 0xFF_0000, 0x8000_0000]),
        ];
        for (bytes, masks) in layouts {
            let (width, height) = (16usize, 16usize);
            let mut state = 0x1234_5678u32;
            let raw: Vec<u8> = (0..width * height * bytes)
                .map(|_| {
                    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    (state >> 24) as u8
                })
                .collect();
            let id = crate::platform::direct3d::test_level(raw.clone());
            let note = super::TextureNote {
                pixels: id,
                len: raw.len(),
                width,
                height,
                pitch: width * bytes,
                bytes,
                masks,
            };
            let got = super::texture_as_bgra(&note);
            for (n, chunk) in raw.chunks_exact(bytes).enumerate() {
                let mut word = [0u8; 4];
                word[..bytes].copy_from_slice(chunk);
                let want = crate::render::rasteriser::to_argb(u32::from_le_bytes(word), &masks);
                assert_eq!(
                    got[n * 4..n * 4 + 4],
                    want.to_le_bytes(),
                    "{bytes} {masks:x?} {n}"
                );
            }
        }
    }

    /// Rows with a gap after them are read at the pitch, and pixels past the end of the bytes
    /// (a last row cut short) count as zero bits, in every layout.
    #[test]
    fn texture_conversion_follows_the_pitch_and_a_short_end() {
        let layouts: [(usize, [u32; 4]); 4] = [
            (2, [0x001F, 0x07E0, 0xF800, 0]),
            (4, [0xFF, 0xFF00, 0xFF_0000, 0xFF00_0000]),
            (4, [0xFF, 0xFF00, 0xFF_0000, 0]),
            (4, [0xFF, 0xFF00, 0xFF_0000, 0x8000_0000]),
        ];
        for (bytes, masks) in layouts {
            let (width, height) = (5usize, 4usize);
            let pitch = width * bytes + 6;
            let mut state = 0x9E37_79B9u32;
            // The last row lacks its last pixel and a byte of the one before.
            let raw: Vec<u8> = (0..pitch * (height - 1) + (width - 1) * bytes - 1)
                .map(|_| {
                    state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    (state >> 24) as u8
                })
                .collect();
            let note = super::TextureNote {
                pixels: TextureId {
                    surface: 0,
                    serial: 0,
                },
                len: raw.len(),
                width,
                height,
                pitch,
                bytes,
                masks,
            };
            let got = super::pixels_as_bgra(&raw, &note);
            assert_eq!(got.len(), width * height * 4);
            for y in 0..height {
                for x in 0..width {
                    let at = y * pitch + x * bytes;
                    let mut word = [0u8; 4];
                    if let Some(from) = raw.get(at..at + bytes) {
                        word[..bytes].copy_from_slice(from);
                    }
                    let want = crate::render::rasteriser::to_argb(u32::from_le_bytes(word), &masks);
                    let n = y * width + x;
                    assert_eq!(
                        got[n * 4..n * 4 + 4],
                        want.to_le_bytes(),
                        "{bytes} {masks:x?} {x} {y}"
                    );
                }
            }
        }
    }

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
        let first = crate::platform::direct3d::test_level(vec![1; 4]);
        let second = TextureId {
            serial: first.serial.wrapping_add(1_000_000),
            ..first
        };
        // SAFETY: nothing writes the test level.
        unsafe {
            assert_eq!(
                crate::platform::direct3d::level_bytes(first),
                Some(&[1u8; 4][..])
            );
            assert_eq!(crate::platform::direct3d::level_bytes(second), None);
        }
    }
}
