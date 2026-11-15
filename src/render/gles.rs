//! The GPU backend on the handheld: OpenGL ES 3 through the system's `libmali` (no Vulkan driver
//! there, and SDL has no display for it), drawing to `/dev/fb0` through its EGL. The same frame
//! as `gpu`'s: `batches`' lists, and before each part's 3D the retained background, the canvas's
//! written pixels and the 2D quads (`layer2d`) put over the target. The GL's own thread holds
//! the context and does the driver's work (a quarter of a frame's time on this processor); the
//! game's thread hands it copies, by channel, and goes on. They pass a batching thread on the
//! way (`gl_batching`), which makes each part's batches while the GL draws the part before.
//! The calls into the library are `ffi::gl`'s.

mod bench;
mod layer2d;

use super::batches::{Batches, NO_TEXTURE, PipelineKey, PixelValues};
use super::gl_batching::{Background, Batched, Job, Part, Written};
use super::gpu_cut::Cuts;
use super::{CommandList, Target, TextureNote};
use super::{background, canvas_rows};
use crate::gfx::draw2d::quads;
use crate::logfile::program_log::log;
use crate::platform::ffi::gl::Gles;
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
const UNSIGNED_SHORT: u32 = 0x1403;
const RGBA: u32 = 0x1908;
const UNSIGNED_BYTE: u32 = 0x1401;
const FRAMEBUFFER: u32 = 0x8D40;
const ARRAY_BUFFER: u32 = 0x8892;
const DEPTH_TEST: u32 = 0x0B71;
const BLEND: u32 = 0x0BE2;
const CULL_FACE: u32 = 0x0B44;
const SCISSOR_TEST: u32 = 0x0C11;
const TRIANGLES: u32 = 4;
const COLOUR_BIT: u32 = 0x4000;
const DEPTH_BIT: u32 = 0x0100;

/// The panel.
const PANEL: (i32, i32) = (720, 480);

