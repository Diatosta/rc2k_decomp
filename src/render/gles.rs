//! The OpenGL backend. On the handheld it is the GPU backend: OpenGL ES 3 through the system's
//! `libmali` (no Vulkan driver there, and SDL has no display for it), drawing to `/dev/fb0`
//! through its EGL. On a PC it is the one after the GPU API in the default order, for machines
//! without Vulkan, Direct3D 12 or Metal: an OpenGL 3.3 core context on an SDL window that
//! is shown in place of the game's own, drawing into its own target, which each frame's end puts
//! on that window as the handheld puts it on its panel (the shaders are the handheld's, under desktop GLSL's version line,
//! `Gles::compile_shader`). The same frame
//! as `gpu`'s: `batches`' lists, and before each part's 3D the retained background, the canvas's
//! written pixels and the 2D quads (`layer2d`) put over the target. The GL's own thread holds
//! the context and does the driver's work (a quarter of a frame's time on this processor); the
//! game's thread hands it copies, by channel, and goes on. They pass a batching thread on the
//! way (`gl_batching`), which makes each part's batches while the GL draws the part before.
//! The calls into the library are `ffi::gl`'s.

mod bench;
mod draw;
mod layer2d;
pub mod mip_levels;
mod present;
mod programs;
mod sources;
mod targets;
mod textures;
mod thread;

use self::sources::*;
pub use self::thread::Front;
#[cfg(handheld)]
pub use self::thread::start_if_wanted;
use self::thread::*;
#[cfg(not(handheld))]
pub use self::thread::{start_on_window, stop, window};

use super::batches::{Batch, Batches, NO_TEXTURE, PassKind, PipelineKey, PixelValues, ShaderKind};
use super::dev::gpu_cut::Cuts;
use super::gl_batching::{Background, Batched, Job, Part, Written};
use super::{CommandList, Target, TextureNote};
use super::{background, canvas_rows};
use crate::gfx::draw2d::quads;
use crate::platform::ffi::gl::Gles;
use crate::program_log::log;
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, SyncSender, channel, sync_channel};

/// A canvas pixel the game has not written (`gpu::UNWRITTEN`'s kin).
pub const UNWRITTEN: u32 = 0xA5FF_00FF;

const TEXTURE_2D: u32 = 0x0DE1;
const TEXTURE0: u32 = 0x84C0;
const TEXTURE_2D_ARRAY: u32 = 0x8C1A;
const READ_FRAMEBUFFER: u32 = 0x8CA8;
const DRAW_FRAMEBUFFER: u32 = 0x8CA9;
const UNSIGNED_SHORT: u32 = 0x1403;
const RGBA: u32 = 0x1908;
const UNSIGNED_BYTE: u32 = 0x1401;
const FRAMEBUFFER: u32 = 0x8D40;
const ARRAY_BUFFER: u32 = 0x8892;
const DEPTH_TEST: u32 = 0x0B71;
const BLEND: u32 = 0x0BE2;
const SRC_ALPHA: u32 = 0x0302;
const ONE_MINUS_SRC_ALPHA: u32 = 0x0303;
const CULL_FACE: u32 = 0x0B44;
const SCISSOR_TEST: u32 = 0x0C11;
const TRIANGLES: u32 = 4;
const COLOUR_BIT: u32 = 0x4000;
const DEPTH_BIT: u32 = 0x0100;

/// The panel.
const PANEL: (i32, i32) = (720, 480);

struct Quad {
    program: u32,
    flip: i32,
    /// Where a picture of another shape goes in the window (`FIT_VS`), and whether the rest is
    /// black; -1 in a quad that has none.
    fit: i32,
    bars: i32,
    /// The wash's sampler and whether there is one (`FIT_VS`'s quads); -1 in the others.
    wash: i32,
    has_wash: i32,
}

/// A picture's place in the window it is shown on.
#[derive(Clone, Copy)]
struct Placement {
    /// Left, top, width and height as fractions of the window.
    fit: [f32; 4],
    /// The window's rest is black: a race's frame (a menu's side space has its edge colours).
    bars: bool,
    /// The picture is shown with no filter.
    sharp: bool,
}

struct Locations {
    size: i32,
    first: ValueLocations,
    second: ValueLocations,
}

