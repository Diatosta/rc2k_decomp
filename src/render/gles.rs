//! The GPU backend on the handheld: OpenGL ES 3 through the system's `libmali` (no Vulkan driver
//! there, and SDL has no display for it), drawing to `/dev/fb0` through its EGL. The same frame
//! as `gpu`'s: `batches`' lists, the canvas's written pixels over the 3D. The GL's own thread
//! holds the context and does the driver's work (a quarter of a frame's time on this processor);
//! the game's thread hands it copies, by channel, and goes on. The calls into the
//! library are `ffi::gl`'s.

mod bench;

use super::batches::{Batches, PipelineKey, PixelValues};
use super::{Command, Target, TextureNote};
use crate::ffi::gl::Gles;
use crate::framework::log;
use std::collections::HashMap;
use std::ffi::CStr;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};

/// A canvas pixel the game has not written (`gpu::UNWRITTEN`'s kin).
pub const UNWRITTEN: u32 = 0xA5FF_00FF;

const TEXTURE_2D: u32 = 0x0DE1;
const TEXTURE0: u32 = 0x84C0;
const TEXTURE1: u32 = 0x84C1;
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
layout(location=6) in vec2 uv2;
uniform vec2 size;
out highp vec4 v_colour; out highp float v_fog; out highp float v_w; out highp vec2 v_uv;
out highp vec4 v_colour2; out highp float v_fog2; out highp vec2 v_uv2;
void main() {
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
out vec4 result;
uniform sampler2D picture; uniform vec4 fog_colour; uniform float alpha_reference;
uniform int alpha_function; uniform int mixing; uniform int flags;
uniform sampler2D picture2; uniform vec4 fog_colour2; uniform float alpha_reference2;
uniform int alpha_function2; uniform int mixing2; uniform int flags2;
bool passes(int function, float reference, float value) {
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
    if ((flags & 4) == 0) return c;
    float kept = clamp(fog, 0.0, 1.0);
    return vec4(c.rgb * kept + fog_colour.rgb * (1.0 - kept), c.a);
}
void main() {
    vec4 colour = v_colour / v_w;
    vec4 c = colour;
    // uploaded as they are: blue first
    if ((flags & 1) != 0) c = mixed(colour, texture(picture, v_uv).bgra, mixing, flags);
    if (alpha_function != 0 && !passes(alpha_function, alpha_reference, c.a)) discard;
    c = fogged(c, v_fog / v_w, fog_colour, flags);
    if ((flags & 8) != 0) {
        vec4 under = floor(c * 255.0 + 0.5) / 255.0;
        vec4 colour2 = v_colour2 / v_w;
        vec4 d = colour2;
        if ((flags2 & 1) != 0) d = mixed(colour2, texture(picture2, v_uv2).bgra, mixing2, flags2);
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

struct Locations {
    size: i32,
    first: ValueLocations,
    second: ValueLocations,
}

/// Where a pass's `PixelValues` go.
#[derive(Default)]
struct ValueLocations {
    fog_colour: i32,
    alpha_reference: i32,
    alpha_function: i32,
    mixing: i32,
    flags: i32,
}

struct CachedTexture {
    name: u32,
    width: usize,
    height: usize,
    /// Every pixel's alpha full: a first pass over it can take a second (`Batches::of`).
    opaque: bool,
}

pub struct Gpu {
    gl: Gles,
    scene: u32,
    locations: Locations,
    overlay: Quad,
    canvas_quad: Quad,
    target_quad: Quad,
    vertices: u32,
    /// The vertices' second passes (`Batches::seconds`).
    seconds: u32,
    empty_array: u32,
    scene_array: u32,
    samplers: [u32; 4],
    framebuffer: u32,
    colour: u32,
    depth: u32,
    canvas: u32,
    canvas_size: (i32, i32),
    width: i32,
    height: i32,
    depth_is_new: bool,
    textures: HashMap<crate::pointers::GamePtr, CachedTexture>,
}

pub static GPU: Mutex<Option<Front>> = Mutex::new(None);

/// Whether the GPU is wanted: `RC2K_RENDERER=gpu`.
pub fn wanted() -> bool {
    std::env::var("RC2K_RENDERER").is_ok_and(|r| r == "gpu")
}

impl Gpu {
    fn new() -> Result<Gpu, String> {
        let gl = Gles::open_panel(PANEL.0 as u16, PANEL.1 as u16)?;
        let mut gpu = Gpu {
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
            empty_array: 0,
            scene_array: 0,
            samplers: [0; 4],
            framebuffer: 0,
            colour: 0,
            depth: 0,
            canvas: 0,
            canvas_size: (0, 0),
            width: 0,
            height: 0,
            depth_is_new: true,
            textures: HashMap::new(),
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

    fn set_up(&mut self) -> Result<(), String> {
        self.scene = self.program(SCENE_VS, SCENE_FS)?;
        let values = |suffix: &str| ValueLocations {
            fog_colour: self.uniform(self.scene, &format!("fog_colour{suffix}")),
            alpha_reference: self.uniform(self.scene, &format!("alpha_reference{suffix}")),
            alpha_function: self.uniform(self.scene, &format!("alpha_function{suffix}")),
            mixing: self.uniform(self.scene, &format!("mixing{suffix}")),
            flags: self.uniform(self.scene, &format!("flags{suffix}")),
        };
        self.locations = Locations {
            size: self.uniform(self.scene, "size"),
            first: values(""),
            second: values("2"),
        };
        let second_picture = self.uniform(self.scene, "picture2");
        self.overlay = self.quad(OVERLAY_FS)?;
        self.canvas_quad = self.quad(CANVAS_FS)?;
        self.target_quad = self.quad(TARGET_FS)?;
        let gl = &self.gl;
        // The attribute layout is `batches::Vertex`'s eight dwords.
        self.empty_array = gl.gen_vertex_array();
        self.scene_array = gl.gen_vertex_array();
        self.vertices = gl.gen_buffer();
        self.seconds = gl.gen_buffer();
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
        gl.use_program(self.scene);
        gl.uniform_1i(second_picture, 1);
        self.samplers = gl.gen_samplers();
        for (n, &sampler) in self.samplers.iter().enumerate() {
            let filter = if n & 1 != 0 { 0x2601 } else { 0x2600 };
            let wrap = if n & 2 != 0 { 0x812F } else { 0x2901 };
            gl.sampler_parameter(sampler, 0x2801, filter);
            gl.sampler_parameter(sampler, 0x2800, filter);
            gl.sampler_parameter(sampler, 0x2802, wrap);
            gl.sampler_parameter(sampler, 0x2803, wrap);
        }
        self.framebuffer = gl.gen_framebuffer();
        self.colour = gl.gen_texture();
        self.canvas = gl.gen_texture();
        self.depth = gl.gen_renderbuffer();
        gl.front_face(0x0900); // the shown ones are clockwise on the screen
        Ok(())
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
    fn draw(&mut self, part: Part) {
        let Part {
            commands,
            uploads,
            written,
            size: (width, height),
        } = part;
        if let Err(why) = self.set_size(width, height) {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput(Box::leak(why.into_boxed_str())),
            );
        }
        for (note, bytes) in &uploads {
            self.bring_texture_up(note, bytes);
        }
        let opaque = |note: &TextureNote| self.textures.get(&note.pixels).is_some_and(|t| t.opaque);
        let batches = Batches::of(&commands, width as u32, height as u32, Some(&opaque));
        super::GPU_DRAWS.fetch_add(
            batches.batches.len() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
        for w in &written {
            self.upload_canvas_rows(&w.bytes, width, height, w.first, w.last);
        }
        self.plain_state();
        let gl = &self.gl;
        gl.bind_framebuffer(FRAMEBUFFER, self.framebuffer);
        gl.viewport(0, 0, width, height);
        if self.depth_is_new {
            gl.depth_mask(true);
            gl.clear_depth(1.0);
            gl.clear(DEPTH_BIT);
            self.depth_is_new = false;
        }
        // Up in clip space is the picture's top for the 2D as for the 3D; the object's
        // last row is that top, which `show_target` and `bring_back` turn over.
        if !written.is_empty() {
            gl.enable(SCISSOR_TEST);
            for w in &written {
                gl.bind_texture(TEXTURE_2D, self.canvas);
                let rows = (w.last - w.first + 1) as i32;
                gl.scissor(0, height - w.last as i32 - 1, width, rows);
                self.draw_quad(&self.overlay, false);
            }
            gl.disable(SCISSOR_TEST);
        }
        if !batches.batches.is_empty() {
            gl.use_program(self.scene);
            gl.uniform_2f(self.locations.size, width as f32, height as f32);
            gl.bind_vertex_array(self.scene_array);
            gl.bind_buffer(ARRAY_BUFFER, self.vertices);
            gl.buffer_data(ARRAY_BUFFER, &batches.vertices, 0x88E0);
            gl.bind_buffer(ARRAY_BUFFER, self.seconds);
            gl.buffer_data(ARRAY_BUFFER, &batches.seconds, 0x88E0);
            gl.enable(DEPTH_TEST);
            let mut last_pipeline = None;
            let mut last_values = None;
            let mut last_texture = None;
            let mut last_sampler = u32::MAX;
            let mut last_second = None;
            let mut scissored = false;
            for batch in &batches.batches {
                if last_pipeline != Some(batch.pipeline) {
                    self.set_pipeline(&batch.pipeline);
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
                    gl.active_texture(TEXTURE1);
                    if let Some(n) = second.texture {
                        let name = self.textures[&batches.textures[n as usize].pixels].name;
                        gl.bind_texture(TEXTURE_2D, name);
                    }
                    let sampler = self.samplers[second.linear as usize + 2 * second.clamp as usize];
                    gl.bind_sampler(1, sampler);
                    gl.active_texture(TEXTURE0);
                    last_second = Some(*second);
                }
                if last_texture != Some(batch.texture) {
                    if let Some(n) = batch.texture {
                        let name = self.textures[&batches.textures[n as usize].pixels].name;
                        gl.bind_texture(TEXTURE_2D, name);
                    }
                    last_texture = Some(batch.texture);
                }
                let sampler = self.samplers[batch.linear as usize + 2 * batch.clamp as usize];
                if sampler != last_sampler {
                    gl.bind_sampler(0, sampler);
                    last_sampler = sampler;
                }
                if let Some([left, top, right, bottom]) = batch.scissor {
                    gl.enable(SCISSOR_TEST);
                    gl.scissor(
                        left as i32,
                        height - bottom as i32,
                        (right - left) as i32,
                        (bottom - top) as i32,
                    );
                    scissored = true;
                } else if scissored {
                    gl.disable(SCISSOR_TEST);
                    scissored = false;
                }
                gl.draw_arrays(TRIANGLES, batch.first_vertex as i32, batch.vertices as i32);
            }
            gl.bind_sampler(0, 0);
            gl.bind_sampler(1, 0);
        }
        if bench::due(batches.batches.len()) {
            self.bench(&batches, height);
        }
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

    fn set_pipeline(&self, key: &PipelineKey) {
        let gl = &self.gl;
        match key.cull {
            3 => {
                gl.enable(CULL_FACE);
                gl.cull_face(0x0405);
            }
            2 => {
                gl.enable(CULL_FACE);
                gl.cull_face(0x0404);
            }
            _ => gl.disable(CULL_FACE),
        }
        gl.depth_func(match key.depth_function {
            f @ 1..=7 => 0x0200 + f - 1,
            _ => 0x0207, // as the rasteriser's `compare`: anything else passes
        });
        gl.depth_mask(key.depth_write);
        match key.blend {
            Some((source, target)) => {
                gl.enable(BLEND);
                gl.blend_func(blend_factor(source), blend_factor(target));
            }
            None => gl.disable(BLEND),
        }
        gl.colour_mask(key.colour);
    }

    fn bring_texture_up(&mut self, note: &TextureNote, bytes: &[u8]) {
        super::UPLOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        super::UPLOAD_BYTES.fetch_add(bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);
        let known = self.textures.get(&note.pixels);
        let (width, height) = (note.width as i32, note.height as i32);
        let gl = &self.gl;
        let mut name = known.map_or(0, |t| t.name);
        let new = known.is_none_or(|t| (t.width, t.height) != (note.width, note.height));
        if name == 0 {
            name = gl.gen_texture();
        }
        gl.bind_texture(TEXTURE_2D, name);
        // `bytes` holds the whole picture (`texture_as_bgra`).
        if new {
            gl.tex_image_2d(0x8058, width, height, RGBA, UNSIGNED_BYTE, Some(bytes));
            // No levels: one the sampler objects' filters read alone.
            gl.tex_parameter(TEXTURE_2D, 0x813D, 0);
        } else {
            gl.tex_sub_image_2d((0, 0), (width, height), RGBA, UNSIGNED_BYTE, bytes);
        }
        self.textures.insert(
            note.pixels,
            CachedTexture {
                name,
                width: note.width,
                height: note.height,
                opaque: bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|pixel| pixel[3] == 0xFF),
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
    fn show_target(&mut self) {
        self.plain_state();
        self.panel_viewport(self.width, self.height);
        let gl = &self.gl;
        gl.bind_sampler(0, 0);
        gl.bind_texture(TEXTURE_2D, self.colour);
        self.draw_quad(&self.target_quad, true);
        gl.swap_buffers();
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
    fn forget(&mut self, gone: &[crate::pointers::GamePtr]) {
        for pixels in gone {
            if let Some(texture) = self.textures.remove(pixels) {
                self.gl.delete_texture(texture.name);
            }
        }
    }

    fn run(mut self, jobs: Receiver<Job>) {
        for job in jobs {
            let start = std::time::Instant::now();
            match job {
                Job::Forget(gone) => self.forget(&gone),
                Job::Draw(part) => self.draw(part),
                Job::ShowTarget => self.show_target(),
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

/// The canvas's written rows, copied.
struct Written {
    first: usize,
    last: usize,
    bytes: Vec<u8>,
}

/// What a draw needs, all of it the thread's own: the game goes on with its memory meanwhile.
struct Part {
    commands: Vec<Command>,
    uploads: Vec<(TextureNote, Vec<u8>)>,
    written: Vec<Written>,
    size: (i32, i32),
}

enum Job {
    /// The textures whose memory is gone, by address.
    Forget(Vec<crate::pointers::GamePtr>),
    Draw(Part),
    ShowTarget,
    ShowPicture {
        frame: Vec<u8>,
        width: i32,
        height: i32,
    },
    ReadTarget(Sender<Vec<u8>>),
}

/// The game's side of the GL's thread, and what it knows the GPU holds.
pub struct Front {
    jobs: SyncSender<Job>,
    /// The textures up to date on the GPU, with their sizes.
    known: HashMap<crate::pointers::GamePtr, (usize, usize)>,
    size: (i32, i32),
}

/// Jobs waiting at most: a frame's part, its end, a little more; then the game waits.
const JOBS_WAITING: usize = 2;

impl Front {
    fn send(&self, job: Job) {
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

    /// Hands over the commands and the canvas's written rows, which it marks again.
    fn draw(&mut self, canvas: &mut [u8], commands: Vec<Command>) {
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
        // One texture is usually drawn many times running.
        let mut last = None;
        for command in &commands {
            if let Command::Fan {
                texture: Some(note),
                ..
            } = command
                && last != Some(note.pixels)
            {
                last = Some(note.pixels);
                if self.known.get(&note.pixels) != Some(&(note.width, note.height)) {
                    uploads.push((*note, super::texture_as_bgra(note)));
                    self.known.insert(note.pixels, (note.width, note.height));
                }
            }
        }
        let stride = width as usize * 4;
        let written = written_runs(canvas, width as usize)
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
            size: self.size,
        }));
    }

    /// The frame's end: what the canvas has written over the target, and the target shown.
    pub fn finish_and_show(&mut self, canvas: &mut [u8]) {
        self.draw(canvas, Vec::new());
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
    pub fn bring_back(&mut self, canvas: &mut [u8]) {
        super::READ_BACKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.draw(canvas, Vec::new());
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

/// Unwritten rows that may lie inside a run: a run costs an upload and a quad.
const RUN_GAP: usize = 16;

/// The runs of the canvas's rows holding a written pixel, each its first and last row: the
/// panels at the top and the bottom leave the rows between them to the 3D.
fn written_runs(canvas: &[u8], width: usize) -> smallvec::SmallVec<[(usize, usize); 4]> {
    const BLOCK: usize = 256;
    let unwritten = UNWRITTEN.to_le_bytes();
    let blank: [u8; BLOCK] = std::array::from_fn(|n| unwritten[n % 4]);
    // By blocks, as the library compares memory: this looks at the screen's pixels every frame.
    let row_written = |row: &[u8]| {
        let (blocks, rest) = row.as_chunks::<BLOCK>();
        blocks.iter().any(|block| *block != blank) || *rest != blank[..rest.len()]
    };
    // Each row looked at once, the screen in two bands side by side on rayon's threads (no
    // thread made per flush): the scan is the most of what the game's thread spends here.
    const BANDS: usize = 2;
    let stride = width * 4;
    let mut written = vec![false; canvas.len() / stride.max(1)];
    let band = written.len().div_ceil(BANDS).max(1);
    let look = |rows: &[u8], into: &mut [bool]| {
        for (row, seen) in rows.chunks_exact(stride).zip(into) {
            *seen = row_written(row);
        }
    };
    let (first_rows, last_rows) = canvas.split_at((band * stride).min(canvas.len()));
    let rows = written.len();
    let (first_into, last_into) = written.split_at_mut(band.min(rows));
    rayon::join(
        || look(first_rows, first_into),
        || look(last_rows, last_into),
    );
    let mut runs = smallvec::SmallVec::<[(usize, usize); 4]>::new();
    for y in (0..written.len()).filter(|&y| written[y]) {
        match runs.last_mut() {
            Some((_, last)) if y - *last <= RUN_GAP => *last = y,
            _ => runs.push((y, y)),
        }
    }
    runs
}

/// Every pixel of the canvas marked as not written.
fn mark_unwritten(canvas: &mut [u8]) {
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
    if held.is_some() || !wanted() || TRIED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let (jobs, waiting) = sync_channel(JOBS_WAITING);
    let (answer, started) = channel();
    let thread = std::thread::Builder::new()
        .name("GL".into())
        .spawn(move || match Gpu::new() {
            Ok(gpu) => {
                let _ = answer.send(Ok(()));
                gpu.run(waiting);
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
    match outcome {
        Ok(()) => {
            log("render: the GPU backend on libmali's OpenGL ES, on its own thread");
            *held = Some(Front {
                jobs,
                known: HashMap::new(),
                size: (0, 0),
            });
            super::gpu_started();
        }
        Err(why) => log(&format!("render: the GPU backend does not start: {why}")),
    }
}

/// It is asked for once: failing, the rasteriser goes on.
static TRIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl super::Backend for Front {
    fn draw(&mut self, target: &Target, commands: Vec<Command>) {
        self.size = (target.width as i32, target.height as i32);
        target.with_canvas(|canvas| Front::draw(self, canvas, commands));
    }
}