// The game's screen in clip space: as `scene.vert`. There is no `noperspective` in ES 3.0: a
// value times W, interpolated with W and divided by it, comes out straight across the screen.
const SCENE_VS: &CStr = c"#version 300 es
layout(location=0) in vec4 place; layout(location=1) in vec4 colour;
layout(location=2) in vec4 specular; layout(location=3) in vec2 uv;
layout(location=4) in vec4 colour2; layout(location=5) in vec4 specular2;
layout(location=6) in vec2 uv2; layout(location=7) in vec4 layers;
uniform vec2 size;
out highp vec4 v_colour; out highp float v_fog; out highp float v_w; out highp vec2 v_uv;
out highp vec4 v_colour2; out highp float v_fog2; out highp vec2 v_uv2;
flat out lowp int v_unit; flat out highp float v_layer;
flat out lowp int v_unit2; flat out highp float v_layer2;
void main() {
    v_unit = int(layers.x); v_layer = layers.y; v_unit2 = int(layers.z); v_layer2 = layers.w;
    float w = place.w > 0.0 ? 1.0 / place.w : 1.0;
    gl_Position = vec4(((place.x + 0.5) / size.x * 2.0 - 1.0) * w,
                       (1.0 - (place.y + 0.5) / size.y * 2.0) * w,
                       (clamp(place.z, 0.0, 1.0) * 2.0 - 1.0) * w, w);
    v_colour = colour.bgra * w; v_fog = specular.a * w; v_w = w; v_uv = uv;
    v_colour2 = colour2.bgra * w; v_fog2 = specular2.a * w; v_uv2 = uv2;
}
";
/// A batch's pass; with `SECOND` in `flags`, a second pass blended over it by its alpha as the
/// target's blend would (`batches::Second`), the first's colour first held in bytes as the
/// target holds it.
const SCENE_FS: &CStr = c"#version 300 es
precision mediump float;
in highp vec4 v_colour; in highp float v_fog; in highp float v_w; in highp vec2 v_uv;
in highp vec4 v_colour2; in highp float v_fog2; in highp vec2 v_uv2;
flat in lowp int v_unit; flat in highp float v_layer;
flat in lowp int v_unit2; flat in highp float v_layer2;
out vec4 result;
// The textures are layers of arrays (`TextureArray`), two bound for each pass: the vertex
// says which and which layer. One level only and the same filter near and far, so the
// level of detail `texture` would work out changes nothing.
precision lowp sampler2DArray;
uniform sampler2DArray picture_a; uniform sampler2DArray picture_b;
uniform vec4 fog_colour; uniform float alpha_reference;
uniform int alpha_function; uniform int mixing; uniform int flags;
uniform sampler2DArray picture2_a; uniform sampler2DArray picture2_b;
uniform vec4 fog_colour2; uniform float alpha_reference2;
uniform int alpha_function2; uniform int mixing2; uniform int flags2;
vec4 fetch(sampler2DArray a, sampler2DArray b, int unit, highp vec3 at) {
#ifdef CUT_TEX
    return vec4(0.5);
#endif
#ifdef CUT_TEXHOT
    at = vec3(0.5, 0.5, at.z);
#endif
    return unit == 0 ? textureLod(a, at, 0.0) : textureLod(b, at, 0.0);
}
bool passes(int function, float reference, float value) {
#ifdef CUT_ALPHA
    return true;
#endif
    if (function == 1) return false;
    if (function == 2) return value < reference;
    if (function == 3) return value == reference;
    if (function == 4) return value <= reference;
    if (function == 5) return value > reference;
    if (function == 6) return value != reference;
    if (function == 7) return value >= reference;
    return true;
}
vec4 mixed(vec4 colour, vec4 texel, int mixing, int flags) {
    if (mixing == 1 || mixing == 7) return texel;
    if (mixing == 4) return texel * colour;
    return vec4(texel.rgb * colour.rgb, (flags & 2) != 0 ? texel.a : colour.a);
}
vec4 fogged(vec4 c, float fog, vec4 fog_colour, int flags) {
#ifdef CUT_FOG
    return c;
#endif
    if ((flags & 4) == 0) return c;
    float kept = clamp(fog, 0.0, 1.0);
    return vec4(c.rgb * kept + fog_colour.rgb * (1.0 - kept), c.a);
}
#ifdef CUT_SECOND
#define SECOND_PASS false
#else
#define SECOND_PASS ((flags & 8) != 0)
#endif
void main() {
    vec4 colour = v_colour / v_w;
    vec4 c = colour;
#ifdef CUT_FLAT
    result = c;
    return;
#endif
    // uploaded as they are: blue first
    if ((flags & 1) != 0) {
        vec4 texel = fetch(picture_a, picture_b, v_unit, vec3(v_uv, v_layer)).bgra;
        c = mixed(colour, texel, mixing, flags);
    }
    if (alpha_function != 0 && !passes(alpha_function, alpha_reference, c.a)) discard;
    c = fogged(c, v_fog / v_w, fog_colour, flags);
    if (SECOND_PASS) {
        vec4 under = floor(c * 255.0 + 0.5) / 255.0;
        vec4 colour2 = v_colour2 / v_w;
        vec4 d = colour2;
        if ((flags2 & 1) != 0) {
            vec4 texel = fetch(picture2_a, picture2_b, v_unit2, vec3(v_uv2, v_layer2)).bgra;
            d = mixed(colour2, texel, mixing2, flags2);
        }
        if (alpha_function2 == 0 || passes(alpha_function2, alpha_reference2, d.a)) {
            d = fogged(d, v_fog2 / v_w, fog_colour2, flags2);
            c = vec4(d.rgb * d.a + under.rgb * (1.0 - d.a), d.a * d.a + under.a * (1.0 - d.a));
        } else {
            c = under;
        }
    }
    result = c;
}
";
/// A picture over a rectangle of the target: `flip` for what the GL's own origin has upside
/// down (the frame buffer object's).
const QUAD_VS: &CStr = c"#version 300 es
uniform float flip;
out vec2 v_uv;
void main() {
    vec2 corner = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
    v_uv = vec2(corner.x, flip > 0.5 ? 1.0 - corner.y : corner.y);
    gl_Position = vec4(corner.x * 2.0 - 1.0, 1.0 - corner.y * 2.0, 0.0, 1.0);
}
";
/// The canvas's written pixels only.
const OVERLAY_FS: &CStr = c"#version 300 es
precision mediump float;
in vec2 v_uv; out vec4 result; uniform sampler2D canvas;
void main() {
    vec4 pixel = texture(canvas, v_uv); // blue first
    uvec4 bytes = uvec4(pixel * 255.0 + 0.5);
    if (bytes == uvec4(0xFF, 0x00, 0xFF, 0xA5)) discard;
    result = vec4(pixel.bgr, 1.0);
}
";
/// A whole picture of the game's bytes (blue first).
const CANVAS_FS: &CStr = c"#version 300 es
precision mediump float;
in vec2 v_uv; out vec4 result; uniform sampler2D canvas;
void main() { result = vec4(texture(canvas, v_uv).bgr, 1.0); }
";
/// The frame buffer object's picture (red first).
const TARGET_FS: &CStr = c"#version 300 es
precision mediump float;
in vec2 v_uv; out vec4 result; uniform sampler2D canvas;
void main() { result = vec4(texture(canvas, v_uv).rgb, 1.0); }
";

struct Quad {
    program: u32,
    flip: i32,
}

#[derive(Clone)]
struct Locations {
    size: i32,
    first: ValueLocations,
    second: ValueLocations,
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
    scene: u32,
    locations: Locations,
    /// The scene programs made so far, by the shader cuts they were made for.
    scene_programs: HashMap<u32, (u32, Locations)>,
    overlay: Quad,
    canvas_quad: Quad,
    target_quad: Quad,
    vertices: u32,
    /// The vertices' second passes (`Batches::seconds`).
    seconds: u32,
    /// The vertices' passes' arrays and layers (`Batches::vertex_textures`).
    layers: u32,
    /// For copying an array's layers into a bigger one.
    copy_framebuffer: u32,
    arrays: HashMap<(usize, usize), TextureArray>,
    empty_array: u32,
    scene_array: u32,
    samplers: [u32; 4],
    framebuffer: u32,
    colour: u32,
    depth: u32,
    canvas: u32,
    canvas_size: (i32, i32),
    /// The 2D layer's atlas, program and background (`layer2d`).
    layer2d: layer2d::Layer2d,
    width: i32,
    height: i32,
    depth_is_new: bool,
    textures: super::TextureMap<CachedTexture>,
}

pub static GPU: Mutex<Option<Front>> = Mutex::new(None);

/// Whether the GPU is to be tried: unless `RC2K_RENDERER=software`.
pub fn wanted() -> bool {
    super::choice::gpu_wanted()
}