/// What one of the scene's programs is made for.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum ProgramKind {
    /// Every batch, each pass's function, mixing and flags read for every pixel;
    /// `discard_free`: with no alpha test, for the batches that cannot fail it.
    Any { discard_free: bool },
    /// The batches of one kind, with nothing of it decided per pixel.
    Made(ShaderKind),
}

impl ProgramKind {
    /// The program for `batch`: its kind's own, or (`for_any`) one of the two for every kind.
    fn of(batch: &Batch, for_any: bool) -> ProgramKind {
        if for_any {
            ProgramKind::Any {
                discard_free: batch.discard_free,
            }
        } else {
            ProgramKind::Made(ShaderKind::of(batch))
        }
    }

    /// Whether the program has no alpha test.
    fn discard_free(&self) -> bool {
        match self {
            ProgramKind::Any { discard_free } => *discard_free,
            ProgramKind::Made(kind) => kind.first.alpha_function == 0,
        }
    }

    fn defines(&self) -> String {
        match self {
            ProgramKind::Any { discard_free } => ShaderKind::defines_for_any(*discard_free),
            ProgramKind::Made(kind) => kind.defines(),
        }
    }
}

/// One of the scene's programs, and what its uniforms were last given: they are the
/// program's own and stay with it from flush to flush.
struct SceneProgram {
    name: u32,
    locations: Locations,
    size: Option<(i32, i32)>,
    first: Option<PixelValues>,
    second: Option<PixelValues>,
}

/// A pass's kind: its alpha test, its mixing, whether it has a texture, takes the texel's
/// alpha, is fogged.
const fn pass(
    alpha_function: u8,
    mixing: u8,
    textured: bool,
    texel_alpha: bool,
    fog: bool,
) -> PassKind {
    PassKind {
        alpha_function,
        mixing,
        textured,
        texel_alpha,
        fog,
    }
}

/// A kind of one pass.
const fn alone(first: PassKind) -> ShaderKind {
    ShaderKind {
        first,
        second: None,
    }
}

/// The kinds of batch the game draws (menus, stages in every weather, replays), whose
/// programs are made when the renderer starts. A kind not listed still draws: its program is
/// made when first met, and the log says which it was.
const KINDS_AT_START: [ShaderKind; 9] = [
    alone(pass(0, 0, false, false, false)),
    alone(pass(5, 0, false, false, false)),
    alone(pass(0, 0, false, false, true)),
    alone(pass(0, 4, true, false, false)),
    alone(pass(0, 4, true, false, true)),
    alone(pass(5, 4, true, false, false)),
    alone(pass(5, 4, true, false, true)),
    alone(pass(5, 0, true, true, false)),
    ShaderKind {
        first: pass(0, 4, true, false, true),
        second: Some(pass(5, 4, true, false, true)),
    },
];

/// Whether each batch is drawn by a program made for its kind (`ProgramKind::Made`): unless
/// `RC2K_SHADER_KINDS=0`, read once.
fn kinds_wanted() -> bool {
    static WANTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *WANTED.get_or_init(|| std::env::var("RC2K_SHADER_KINDS").map_or(true, |v| v != "0"))
}

/// Where a pass's `PixelValues` go.
#[derive(Default, Clone)]
struct ValueLocations {
    fog_colour: i32,
    alpha_reference: i32,
    alpha_function: i32,
    mixing: i32,
    flags: i32,
}

/// The sizes whose textures one draw can read together: a stage's are all one of these.
/// A texture of another size has an array of its own, drawn alone.
const MAIN_SIZES: [(usize, usize); 2] = [(128, 128), (256, 256)];

/// Layers a new array has room for; it doubles when full.
const FIRST_LAYERS: i32 = 16;

/// The textures of one size, each a layer of one GL texture array: batches that differ only
/// in their textures are then one draw (`Batches::of_layered`), the driver's cost per draw
/// being most of the GL thread's.
struct TextureArray {
    name: u32,
    capacity: i32,
    /// Layers below it have been handed out.
    next: i32,
    /// Layers handed out and given back.
    free: Vec<i32>,
    /// Whether the smaller levels are allocated (and kept for each uploaded layer): only once a
    /// batch asks for mipmapping, so a game that never does pays no memory for them.
    levelled: bool,
}

/// Which bound array of a pass's two a texture of `size` is read from.
fn unit_of(size: (usize, usize)) -> u16 {
    u16::from(size == MAIN_SIZES[1])
}