impl Gpu {
    fn new() -> Result<Gpu, String> {
        if super::choice::forced_failure("gles") {
            return Err("made to fail".into());
        }
        let gl = Gles::open_panel(PANEL.0 as u16, PANEL.1 as u16)?;
        if let Some(line) = Cuts::start_from_env() {
            log(&line);
        }
        let mut gpu = Gpu {
            cuts: Cuts::default(),
            scene_programs: HashMap::new(),
            scene: 0,
            locations: Locations {
                size: 0,
                first: ValueLocations::default(),
                second: ValueLocations::default(),
            },
            overlay: Quad {
                program: 0,
                flip: 0,
            },
            canvas_quad: Quad {
                program: 0,
                flip: 0,
            },
            target_quad: Quad {
                program: 0,
                flip: 0,
            },
            vertices: 0,
            seconds: 0,
            layers: 0,
            copy_framebuffer: 0,
            arrays: HashMap::new(),
            empty_array: 0,
            scene_array: 0,
            samplers: [0; 4],
            framebuffer: 0,
            colour: 0,
            depth: 0,
            canvas: 0,
            canvas_size: (0, 0),
            layer2d: Default::default(),
            width: 0,
            height: 0,
            depth_is_new: true,
            textures: Default::default(),
            gl,
        };
        gpu.set_up()?;
        Ok(gpu)
    }

    fn program(&self, vertex: &CStr, fragment: &CStr) -> Result<u32, String> {
        let (v, f) = (
            self.gl.compile_shader(0x8B31, vertex)?,
            self.gl.compile_shader(0x8B30, fragment)?,
        );
        Ok(self.gl.link_program(v, f))
    }

    fn uniform(&self, program: u32, name: &str) -> i32 {
        self.gl.uniform_location(program, name)
    }

    fn quad(&self, fragment: &CStr) -> Result<Quad, String> {
        let program = self.program(QUAD_VS, fragment)?;
        Ok(Quad {
            program,
            flip: self.uniform(program, "flip"),
        })
    }

    /// The scene's program with the shader cuts of `cuts` defined, and where its values go;
    /// its picture units are set.
    fn scene_program(&self, cuts: Cuts) -> Result<(u32, Locations), String> {
        let fragment = std::ffi::CString::new(cuts.shader(&SCENE_FS.to_string_lossy()))
            .map_err(|_| "the scene's shader holds a NUL".to_string())?;
        let program = self.program(SCENE_VS, &fragment)?;
        let values = |suffix: &str| ValueLocations {
            fog_colour: self.uniform(program, &format!("fog_colour{suffix}")),
            alpha_reference: self.uniform(program, &format!("alpha_reference{suffix}")),
            alpha_function: self.uniform(program, &format!("alpha_function{suffix}")),
            mixing: self.uniform(program, &format!("mixing{suffix}")),
            flags: self.uniform(program, &format!("flags{suffix}")),
        };
        let locations = Locations {
            size: self.uniform(program, "size"),
            first: values(""),
            second: values("2"),
        };
        self.gl.use_program(program);
        for (unit, name) in ["picture_a", "picture_b", "picture2_a", "picture2_b"]
            .into_iter()
            .enumerate()
        {
            self.gl.uniform_1i(self.uniform(program, name), unit as i32);
        }
        Ok((program, locations))
    }

    /// Takes the live cuts for the flush about to be drawn: the scene's program for its shader
    /// cuts (made on first use; the plain one if that fails) and the samplers' filters.
    fn take_cuts(&mut self) {
        let cuts = super::gpu_cut::live();
        if cuts == self.cuts {
            return;
        }
        let key = cuts.shader_part().bits();
        if !self.scene_programs.contains_key(&key) {
            let made = self.scene_program(cuts.shader_part());
            let entry = made.unwrap_or_else(|why| {
                log(&format!(
                    "render: the scene's shader with cuts {} failed: {why}",
                    cuts.names()
                ));
                self.scene_programs[&0].clone()
            });
            self.scene_programs.insert(key, entry);
        }
        let (scene, locations) = self.scene_programs[&key].clone();
        self.scene = scene;
        self.locations = locations;
        self.set_filters(cuts.nearest, cuts.linear);
        log(&format!("render: GPU cuts now on: {}", cuts.names()));
        self.cuts = cuts;
    }

    /// Every sampler's filter: nearest for the even ones and linear for the odd, unless a cut
    /// makes all one or the other.
    fn set_filters(&self, nearest: bool, linear_cut: bool) {
        for (n, &sampler) in self.samplers.iter().enumerate() {
            let linear = (n & 1 != 0 || linear_cut) && !nearest;
            let filter = if linear { 0x2601 } else { 0x2600 };
            let wrap = if n & 2 != 0 { 0x812F } else { 0x2901 };
            self.gl.sampler_parameter(sampler, 0x2801, filter);
            self.gl.sampler_parameter(sampler, 0x2800, filter);
            self.gl.sampler_parameter(sampler, 0x2802, wrap);
            self.gl.sampler_parameter(sampler, 0x2803, wrap);
        }
    }

    fn set_up(&mut self) -> Result<(), String> {
        let (scene, locations) = self.scene_program(Cuts::default())?;
        self.scene = scene;
        self.locations = locations;
        self.scene_programs
            .insert(0, (scene, self.locations.clone()));
        self.overlay = self.quad(OVERLAY_FS)?;
        self.canvas_quad = self.quad(CANVAS_FS)?;
        self.target_quad = self.quad(TARGET_FS)?;
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
        self.canvas = gl.gen_texture();
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

    /// A new array of `capacity` layers of `size`, bound on the active unit.
    fn make_array(&self, (width, height): (usize, usize), capacity: i32) -> TextureArray {
        let gl = &self.gl;
        let name = gl.gen_texture();
        gl.bind_texture(TEXTURE_2D_ARRAY, name);
        gl.tex_image_3d_empty(0x8058, width as i32, height as i32, capacity);
        // No levels: one the sampler objects' filters read alone.
        gl.tex_parameter(TEXTURE_2D_ARRAY, 0x813D, 0);
        TextureArray {
            name,
            capacity,
            next: 0,
            free: Vec::new(),
        }
    }

    /// A layer of the array of `size` for a new texture: one given back, a new one, or (the
    /// array full) one of a new array twice its size, the old one's layers copied into it.
    fn new_layer(&mut self, size: (usize, usize)) -> i32 {
        if !self.arrays.contains_key(&size) {
            let array = self.make_array(size, FIRST_LAYERS);
            self.arrays.insert(size, array);
        }
        let full = self
            .arrays
            .get(&size)
            .is_some_and(|a| a.free.is_empty() && a.next == a.capacity);
        if full {
            self.grow(size);
        }
        let Some(array) = self.arrays.get_mut(&size) else {
            unreachable!("made above");
        };
        array.free.pop().unwrap_or_else(|| {
            array.next += 1;
            array.next - 1
        })
    }

    /// The array of `size` twice as big, its layers kept.
    fn grow(&mut self, size: (usize, usize)) {
        let Some(old) = self.arrays.remove(&size) else {
            return;
        };
        let mut bigger = self.make_array(size, old.capacity * 2);
        let gl = &self.gl;
        gl.bind_framebuffer(READ_FRAMEBUFFER, self.copy_framebuffer);
        for layer in 0..old.next {
            gl.framebuffer_texture_layer(READ_FRAMEBUFFER, old.name, layer);
            gl.copy_into_layer(layer, size.0 as i32, size.1 as i32);
        }
        gl.bind_framebuffer(READ_FRAMEBUFFER, 0);
        gl.delete_texture(old.name);
        (bigger.next, bigger.free) = (old.next, old.free);
        self.arrays.insert(size, bigger);
    }

    /// The two arrays a pass whose textures are of `size` reads, onto units `unit` and
    /// `unit + 1` (the active unit left at `unit + 1`).
    fn bind_arrays(&self, unit: u32, size: (usize, usize)) {
        let name = |size| self.arrays.get(&size).map_or(0, |a| a.name);
        let (a, b) = if MAIN_SIZES.contains(&size) {
            (name(MAIN_SIZES[0]), name(MAIN_SIZES[1]))
        } else {
            (name(size), 0)
        };
        let gl = &self.gl;
        gl.active_texture(TEXTURE0 + unit);
        gl.bind_texture(TEXTURE_2D_ARRAY, a);
        gl.active_texture(TEXTURE0 + unit + 1);
        gl.bind_texture(TEXTURE_2D_ARRAY, b);
    }

    /// The frame buffer object at the screen's size.
    fn set_size(&mut self, width: i32, height: i32) -> Result<(), String> {
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        let gl = &self.gl;
        // No pixels given.
        gl.bind_texture(TEXTURE_2D, self.colour);
        gl.tex_image_2d(0x8058, width, height, RGBA, UNSIGNED_BYTE, None);
        gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2600);
        gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2600);
        gl.bind_renderbuffer(0x8D41, self.depth);
        gl.renderbuffer_storage(0x8D41, 0x81A6, width, height);
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        gl.framebuffer_texture_2d(FRAMEBUFFER, 0x8CE0, self.colour);
        gl.framebuffer_renderbuffer(FRAMEBUFFER, 0x8D00, self.depth);
        if gl.framebuffer_status(FRAMEBUFFER) != 0x8CD5 {
            return Err("the frame buffer object is not complete".into());
        }
        (self.width, self.height) = (width, height);
        self.depth_is_new = true;
        Ok(())
    }

    /// The canvas's bytes into its texture.
    fn upload_canvas(&mut self, canvas: &[u8], width: i32, height: i32) {
        let gl = &self.gl;
        gl.bind_texture(TEXTURE_2D, self.canvas);
        gl.bind_sampler(0, 0);
        if self.canvas_size != (width, height) {
            gl.tex_image_2d(0x8058, width, height, RGBA, UNSIGNED_BYTE, Some(canvas));
            gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2600);
            gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2600);
            self.canvas_size = (width, height);
        } else {
            gl.tex_sub_image_2d((0, 0), (width, height), RGBA, UNSIGNED_BYTE, canvas);
        }
    }

    /// Rows `first ..= last` of the canvas (`rows`, those alone) into its texture. At a new
    /// size the texture is made first, with no pixels: what it holds outside the rows is cut off
    /// by the overlay's scissor.
    fn upload_canvas_rows(
        &mut self,
        rows: &[u8],
        width: i32,
        height: i32,
        first: usize,
        last: usize,
    ) {
        let gl = &self.gl;
        gl.bind_texture(TEXTURE_2D, self.canvas);
        gl.bind_sampler(0, 0);
        if self.canvas_size != (width, height) {
            gl.tex_image_2d(0x8058, width, height, RGBA, UNSIGNED_BYTE, None);
            gl.tex_parameter(TEXTURE_2D, 0x2801, 0x2600);
            gl.tex_parameter(TEXTURE_2D, 0x2800, 0x2600);
            self.canvas_size = (width, height);
        }
        gl.tex_sub_image_2d(
            (0, first as i32),
            (width, (last - first + 1) as i32),
            RGBA,
            UNSIGNED_BYTE,
            rows,
        );
    }

    /// Everything a quad needs off.
    fn plain_state(&self) {
        let gl = &self.gl;
        gl.disable(DEPTH_TEST);
        gl.disable(BLEND);
        gl.disable(CULL_FACE);
        gl.disable(SCISSOR_TEST);
        gl.colour_mask(true);
        gl.bind_vertex_array(self.empty_array);
    }

    fn draw_quad(&self, quad: &Quad, flip: bool) {
        let gl = &self.gl;
        // A program made in `set_up`; three vertices of no attributes.
        gl.use_program(quad.program);
        gl.uniform_1f(quad.flip, if flip { 1.0 } else { 0.0 });
        gl.draw_arrays(TRIANGLES, 0, 3);
    }

    /// A frame's part: the textures brought up, the canvas's written rows over the target, then
    /// the batches.
    fn draw(&mut self, batched: Batched) {
        let Batched {
            part:
                Part {
                    commands,
                    uploads,
                    written,
                    quads,
                    background,
                    size: (width, height),
                },
            batches,
            quad_vertices,
            opaque,
        } = batched;
        self.take_cuts();
        if let Err(why) = self.set_size(width, height) {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput(Box::leak(why.into_boxed_str())),
            );
        }
        let started = std::time::Instant::now();
        for (note, bytes) in &uploads {
            self.bring_texture_up(note, bytes);
        }
        note_if_slow(
            "textures",
            started,
            uploads.iter().map(|(_, b)| b.len()).sum(),
        );
        super::GPU_DRAWS.fetch_add(
            batches.batches.len() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        super::GPU_FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if !written.is_empty() {
            super::CANVAS_UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        let started = std::time::Instant::now();
        for w in &written {
            self.upload_canvas_rows(&w.bytes, width, height, w.first, w.last);
            super::CANVAS_UPLOAD_BYTES
                .fetch_add(w.bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);
        }
        note_if_slow(
            "canvas rows",
            started,
            written.iter().map(|w| w.bytes.len()).sum(),
        );
        let started = std::time::Instant::now();
        self.send_atlas(&quads.uploads);
        note_if_slow(
            "atlas",
            started,
            quads.uploads.iter().map(|u| u.texels.len() * 4).sum(),
        );
        let started = std::time::Instant::now();
        if let Some(background) = &background {
            self.send_background(background, (width, height));
        }
        note_if_slow("background", started, 0);
        let started = std::time::Instant::now();
        self.send_quad_vertices(&quad_vertices);
        note_if_slow("quad vertices", started, quad_vertices.len() * 24);
        self.plain_state();
        let gl = &self.gl;
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        // `half`: a quarter of the target, a quarter of the fragments.
        let (view_width, view_height) = if self.cuts.half {
            (width / 2, height / 2)
        } else {
            (width, height)
        };
        gl.viewport(0, 0, view_width, view_height);
        if self.depth_is_new {
            gl.depth_mask(true);
            gl.clear_depth(1.0);
            gl.clear(DEPTH_BIT);
            self.depth_is_new = false;
        }
        // Up in clip space is the picture's top for the 2D as for the 3D; the object's
        // last row is that top, which `show_target` and `bring_back` turn over.
        if background.is_some() && !self.cuts.two_d {
            self.draw_background((width, height));
        }
        if !written.is_empty() && !self.cuts.two_d {
            gl.enable(SCISSOR_TEST);
            for w in &written {
                gl.bind_texture(TEXTURE_2D, self.canvas);
                let rows = (w.last - w.first + 1) as i32;
                gl.scissor(0, height - w.last as i32 - 1, width, rows);
                self.draw_quad(&self.overlay, false);
            }
            gl.disable(SCISSOR_TEST);
        }
        if !self.cuts.two_d {
            self.draw_quads(quad_vertices.len(), (width, height));
        }
        if !batches.batches.is_empty() && !self.cuts.scene {
            gl.use_program(self.scene);
            gl.uniform_2f(self.locations.size, width as f32, height as f32);
            gl.bind_vertex_array(self.scene_array);
            gl.bind_buffer(ARRAY_BUFFER, self.vertices);
            gl.buffer_data(ARRAY_BUFFER, &batches.vertices, 0x88E0);
            gl.bind_buffer(ARRAY_BUFFER, self.seconds);
            gl.buffer_data(ARRAY_BUFFER, &batches.seconds, 0x88E0);
            gl.bind_buffer(ARRAY_BUFFER, self.layers);
            gl.buffer_data(ARRAY_BUFFER, &self.vertex_layers(&batches), 0x88E0);
            if !self.cuts.depth {
                gl.enable(DEPTH_TEST);
            }
            let mut last_pipeline = None;
            let mut last_values = None;
            let mut last_arrays = None;
            let mut last_sampler = u32::MAX;
            let mut last_second = None;
            let size = |n: Option<u32>| {
                n.map(|n| {
                    let note = &batches.textures[n as usize];
                    (note.width, note.height)
                })
            };
            let mut scissored = false;
            for batch in &batches.batches {
                if self.cuts.drops(batch.pipeline.blend.is_some()) {
                    continue;
                }
                if last_pipeline != Some(batch.pipeline) {
                    self.set_pipeline(&batch.pipeline, last_pipeline.as_ref());
                    last_pipeline = Some(batch.pipeline);
                }
                self.set_values(&self.locations.first, last_values.as_ref(), &batch.values);
                last_values = Some(batch.values);
                if let Some(second) = &batch.second
                    && last_second != Some(*second)
                {
                    let was = last_second
                        .as_ref()
                        .map(|s: &super::batches::Second| &s.values);
                    self.set_values(&self.locations.second, was, &second.values);
                    if let Some(size) = size(second.texture) {
                        self.bind_arrays(2, size);
                        gl.active_texture(TEXTURE0);
                    }
                    let sampler = self.samplers[second.linear as usize + 2 * second.clamp as usize];
                    gl.bind_sampler(2, sampler);
                    gl.bind_sampler(3, sampler);
                    last_second = Some(*second);
                }
                // A batch's textures are all read from the arrays its first one's size names.
                if let Some(size) = size(batch.texture)
                    && last_arrays != Some(size)
                {
                    self.bind_arrays(0, size);
                    gl.active_texture(TEXTURE0);
                    last_arrays = Some(size);
                }
                let sampler = self.samplers[batch.linear as usize + 2 * batch.clamp as usize];
                if sampler != last_sampler {
                    gl.bind_sampler(0, sampler);
                    gl.bind_sampler(1, sampler);
                    last_sampler = sampler;
                }
                if let Some([left, top, right, bottom]) = batch.scissor {
                    gl.enable(SCISSOR_TEST);
                    // Under `half` the rectangle is scaled with the viewport.
                    let shrink = |n: i32| if self.cuts.half { n / 2 } else { n };
                    gl.scissor(
                        shrink(left as i32),
                        shrink(height - bottom as i32),
                        shrink((right - left) as i32),
                        shrink((bottom - top) as i32),
                    );
                    scissored = true;
                } else if scissored {
                    gl.disable(SCISSOR_TEST);
                    scissored = false;
                }
                gl.draw_arrays(TRIANGLES, batch.first_vertex as i32, batch.vertices as i32);
            }
            for unit in 0..4 {
                gl.bind_sampler(unit, 0);
            }
        }
        if bench::due(batches.batches.len()) {
            self.bench(&commands, &batches, &opaque, height);
        }
        super::give_back(commands);
        super::batches::give_back_batches(batches);
    }

    /// Each vertex's passes' arrays and layers (`unit_of`, the texture's layer), two
    /// sixteen-bit numbers each, as the scene's attribute 7 reads them.
    fn vertex_layers(&self, batches: &Batches) -> Vec<[u32; 2]> {
        let places: Vec<u32> = batches
            .textures
            .iter()
            .map(|note| {
                self.textures.get(&note.pixels).map_or(0, |t| {
                    u32::from(unit_of((t.width, t.height))) | (t.layer as u32) << 16
                })
            })
            .collect();
        let place = |n: u16| {
            if n == NO_TEXTURE {
                0
            } else {
                places.get(n as usize).copied().unwrap_or(0)
            }
        };
        batches
            .vertex_textures
            .iter()
            .map(|&[first, second]| [place(first), place(second)])
            .collect()
    }

    /// A pass's values into `at`, each only when it differs from `was`'s: the batches differ
    /// in one or two. The scene's program is in use.
    fn set_values(&self, at: &ValueLocations, was: Option<&PixelValues>, v: &PixelValues) {
        let gl = &self.gl;
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
    }

    /// The key's states that differ from `was`'s (all of them without one).
    fn set_pipeline(&self, key: &PipelineKey, was: Option<&PipelineKey>) {
        let gl = &self.gl;
        let change = key.changes_from(was);
        if change.cull {
            match key.cull_face() {
                Some(face) => {
                    if was.is_none_or(|w| w.cull_face().is_none()) {
                        gl.enable(CULL_FACE);
                    }
                    gl.cull_face(face);
                }
                None => gl.disable(CULL_FACE),
            }
        }
        if change.depth_function {
            gl.depth_func(key.depth_func());
        }
        if change.depth_write {
            gl.depth_mask(key.depth_write);
        }
        if change.blend {
            match key.blend {
                // `blend` cut: never on.
                Some(_) if self.cuts.blend => gl.disable(BLEND),
                Some((source, target)) => {
                    if was.is_none_or(|w| w.blend.is_none()) {
                        gl.enable(BLEND);
                    }
                    gl.blend_func(blend_factor(source), blend_factor(target));
                }
                None => gl.disable(BLEND),
            }
        }
        if change.colour {
            gl.colour_mask(key.colour);
        }
    }

    fn bring_texture_up(&mut self, note: &TextureNote, bytes: &[u8]) {
        super::UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        super::UPLOAD_BYTES.fetch_add(bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);
        let size = (note.width, note.height);
        let known = self
            .textures
            .get(&note.pixels)
            .map(|t| ((t.width, t.height), t.layer));
        let layer = match known {
            Some((was, layer)) if was == size => layer,
            Some((was, layer)) => {
                self.give_back(was, layer);
                self.new_layer(size)
            }
            None => self.new_layer(size),
        };
        let gl = &self.gl;
        gl.bind_texture(
            TEXTURE_2D_ARRAY,
            self.arrays.get(&size).map_or(0, |a| a.name),
        );
        // `bytes` holds the whole picture (`texture_as_bgra`).
        gl.tex_sub_image_layer(layer, note.width as i32, note.height as i32, bytes);
        self.textures.insert(
            note.pixels,
            CachedTexture {
                layer,
                width: note.width,
                height: note.height,
            },
        );
    }

    /// The whole of the panel black, and the viewport where a picture of `width` x `height` shows.
    fn panel_viewport(&self, width: i32, height: i32) {
        let gl = &self.gl;
        // Shrunk to fit if it is too big, its shape kept; centred.
        let (w, h) = if width > PANEL.0 || height > PANEL.1 {
            (width * PANEL.1 / height, PANEL.1)
        } else {
            (width, height)
        };
        gl.bind_framebuffer(FRAMEBUFFER, 0);
        gl.clear_colour(0.0, 0.0, 0.0, 1.0);
        gl.clear(COLOUR_BIT);
        gl.viewport((PANEL.0 - w) / 2, (PANEL.1 - h) / 2, w, h);
    }

    /// The target on the panel.
    ///
    /// With `wait`, the GPU is waited for before the frame goes to the panel (the GPU-time
    /// diagnostic, `gpu_time`); the wait's time comes back.
    fn show_target(&mut self, wait: bool) -> Option<std::time::Duration> {
        self.plain_state();
        self.panel_viewport(self.width, self.height);
        let gl = &self.gl;
        gl.bind_sampler(0, 0);
        gl.bind_texture(TEXTURE_2D, self.colour);
        self.draw_quad(&self.target_quad, true);
        let waited = wait.then(|| {
            let start = std::time::Instant::now();
            gl.finish();
            start.elapsed()
        });
        gl.swap_buffers();
        waited
    }

    /// A picture of the game's bytes (`width` x `height`, blue first) on the panel: the frames the
    /// rasteriser made.
    fn show_picture(&mut self, frame: &[u8], width: i32, height: i32) {
        self.upload_canvas(frame, width, height);
        self.plain_state();
        self.panel_viewport(width, height);
        self.draw_quad(&self.canvas_quad, false);
        self.gl.swap_buffers();
    }

    /// The target's rows as the GL holds them.
    fn read_target(&mut self) -> Vec<u8> {
        let (width, height) = (self.width, self.height);
        let mut out = vec![0u8; width as usize * height as usize * 4];
        self.gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        self.gl
            .read_pixels(width, height, RGBA, UNSIGNED_BYTE, &mut out);
        out
    }

    /// The textures whose memory is gone let go of.
    fn forget(&mut self, gone: &[super::TextureId]) {
        for pixels in gone {
            if let Some(texture) = self.textures.remove(pixels) {
                self.give_back((texture.width, texture.height), texture.layer);
            }
        }
    }

    /// A layer of the array of `size` free again.
    fn give_back(&mut self, size: (usize, usize), layer: i32) {
        if let Some(array) = self.arrays.get_mut(&size) {
            array.free.push(layer);
        }
    }

    fn run(mut self, jobs: Receiver<Job<Batched>>) {
        let mut sampler = super::gpu_time::Sampler::from_env();
        for job in jobs {
            let start = std::time::Instant::now();
            match job {
                Job::Forget(gone) => self.forget(&gone),
                Job::Draw(part) => {
                    if let Some(sampler) = &mut sampler {
                        sampler.part_begins();
                    }
                    self.draw(part);
                }
                Job::ShowTarget => {
                    let wait = sampler
                        .as_mut()
                        .is_some_and(super::gpu_time::Sampler::frame_ends);
                    let waited = self.show_target(wait);
                    if let Some(sampler) = &mut sampler {
                        match waited {
                            Some(waited) => sampler.counted(waited),
                            None => sampler.skipped(),
                        }
                    }
                    super::GL_SHOW_MICROS.fetch_add(
                        start.elapsed().as_micros() as u64,
                        std::sync::atomic::Ordering::Relaxed,
                    );
                }
                Job::ShowPicture {
                    frame,
                    width,
                    height,
                } => self.show_picture(&frame, width, height),
                Job::ReadTarget(answer) => {
                    let _ = answer.send(self.read_target());
                }
            }
            super::GL_BUSY_MICROS.fetch_add(
                start.elapsed().as_micros() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
        }
    }
}

/// The game's side of the GL's thread, and what it knows the GPU holds.
pub struct Front {
    jobs: SyncSender<Job<Part>>,
    /// The textures up to date on the GPU, with their sizes.
    known: super::TextureMap<(usize, usize)>,
    size: (i32, i32),
    /// What the GL's background texture holds, at the screen's size.
    background_held: Option<background::Held>,
}

/// Jobs waiting for the batching thread at most: a frame's part, its end, a little more; then
/// the game waits. Past them, one being batched and `BATCHED_WAITING` batched wait for the GL:
/// with fewer (one waiting in all, handed over directly) the game waited for the GL three
/// times as long and the handheld drew about 8 % fewer frames (2026-11-13).
const JOBS_WAITING: usize = 2;

/// Parts batched and waiting for the GL's thread at most.
const BATCHED_WAITING: usize = 1;

impl Front {
    fn send(&self, job: Job<Part>) {
        let start = std::time::Instant::now();
        let sent = self.jobs.send(job);
        super::GL_WAIT_MICROS.fetch_add(
            start.elapsed().as_micros() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        if sent.is_err() {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput("the GL's thread has ended"),
            );
        }
    }

    /// Hands over the commands and the canvas's written rows (as `target` noted them), which it
    /// marks again.
    fn draw(&mut self, target: &Target, canvas: &mut [u8], commands: CommandList) {
        // A frame with only 2D in it (a menu's background and quads) has no `Backend::draw` of
        // its own: the size is the target's here.
        let size = (target.width as i32, target.height as i32);
        if size != self.size {
            // The GL's background texture is remade at the new size.
            self.background_held = None;
            self.size = size;
        }
        super::FANS.fetch_add(commands.len() as u64, std::sync::atomic::Ordering::Relaxed);
        super::FLUSHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let changes = super::take_changed_textures();
        for pixels in changes.written.iter().chain(&changes.gone) {
            self.known.remove(pixels);
        }
        if !changes.gone.is_empty() {
            self.send(Job::Forget(changes.gone.into_iter().collect()));
        }
        let (width, _) = self.size;
        let mut uploads = Vec::new();
        // Each texture the fans read once, in the order they first read them.
        for note in commands.textures() {
            if self.known.get(&note.pixels) != Some(&(note.width, note.height)) {
                uploads.push((*note, super::texture_as_bgra(note)));
                self.known.insert(note.pixels, (note.width, note.height));
            }
        }
        let quads = quads::take().fitted_to((width as u32, self.size.1 as u32));
        let background = self.take_background();
        let stride = width as usize * 4;
        let rows = canvas_rows::written_rows(target, canvas);
        let written = canvas_rows::written_runs(&rows)
            .into_iter()
            .map(|(first, last)| {
                let rows = &mut canvas[first * stride..(last + 1) * stride];
                let bytes = rows.to_vec();
                mark_unwritten(rows);
                Written { first, last, bytes }
            })
            .collect();
        self.send(Job::Draw(Part {
            commands,
            uploads,
            written,
            quads,
            background,
            size: self.size,
        }));
    }

    /// The noted background (`background`), if there is one, as the part brings it: its picture
    /// when the GL's texture does not hold it already.
    fn take_background(&mut self) -> Option<Background> {
        let pending = background::take()?;
        let bytes = self.size.0 as usize * self.size.1 as usize * 4;
        let mut picture = None;
        let sent = background::bring_up(
            pending,
            &mut self.background_held,
            bytes,
            crate::platform::direct3d::handles().pictures,
            |rows| {
                picture = Some(rows.to_vec());
                Ok(())
            },
        );
        if let Err(why) = sent {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput(Box::leak(why.into_boxed_str())),
            );
        }
        Some(picture.map_or(Background::Held, Background::Upload))
    }

    /// The frame's end: what the canvas has written over the target, and the target shown.
    pub fn finish_and_show(&mut self, target: &Target, canvas: &mut [u8]) {
        self.draw(target, canvas, CommandList::new());
        self.send(Job::ShowTarget);
    }

    /// A picture of the game's bytes on the panel.
    pub fn show_picture(&mut self, frame: &[u8], width: usize, height: usize) {
        self.send(Job::ShowPicture {
            frame: frame.to_vec(),
            width: width as i32,
            height: height as i32,
        });
    }

    /// The target's picture, with the canvas's written pixels, into the canvas (waits for the
    /// GL: the frame's whole work so far).
    pub fn bring_back(&mut self, target: &Target, canvas: &mut [u8]) {
        super::READ_BACKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.draw(target, canvas, CommandList::new());
        let (answer, picture) = channel();
        self.send(Job::ReadTarget(answer));
        let Ok(picture) = picture.recv() else {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput("the GL's thread has ended"),
            );
        };
        // Red first and bottom row first, as the GL gives them: the game's blue first, top first.
        let stride = self.size.0 as usize * 4;
        for (to, from) in canvas
            .chunks_exact_mut(stride)
            .zip(picture.rchunks_exact(stride))
        {
            for (t, f) in to
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(from.as_chunks::<4>().0)
            {
                *t = [f[2], f[1], f[0], 0];
            }
        }
    }
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