/// Whether textures `a` and `b` can be read in one draw.
fn same_array(a: &TextureNote, b: &TextureNote) -> bool {
    let (a, b) = ((a.width, a.height), (b.width, b.height));
    a == b || (MAIN_SIZES.contains(&a) && MAIN_SIZES.contains(&b))
}

struct CachedTexture {
    /// Its layer of its size's `TextureArray`.
    layer: i32,
    width: usize,
    height: usize,
}

pub struct Gpu {
    gl: Gles,
    /// The diagnostic cuts in force for the flush being drawn (the live set, read once per
    /// flush), none unless asked for.
    cuts: Cuts,
    /// The scene's programs made so far. The first is the one for every batch with no cut,
    /// which the others fall back to.
    programs: Vec<SceneProgram>,
    /// Which of `programs` draws a kind under the shader cuts of the given bits: made on
    /// first use, or (it did not build) another's.
    program_numbers: HashMap<(u32, ProgramKind), usize>,
    overlay: Quad,
    canvas_quad: Quad,
    target_quad: Quad,
    /// The frame's picture with a menu's larger 3D (`MERGE_FS`).
    merge_quad: Quad,
    vertices: u32,
    /// The vertices' second passes (`Batches::seconds`).
    seconds: u32,
    /// The vertices' passes' arrays and layers (`Batches::vertex_textures`).
    layers: u32,
    /// For copying an array's layers into a bigger one.
    copy_framebuffer: u32,
    arrays: HashMap<(usize, usize), TextureArray>,
    /// The arrays hold their smaller levels (from the first batch that asked for them on).
    mip_levels: bool,
    empty_array: u32,
    /// The menu's wash texture (its size and number), once one has been sent.
    wash: Option<(u32, (i32, i32))>,
    scene_array: u32,
    samplers: [u32; 8],
    framebuffer: u32,
    colour: u32,
    depth: u32,
    /// The target a 3D drawn smaller than the screen goes into (`render_scale`): the screen's
    /// size, the picture in its bottom-left corner; made when first needed.
    small: Option<SmallTarget>,
    /// The targets of a menu's larger 3D, made when first needed.
    high: Option<HighTarget>,
    /// Which of its `colour` holds the frame in progress, once a part of the frame has drawn 3D
    /// there.
    frame_high: Option<usize>,
    /// The canvas's textures, taken in turn: a flush's rows go into the one after the last
    /// flush's, so an upload never waits for the GPU to finish a frame still reading its texture.
    canvases: [CanvasTexture; CANVAS_TEXTURES],
    /// The one of `canvases` the last upload went into.
    canvas_now: usize,
    /// The 2D layer's atlas, program and background (`layer2d`).
    layer2d: layer2d::Layer2d,
    width: i32,
    height: i32,
    depth_is_new: bool,
    /// The programs of the start are made: one made from here on holds a frame up.
    set_up_done: bool,
    textures: super::TextureMap<CachedTexture>,
}

/// The targets of a menu's 3D drawn larger than the menu's frame, so that it is sharp on a window
/// larger than the frame (`render::menu_high_percent`).
struct HighTarget {
    /// The picture of the frame with the 3D drawn so far, one of them being drawn into while the
    /// other is read (`Gles::frame_high`); both with `depth`.
    framebuffer: [u32; 2],
    colour: [u32; 2],
    depth: u32,
    /// The frame's colour as it was right after the 3D was drawn into it (`MERGE_FS`).
    kept_framebuffer: u32,
    kept: u32,
    /// The size of the larger ones.
    size: (i32, i32),
    /// The size of `kept`.
    kept_size: (i32, i32),
    depth_is_new: bool,
}

/// The frame buffer object of a 3D drawn smaller, and the size it was made at.
struct SmallTarget {
    framebuffer: u32,
    colour: u32,
    depth: u32,
    size: (i32, i32),
    depth_is_new: bool,
}

/// How many textures the canvas's uploads take turns in: more than the frames the GPU may
/// still have to draw when the next one's rows arrive.
const CANVAS_TEXTURES: usize = 4;

/// One of the canvas's textures, and the size it was made at.
#[derive(Clone, Copy, Default)]
struct CanvasTexture {
    texture: u32,
    size: (i32, i32),
}

pub static GPU: Mutex<Option<Front>> = Mutex::new(None);

/// Whether this is a phone whose 3D goes on the software rasteriser: never where this renderer
/// stands in for the GPU one (the handheld).
#[cfg(handheld)]
pub fn software_phone() -> bool {
    false
}

/// Whether the GL is to be tried: unless `--renderer software` or `RC2K_RENDERER=software` (on a
/// PC also not when another renderer is the only one asked for).
pub fn wanted() -> bool {
    #[cfg(handheld)]
    return super::choice::gpu_wanted();
    #[cfg(not(handheld))]
    super::choice::gl_wanted()
}

impl Gpu {
    fn new(open: impl FnOnce() -> Result<Gles, Failed>) -> Result<Gpu, Failed> {
        let gl = open()?;
        log(&format!("render: the GL is {}", gl.version()));
        if super::choice::forced_failure(FAILURE_STEP) {
            return Err(("made to fail".into(), gl.close_after_failure()));
        }
        if let Some(line) = Cuts::start_from_env() {
            log(&line);
        }
        let mut gpu = Gpu {
            cuts: Cuts::default(),
            programs: Vec::new(),
            program_numbers: HashMap::new(),
            overlay: Quad {
                program: 0,
                flip: 0,
                fit: -1,
                bars: -1,
                wash: -1,
                has_wash: -1,
            },
            canvas_quad: Quad {
                program: 0,
                flip: 0,
                fit: -1,
                bars: -1,
                wash: -1,
                has_wash: -1,
            },
            target_quad: Quad {
                program: 0,
                flip: 0,
                fit: -1,
                bars: -1,
                wash: -1,
                has_wash: -1,
            },
            merge_quad: Quad {
                program: 0,
                flip: 0,
                fit: -1,
                bars: -1,
                wash: -1,
                has_wash: -1,
            },
            vertices: 0,
            seconds: 0,
            layers: 0,
            copy_framebuffer: 0,
            arrays: HashMap::new(),
            mip_levels: false,
            empty_array: 0,
            wash: None,
            scene_array: 0,
            samplers: [0; 8],
            framebuffer: 0,
            small: None,
            high: None,
            frame_high: None,
            colour: 0,
            depth: 0,
            canvases: [CanvasTexture::default(); CANVAS_TEXTURES],
            canvas_now: 0,
            layer2d: Default::default(),
            width: 0,
            height: 0,
            depth_is_new: true,
            set_up_done: false,
            textures: Default::default(),
            gl,
        };
        if let Err(why) = gpu.set_up() {
            return Err((why, gpu.gl.close_after_failure()));
        }
        gpu.set_up_done = true;
        Ok(gpu)
    }