/// Starts the GL's thread if it is wanted.
pub fn start_if_wanted() {
    let mut held = GPU.lock().unwrap();
    if held.is_some() || TRIED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    if !wanted() {
        log("render: the software rasteriser draws the 3D (RC2K_RENDERER=software)");
        return;
    }
    let (jobs, to_batch) = sync_channel(JOBS_WAITING);
    let (batched, to_draw) = sync_channel(BATCHED_WAITING);
    let (answer, started) = channel();
    let thread = std::thread::Builder::new()
        .name("GL".into())
        .spawn(move || match Gpu::new() {
            Ok(gpu) => {
                let _ = answer.send(Ok(()));
                gpu.run(to_draw);
            }
            Err(why) => {
                let _ = answer.send(Err(why));
            }
        });
    let outcome = match thread {
        Ok(_) => started
            .recv()
            .unwrap_or_else(|_| Err("the GL's thread ended".into())),
        Err(why) => Err(why.to_string()),
    };
    // The GL's thread, without its batches' sender, ends when this does not start.
    let outcome = outcome.and_then(|()| {
        std::thread::Builder::new()
            .name("GL batches".into())
            .spawn(move || super::gl_batching::run(to_batch, batched, same_array, bench::enabled()))
            .map(drop)
            .map_err(|why| why.to_string())
    });
    match outcome {
        Ok(()) => {
            log("render: the GPU backend on libmali's OpenGL ES, on its own thread");
            *held = Some(Front {
                jobs,
                known: Default::default(),
                size: (0, 0),
                background_held: None,
            });
            super::gpu_started();
        }
        Err(why) => log(&format!(
            "render: the GPU backend does not start ({why}): the software rasteriser draws the 3D"
        )),
    }
}

/// It is asked for once: failing, the rasteriser goes on.
static TRIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl super::Backend for Front {
    fn draw(&mut self, target: &Target, commands: CommandList) {
        target.with_canvas(|canvas| Front::draw(self, target, canvas, commands));
    }
}