    fn set_up(&mut self) -> Result<(), String> {
        let for_any = ProgramKind::Any {
            discard_free: false,
        };
        let first = self.scene_program(Cuts::default(), for_any)?;
        self.programs.push(first);
        self.program_numbers.insert((0, for_any), 0);
        // Made now, so that no frame that needs one waits for the compiler.
        let started = std::time::Instant::now();
        if kinds_wanted() {
            for kind in KINDS_AT_START {
                self.program_number(Cuts::default(), ProgramKind::Made(kind));
            }
            log(&format!(
                "render: {} scene programs made for the kinds of batch in {:.0} ms",
                KINDS_AT_START.len(),
                started.elapsed().as_secs_f64() * 1000.0
            ));
        } else {
            self.program_number(Cuts::default(), ProgramKind::Any { discard_free: true });
        }
        self.overlay = self.quad(QUAD_VS, OVERLAY_FS)?;
        self.canvas_quad = self.quad(FIT_VS, CANVAS_FS)?;
        self.target_quad = self.quad(FIT_VS, TARGET_FS)?;
        self.merge_quad = self.quad(FIT_VS, MERGE_FS)?;
        // The merge reads the frame's picture on unit 0, the wash on 1, the kept one on 2 and the
        // larger 3D on 3.
        self.gl.use_program(self.merge_quad.program);
        self.gl
            .uniform_1i(self.uniform(self.merge_quad.program, "kept"), 2);
        self.gl
            .uniform_1i(self.uniform(self.merge_quad.program, "high"), 3);
        let gl = &self.gl;
        // The attribute layout is `batches::Vertex`'s eight dwords.
        self.empty_array = gl.gen_vertex_array();
        self.scene_array = gl.gen_vertex_array();
        self.vertices = gl.gen_buffer();
        self.seconds = gl.gen_buffer();
        self.layers = gl.gen_buffer();
        gl.bind_vertex_array(self.scene_array);
        gl.bind_buffer(ARRAY_BUFFER, self.vertices);
        for (n, count, kind, normal, offset) in [
            (0u32, 4, 0x1406u32, false, 0usize),
            (1, 4, UNSIGNED_BYTE, true, 16),
            (2, 4, UNSIGNED_BYTE, true, 20),
            (3, 2, 0x1406, false, 24),
        ] {
            gl.attribute(n, count, kind, normal, 32, offset);
        }
        // The second passes' colour, specular and texture place: `Batches::seconds`.
        gl.bind_buffer(ARRAY_BUFFER, self.seconds);
        for (n, count, kind, normal, offset) in [
            (4u32, 4, UNSIGNED_BYTE, true, 0usize),
            (5, 4, UNSIGNED_BYTE, true, 4),
            (6, 2, 0x1406, false, 8),
        ] {
            gl.attribute(n, count, kind, normal, 16, offset);
        }
        // Each pass's array (of its two) and layer, as four numbers.
        gl.bind_buffer(ARRAY_BUFFER, self.layers);
        gl.attribute(7, 4, UNSIGNED_SHORT, false, 8, 0);
        self.samplers = gl.gen_samplers();
        self.set_filters(false, false);
        self.framebuffer = gl.gen_framebuffer();
        self.colour = gl.gen_texture();
        for canvas in &mut self.canvases {
            canvas.texture = gl.gen_texture();
        }
        self.depth = gl.gen_renderbuffer();
        self.copy_framebuffer = gl.gen_framebuffer();
        self.layer2d = self.make_layer2d()?;
        let gl = &self.gl;
        gl.front_face(0x0900); // the shown ones are clockwise on the screen
        // Made now: a draw binds both whichever it reads.
        for size in MAIN_SIZES {
            let array = self.make_array(size, FIRST_LAYERS);
            self.arrays.insert(size, array);
        }
        Ok(())
    }
}

/// A pass's values into `at` of the program in use, each only when it differs from what the
/// program holds (`held`, which then holds them): the batches differ in one or two.
fn set_values(gl: &Gles, at: &ValueLocations, held: &mut Option<PixelValues>, v: &PixelValues) {
    let was = held.as_ref();
    if was.is_none_or(|w| w.fog_colour != v.fog_colour) {
        gl.uniform_4fv(at.fog_colour, &v.fog_colour);
    }
    if was.is_none_or(|w| w.alpha_reference != v.alpha_reference) {
        gl.uniform_1f(at.alpha_reference, v.alpha_reference);
    }
    if was.is_none_or(|w| w.alpha_function != v.alpha_function) {
        gl.uniform_1i(at.alpha_function, v.alpha_function);
    }
    if was.is_none_or(|w| w.mixing != v.mixing) {
        gl.uniform_1i(at.mixing, v.mixing);
    }
    if was.is_none_or(|w| w.flags != v.flags) {
        gl.uniform_1i(at.flags, v.flags);
    }
    *held = Some(*v);
}

/// Logs a step of the GL thread's that took long (the first 300 times): which, how long and how
/// many bytes it sent, to find what makes a frame run over.
fn note_if_slow(what: &str, since: std::time::Instant, bytes: usize) {
    static LOGGED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let took = since.elapsed();
    if took.as_micros() > 4000 && LOGGED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 300 {
        log(&format!(
            "render: GL step slow: {what} took {:.1} ms for {} KB",
            took.as_secs_f64() * 1000.0,
            bytes / 1024
        ));
    }
}

/// Every pixel of the canvas marked as not written.
pub fn mark_unwritten(canvas: &mut [u8]) {
    canvas.as_chunks_mut::<4>().0.fill(UNWRITTEN.to_le_bytes());
}

fn blend_factor(factor: u32) -> u32 {
    // As the rasteriser's `factor`.
    match factor {
        1 => 0,
        3 => 0x0300,
        4 => 0x0301,
        5 => 0x0302,
        6 => 0x0303,
        7 => 0x0304,
        8 => 0x0305,
        9 => 0x0306,
        10 => 0x0307,
        _ => 1,
    }
}
