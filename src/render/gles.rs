//! The GPU backend on the handheld: OpenGL ES 3 through the system's `libmali` (no Vulkan driver
//! there, and SDL has no display for it), drawing to `/dev/fb0` through its EGL. The same frame
//! as `gpu`'s: `batches`' lists, the canvas's written pixels over the 3D. The GL's own thread
//! holds the context and does the driver's work (a quarter of a frame's time on this processor);
//! the game's thread hands it copies, by channel, and goes on. All of this file's `unsafe` is
//! the calls through the library's table, on that thread alone.

use super::batches::{Batches, PipelineKey, PixelValues};
use super::{Command, Target, TextureNote};
use crate::framework::log;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, SyncSender, channel, sync_channel};

/// A canvas pixel the game has not written (`gpu::UNWRITTEN`'s kin).
pub const UNWRITTEN: u32 = 0xA5FF_00FF;

type P = *mut c_void;
macro_rules! library {
    ($($name:ident: fn($($arg:ty),*) $(-> $ret:ty)?;)*) => {
        #[allow(non_snake_case)] // the library's names
        struct Library { $($name: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)* }
        impl Library {
            fn open() -> Result<Library, String> {
                // SAFETY: a constant name; each symbol is cast to the type the Khronos
                // headers give it, and the library stays open for the program's life.
                unsafe {
                    let handle = libc::dlopen(c"/usr/lib/libmali.so".as_ptr(), libc::RTLD_NOW);
                    if handle.is_null() {
                        return Err("no /usr/lib/libmali.so".into());
                    }
                    Ok(Library { $($name: {
                        let symbol = libc::dlsym(
                            handle,
                            concat!(stringify!($name), "\0").as_ptr().cast(),
                        );
                        if symbol.is_null() {
                            return Err(concat!("libmali has no ", stringify!($name)).into());
                        }
                        std::mem::transmute::<*mut c_void, unsafe extern "C" fn($($arg),*) $(-> $ret)?>(symbol)
                    },)* })
                }
            }
        }
    };
}
library! {
    eglGetDisplay: fn(P) -> P;
    eglInitialize: fn(P, *mut i32, *mut i32) -> u32;
    eglChooseConfig: fn(P, *const i32, *mut P, i32, *mut i32) -> u32;
    eglCreateWindowSurface: fn(P, P, P, *const i32) -> P;
    eglCreateContext: fn(P, P, P, *const i32) -> P;
    eglMakeCurrent: fn(P, P, P, P) -> u32;
    eglSwapBuffers: fn(P, P) -> u32;
    eglSwapInterval: fn(P, i32) -> u32;
    eglBindAPI: fn(u32) -> u32;
    eglGetError: fn() -> i32;
    glCreateShader: fn(u32) -> u32;
    glShaderSource: fn(u32, i32, *const *const c_char, *const i32);
    glCompileShader: fn(u32);
    glGetShaderiv: fn(u32, u32, *mut i32);
    glGetShaderInfoLog: fn(u32, i32, *mut i32, *mut c_char);
    glCreateProgram: fn() -> u32;
    glAttachShader: fn(u32, u32);
    glLinkProgram: fn(u32);
    glUseProgram: fn(u32);
    glGetUniformLocation: fn(u32, *const c_char) -> i32;
    glUniform1i: fn(i32, i32);
    glUniform1f: fn(i32, f32);
    glUniform2f: fn(i32, f32, f32);
    glUniform4fv: fn(i32, i32, *const f32);
    glGenTextures: fn(i32, *mut u32);
    glBindTexture: fn(u32, u32);
    glTexImage2D: fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
    glTexSubImage2D: fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
    glTexParameteri: fn(u32, u32, i32);
    glGenSamplers: fn(i32, *mut u32);
    glBindSampler: fn(u32, u32);
    glSamplerParameteri: fn(u32, u32, i32);
    glGenBuffers: fn(i32, *mut u32);
    glBindBuffer: fn(u32, u32);
    glBufferData: fn(u32, isize, *const c_void, u32);
    glGenVertexArrays: fn(i32, *mut u32);
    glBindVertexArray: fn(u32);
    glEnableVertexAttribArray: fn(u32);
    glVertexAttribPointer: fn(u32, i32, u32, u8, i32, *const c_void);
    glGenFramebuffers: fn(i32, *mut u32);
    glBindFramebuffer: fn(u32, u32);
    glFramebufferTexture2D: fn(u32, u32, u32, u32, i32);
    glGenRenderbuffers: fn(i32, *mut u32);
    glBindRenderbuffer: fn(u32, u32);
    glRenderbufferStorage: fn(u32, u32, i32, i32);
    glFramebufferRenderbuffer: fn(u32, u32, u32, u32);
    glCheckFramebufferStatus: fn(u32) -> u32;
    glViewport: fn(i32, i32, i32, i32);
    glScissor: fn(i32, i32, i32, i32);
    glClearColor: fn(f32, f32, f32, f32);
    glClearDepthf: fn(f32);
    glClear: fn(u32);
    glEnable: fn(u32);
    glDisable: fn(u32);
    glBlendFunc: fn(u32, u32);
    glDepthFunc: fn(u32);
    glDepthMask: fn(u8);
    glColorMask: fn(u8, u8, u8, u8);
    glCullFace: fn(u32);
    glFrontFace: fn(u32);
    glDrawArrays: fn(u32, i32, i32);
    glReadPixels: fn(i32, i32, i32, i32, u32, u32, *mut c_void);
}

const TEXTURE_2D: u32 = 0x0DE1;
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
uniform vec2 size;
out highp vec4 v_colour; out highp float v_fog; out highp float v_w; out highp vec2 v_uv;
void main() {
    float w = place.w > 0.0 ? 1.0 / place.w : 1.0;
    gl_Position = vec4(((place.x + 0.5) / size.x * 2.0 - 1.0) * w,
                       (1.0 - (place.y + 0.5) / size.y * 2.0) * w,
                       (clamp(place.z, 0.0, 1.0) * 2.0 - 1.0) * w, w);
    v_colour = colour.bgra * w; v_fog = specular.a * w; v_w = w; v_uv = uv;
}
";
const SCENE_FS: &CStr = c"#version 300 es
precision mediump float;
in highp vec4 v_colour; in highp float v_fog; in highp float v_w; in highp vec2 v_uv;
out vec4 result;
uniform sampler2D picture; uniform vec4 fog_colour; uniform float alpha_reference;
uniform int alpha_function; uniform int mixing; uniform int flags;
bool passes(float value) {
    if (alpha_function == 1) return false;
    if (alpha_function == 2) return value < alpha_reference;
    if (alpha_function == 3) return value == alpha_reference;
    if (alpha_function == 4) return value <= alpha_reference;
    if (alpha_function == 5) return value > alpha_reference;
    if (alpha_function == 6) return value != alpha_reference;
    if (alpha_function == 7) return value >= alpha_reference;
    return true;
}
void main() {
    vec4 colour = v_colour / v_w;
    vec4 c = colour;
    if ((flags & 1) != 0) {
        vec4 texel = texture(picture, v_uv).bgra; // uploaded as they are: blue first
        if (mixing == 1 || mixing == 7) c = texel;
        else if (mixing == 4) c = texel * colour;
        else c = vec4(texel.rgb * colour.rgb, (flags & 2) != 0 ? texel.a : colour.a);
    }
    if (alpha_function != 0 && !passes(c.a)) discard;
    if ((flags & 4) != 0) {
        float kept = clamp(v_fog / v_w, 0.0, 1.0);
        c.rgb = c.rgb * kept + fog_colour.rgb * (1.0 - kept);
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
}

pub struct Gpu {
    gl: Library,
    display: P,
    surface: P,
    scene: u32,
    locations: Locations,
    overlay: Quad,
    canvas_quad: Quad,
    target_quad: Quad,
    vertices: u32,
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
    textures: HashMap<usize, CachedTexture>,
}

pub static GPU: Mutex<Option<Front>> = Mutex::new(None);

/// Whether the GPU is wanted: `RC2K_RENDERER=gpu`.
pub fn wanted() -> bool {
    std::env::var("RC2K_RENDERER").is_ok_and(|r| r == "gpu")
}

impl Gpu {
    fn new() -> Result<Gpu, String> {
        let gl = Library::open()?;
        // SAFETY: EGL's and GL's calls with the values their documents ask for; the attribute
        // lists are ended by EGL_NONE and live through their calls; the window record's
        // lifetime is the program's (leaked: libmali holds on to it).
        unsafe {
            let display = (gl.eglGetDisplay)(std::ptr::null_mut());
            let (mut major, mut minor) = (0, 0);
            if (gl.eglInitialize)(display, &raw mut major, &raw mut minor) == 0 {
                return Err(format!("eglInitialize: {:#x}", (gl.eglGetError)()));
            }
            let wanted = [
                0x3024, 8, 0x3023, 8, 0x3022, 8, 0x3033, 4, 0x3040, 0x40, 0x3038i32,
            ];
            let (mut config, mut found) = (std::ptr::null_mut(), 0);
            (gl.eglChooseConfig)(display, wanted.as_ptr(), &raw mut config, 1, &raw mut found);
            if found == 0 {
                return Err("EGL has no such config".into());
            }
            (gl.eglBindAPI)(0x30A0);
            #[repr(C)]
            struct FbdevWindow {
                width: u16,
                height: u16,
            }
            let window = Box::leak(Box::new(FbdevWindow {
                width: PANEL.0 as u16,
                height: PANEL.1 as u16,
            }));
            let surface = (gl.eglCreateWindowSurface)(
                display,
                config,
                std::ptr::from_mut(window).cast(),
                std::ptr::null(),
            );
            let attributes = [0x3098, 3, 0x3038i32];
            let context =
                (gl.eglCreateContext)(display, config, std::ptr::null_mut(), attributes.as_ptr());
            if surface.is_null()
                || context.is_null()
                || (gl.eglMakeCurrent)(display, surface, surface, context) == 0
            {
                return Err(format!("the EGL context: {:#x}", (gl.eglGetError)()));
            }
            (gl.eglSwapInterval)(display, 0);
            let mut gpu = Gpu {
                display,
                surface,
                scene: 0,
                locations: Locations {
                    size: 0,
                    fog_colour: 0,
                    alpha_reference: 0,
                    alpha_function: 0,
                    mixing: 0,
                    flags: 0,
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
    }

    fn compile(&self, kind: u32, text: &CStr) -> Result<u32, String> {
        // SAFETY: the text lives through the call, ended by its NUL; the log's room is given.
        unsafe {
            let shader = (self.gl.glCreateShader)(kind);
            let pointer = text.as_ptr();
            (self.gl.glShaderSource)(shader, 1, &raw const pointer, std::ptr::null());
            (self.gl.glCompileShader)(shader);
            let mut done = 0;
            (self.gl.glGetShaderiv)(shader, 0x8B81, &raw mut done);
            if done == 0 {
                let mut log = [0 as c_char; 600];
                (self.gl.glGetShaderInfoLog)(shader, 600, std::ptr::null_mut(), log.as_mut_ptr());
                return Err(CStr::from_ptr(log.as_ptr()).to_string_lossy().into_owned());
            }
            Ok(shader)
        }
    }

    fn program(&self, vertex: &CStr, fragment: &CStr) -> Result<u32, String> {
        let (v, f) = (
            self.compile(0x8B31, vertex)?,
            self.compile(0x8B30, fragment)?,
        );
        // SAFETY: shaders just made.
        unsafe {
            let program = (self.gl.glCreateProgram)();
            (self.gl.glAttachShader)(program, v);
            (self.gl.glAttachShader)(program, f);
            (self.gl.glLinkProgram)(program);
            Ok(program)
        }
    }

    fn uniform(&self, program: u32, name: &str) -> i32 {
        let name = CString::new(name).expect("no NUL in a uniform's name");
        // SAFETY: the name lives through the call.
        unsafe { (self.gl.glGetUniformLocation)(program, name.as_ptr()) }
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
        self.locations = Locations {
            size: self.uniform(self.scene, "size"),
            fog_colour: self.uniform(self.scene, "fog_colour"),
            alpha_reference: self.uniform(self.scene, "alpha_reference"),
            alpha_function: self.uniform(self.scene, "alpha_function"),
            mixing: self.uniform(self.scene, "mixing"),
            flags: self.uniform(self.scene, "flags"),
        };
        self.overlay = self.quad(OVERLAY_FS)?;
        self.canvas_quad = self.quad(CANVAS_FS)?;
        self.target_quad = self.quad(TARGET_FS)?;
        let gl = &self.gl;
        // SAFETY: names made here, each out-pointer valid for the call; the attribute layout is
        // `batches::Vertex`'s eight dwords.
        unsafe {
            (gl.glGenVertexArrays)(1, &raw mut self.empty_array);
            (gl.glGenVertexArrays)(1, &raw mut self.scene_array);
            (gl.glGenBuffers)(1, &raw mut self.vertices);
            (gl.glBindVertexArray)(self.scene_array);
            (gl.glBindBuffer)(ARRAY_BUFFER, self.vertices);
            for (n, count, kind, normal, offset) in [
                (0u32, 4, 0x1406u32, 0u8, 0usize),
                (1, 4, UNSIGNED_BYTE, 1, 16),
                (2, 4, UNSIGNED_BYTE, 1, 20),
                (3, 2, 0x1406, 0, 24),
            ] {
                (gl.glEnableVertexAttribArray)(n);
                (gl.glVertexAttribPointer)(n, count, kind, normal, 32, offset as *const c_void);
            }
            (gl.glGenSamplers)(4, self.samplers.as_mut_ptr());
            for (n, &sampler) in self.samplers.iter().enumerate() {
                let filter = if n & 1 != 0 { 0x2601 } else { 0x2600 };
                let wrap = if n & 2 != 0 { 0x812F } else { 0x2901 };
                (gl.glSamplerParameteri)(sampler, 0x2801, filter);
                (gl.glSamplerParameteri)(sampler, 0x2800, filter);
                (gl.glSamplerParameteri)(sampler, 0x2802, wrap);
                (gl.glSamplerParameteri)(sampler, 0x2803, wrap);
            }
            (gl.glGenFramebuffers)(1, &raw mut self.framebuffer);
            (gl.glGenTextures)(1, &raw mut self.colour);
            (gl.glGenTextures)(1, &raw mut self.canvas);
            (gl.glGenRenderbuffers)(1, &raw mut self.depth);
            (gl.glFrontFace)(0x0900); // the shown ones are clockwise on the screen
        }
        Ok(())
    }

    /// The frame buffer object at the screen's size.
    fn set_size(&mut self, width: i32, height: i32) -> Result<(), String> {
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        let gl = &self.gl;
        // SAFETY: names made in `set_up`; no pixels given.
        unsafe {
            (gl.glBindTexture)(TEXTURE_2D, self.colour);
            (gl.glTexImage2D)(
                TEXTURE_2D,
                0,
                0x8058,
                width,
                height,
                0,
                RGBA,
                UNSIGNED_BYTE,
                std::ptr::null(),
            );
            (gl.glTexParameteri)(TEXTURE_2D, 0x2801, 0x2600);
            (gl.glTexParameteri)(TEXTURE_2D, 0x2800, 0x2600);
            (gl.glBindRenderbuffer)(0x8D41, self.depth);
            (gl.glRenderbufferStorage)(0x8D41, 0x81A6, width, height);
            (gl.glBindFramebuffer)(FRAMEBUFFER, self.framebuffer);
            (gl.glFramebufferTexture2D)(FRAMEBUFFER, 0x8CE0, TEXTURE_2D, self.colour, 0);
            (gl.glFramebufferRenderbuffer)(FRAMEBUFFER, 0x8D00, 0x8D41, self.depth);
            if (gl.glCheckFramebufferStatus)(FRAMEBUFFER) != 0x8CD5 {
                return Err("the frame buffer object is not complete".into());
            }
        }
        (self.width, self.height) = (width, height);
        self.depth_is_new = true;
        Ok(())
    }

    /// The canvas's bytes into its texture.
    fn upload_canvas(&mut self, canvas: &[u8], width: i32, height: i32) {
        let gl = &self.gl;
        // SAFETY: `canvas` holds `width` x `height` pixels of four bytes (`Target::bytes`,
        // `present`'s check).
        unsafe {
            (gl.glBindTexture)(TEXTURE_2D, self.canvas);
            (gl.glBindSampler)(0, 0);
            if self.canvas_size != (width, height) {
                (gl.glTexImage2D)(
                    TEXTURE_2D,
                    0,
                    0x8058,
                    width,
                    height,
                    0,
                    RGBA,
                    UNSIGNED_BYTE,
                    canvas.as_ptr().cast(),
                );
                (gl.glTexParameteri)(TEXTURE_2D, 0x2801, 0x2600);
                (gl.glTexParameteri)(TEXTURE_2D, 0x2800, 0x2600);
                self.canvas_size = (width, height);
            } else {
                (gl.glTexSubImage2D)(
                    TEXTURE_2D,
                    0,
                    0,
                    0,
                    width,
                    height,
                    RGBA,
                    UNSIGNED_BYTE,
                    canvas.as_ptr().cast(),
                );
            }
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
        // SAFETY: `rows` holds the rows the call reads; no pixels given to the first.
        unsafe {
            (gl.glBindTexture)(TEXTURE_2D, self.canvas);
            (gl.glBindSampler)(0, 0);
            if self.canvas_size != (width, height) {
                (gl.glTexImage2D)(
                    TEXTURE_2D,
                    0,
                    0x8058,
                    width,
                    height,
                    0,
                    RGBA,
                    UNSIGNED_BYTE,
                    std::ptr::null(),
                );
                (gl.glTexParameteri)(TEXTURE_2D, 0x2801, 0x2600);
                (gl.glTexParameteri)(TEXTURE_2D, 0x2800, 0x2600);
                self.canvas_size = (width, height);
            }
            (gl.glTexSubImage2D)(
                TEXTURE_2D,
                0,
                0,
                first as i32,
                width,
                (last - first + 1) as i32,
                RGBA,
                UNSIGNED_BYTE,
                rows.as_ptr().cast(),
            );
        }
    }

    /// Everything a quad needs off.
    fn plain_state(&self) {
        let gl = &self.gl;
        // SAFETY: state only.
        unsafe {
            (gl.glDisable)(DEPTH_TEST);
            (gl.glDisable)(BLEND);
            (gl.glDisable)(CULL_FACE);
            (gl.glDisable)(SCISSOR_TEST);
            (gl.glColorMask)(1, 1, 1, 1);
            (gl.glBindVertexArray)(self.empty_array);
        }
    }

    fn draw_quad(&self, quad: &Quad, flip: bool) {
        let gl = &self.gl;
        // SAFETY: a program made in `set_up`; three vertices of no attributes.
        unsafe {
            (gl.glUseProgram)(quad.program);
            (gl.glUniform1f)(quad.flip, if flip { 1.0 } else { 0.0 });
            (gl.glDrawArrays)(TRIANGLES, 0, 3);
        }
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
        let batches = Batches::of(&commands, width as u32, height as u32);
        if let Err(why) = self.set_size(width, height) {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput(Box::leak(why.into_boxed_str())),
            );
        }
        for (note, bytes) in &uploads {
            self.bring_texture_up(note, bytes);
        }
        let rows = written.as_ref().map(|w| (w.first, w.last));
        if let Some(w) = &written {
            self.upload_canvas_rows(&w.bytes, width, height, w.first, w.last);
        }
        self.plain_state();
        let gl = &self.gl;
        // SAFETY: names ours; the vertices live through `glBufferData`, which copies them; the
        // uniforms' values are read in the calls; the texture of each batch was made above.
        unsafe {
            (gl.glBindFramebuffer)(FRAMEBUFFER, self.framebuffer);
            (gl.glViewport)(0, 0, width, height);
            if self.depth_is_new {
                (gl.glDepthMask)(1);
                (gl.glClearDepthf)(1.0);
                (gl.glClear)(DEPTH_BIT);
                self.depth_is_new = false;
            }
            // Up in clip space is the picture's top for the 2D as for the 3D; the object's
            // last row is that top, which `show_target` and `bring_back` turn over.
            if let Some((first, last)) = rows {
                (gl.glBindTexture)(TEXTURE_2D, self.canvas);
                (gl.glEnable)(SCISSOR_TEST);
                let rows = (last - first + 1) as i32;
                (gl.glScissor)(0, height - last as i32 - 1, width, rows);
                self.draw_quad(&self.overlay, false);
                (gl.glDisable)(SCISSOR_TEST);
            }
            if !batches.batches.is_empty() {
                (gl.glUseProgram)(self.scene);
                (gl.glUniform2f)(self.locations.size, width as f32, height as f32);
                (gl.glBindVertexArray)(self.scene_array);
                (gl.glBindBuffer)(ARRAY_BUFFER, self.vertices);
                (gl.glBufferData)(
                    ARRAY_BUFFER,
                    size_of_val(&batches.vertices[..]) as isize,
                    batches.vertices.as_ptr().cast(),
                    0x88E0,
                );
                (gl.glEnable)(DEPTH_TEST);
                let mut last_pipeline = None;
                let mut last_values = None;
                let mut last_texture = None;
                let mut last_sampler = u32::MAX;
                let mut scissored = false;
                for batch in &batches.batches {
                    if last_pipeline != Some(batch.pipeline) {
                        self.set_pipeline(&batch.pipeline);
                        last_pipeline = Some(batch.pipeline);
                    }
                    // Each value only when it changes: the batches differ in one or two.
                    let v = &batch.values;
                    let was = last_values.as_ref();
                    let l = &self.locations;
                    if was.is_none_or(|w: &PixelValues| w.fog_colour != v.fog_colour) {
                        (gl.glUniform4fv)(l.fog_colour, 1, v.fog_colour.as_ptr());
                    }
                    if was.is_none_or(|w| w.alpha_reference != v.alpha_reference) {
                        (gl.glUniform1f)(l.alpha_reference, v.alpha_reference);
                    }
                    if was.is_none_or(|w| w.alpha_function != v.alpha_function) {
                        (gl.glUniform1i)(l.alpha_function, v.alpha_function);
                    }
                    if was.is_none_or(|w| w.mixing != v.mixing) {
                        (gl.glUniform1i)(l.mixing, v.mixing);
                    }
                    if was.is_none_or(|w| w.flags != v.flags) {
                        (gl.glUniform1i)(l.flags, v.flags);
                    }
                    last_values = Some(*v);
                    if last_texture != Some(batch.texture) {
                        if let Some(n) = batch.texture {
                            let name = self.textures[&batches.textures[n as usize].pixels].name;
                            (gl.glBindTexture)(TEXTURE_2D, name);
                        }
                        last_texture = Some(batch.texture);
                    }
                    let sampler = self.samplers[batch.linear as usize + 2 * batch.clamp as usize];
                    if sampler != last_sampler {
                        (gl.glBindSampler)(0, sampler);
                        last_sampler = sampler;
                    }
                    if let Some([left, top, right, bottom]) = batch.scissor {
                        (gl.glEnable)(SCISSOR_TEST);
                        (gl.glScissor)(
                            left as i32,
                            height - bottom as i32,
                            (right - left) as i32,
                            (bottom - top) as i32,
                        );
                        scissored = true;
                    } else if scissored {
                        (gl.glDisable)(SCISSOR_TEST);
                        scissored = false;
                    }
                    (gl.glDrawArrays)(TRIANGLES, batch.first_vertex as i32, batch.vertices as i32);
                }
                (gl.glBindSampler)(0, 0);
            }
        }
    }

    /// # Safety
    /// The context is current.
    unsafe fn set_pipeline(&self, key: &PipelineKey) {
        let gl = &self.gl;
        // SAFETY: state only.
        unsafe {
            match key.cull {
                3 => {
                    (gl.glEnable)(CULL_FACE);
                    (gl.glCullFace)(0x0405);
                }
                2 => {
                    (gl.glEnable)(CULL_FACE);
                    (gl.glCullFace)(0x0404);
                }
                _ => (gl.glDisable)(CULL_FACE),
            }
            (gl.glDepthFunc)(match key.depth_function {
                f @ 1..=7 => 0x0200 + f - 1,
                _ => 0x0207, // as the rasteriser's `compare`: anything else passes
            });
            (gl.glDepthMask)(key.depth_write as u8);
            match key.blend {
                Some((source, target)) => {
                    (gl.glEnable)(BLEND);
                    (gl.glBlendFunc)(blend_factor(source), blend_factor(target));
                }
                None => (gl.glDisable)(BLEND),
            }
            let c = key.colour as u8;
            (gl.glColorMask)(c, c, c, c);
        }
    }

    fn bring_texture_up(&mut self, note: &TextureNote, bytes: &[u8]) {
        let known = self.textures.get(&note.pixels);
        let (width, height) = (note.width as i32, note.height as i32);
        let gl = &self.gl;
        // SAFETY: `bytes` holds the whole picture (`texture_as_bgra`), for the calls' length.
        unsafe {
            let mut name = known.map_or(0, |t| t.name);
            let new = known.is_none_or(|t| (t.width, t.height) != (note.width, note.height));
            if name == 0 {
                (gl.glGenTextures)(1, &raw mut name);
            }
            (gl.glBindTexture)(TEXTURE_2D, name);
            if new {
                (gl.glTexImage2D)(
                    TEXTURE_2D,
                    0,
                    0x8058,
                    width,
                    height,
                    0,
                    RGBA,
                    UNSIGNED_BYTE,
                    bytes.as_ptr().cast(),
                );
                // No levels: one the sampler objects' filters read alone.
                (gl.glTexParameteri)(TEXTURE_2D, 0x813D, 0);
            } else {
                (gl.glTexSubImage2D)(
                    TEXTURE_2D,
                    0,
                    0,
                    0,
                    width,
                    height,
                    RGBA,
                    UNSIGNED_BYTE,
                    bytes.as_ptr().cast(),
                );
            }
            self.textures.insert(
                note.pixels,
                CachedTexture {
                    name,
                    width: note.width,
                    height: note.height,
                },
            );
        }
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
        // SAFETY: state, and a clear of the default frame buffer.
        unsafe {
            (gl.glBindFramebuffer)(FRAMEBUFFER, 0);
            (gl.glClearColor)(0.0, 0.0, 0.0, 1.0);
            (gl.glClear)(COLOUR_BIT);
            (gl.glViewport)((PANEL.0 - w) / 2, (PANEL.1 - h) / 2, w, h);
        }
    }

    /// The target on the panel.
    fn show_target(&mut self) {
        self.plain_state();
        self.panel_viewport(self.width, self.height);
        let gl = &self.gl;
        // SAFETY: names ours; the context is current.
        unsafe {
            (gl.glBindSampler)(0, 0);
            (gl.glBindTexture)(TEXTURE_2D, self.colour);
            self.draw_quad(&self.target_quad, true);
            (gl.eglSwapBuffers)(self.display, self.surface);
        }
    }

    /// A picture of the game's bytes (`width` x `height`, blue first) on the panel: the frames the
    /// rasteriser made.
    fn show_picture(&mut self, frame: &[u8], width: i32, height: i32) {
        self.upload_canvas(frame, width, height);
        self.plain_state();
        self.panel_viewport(width, height);
        // SAFETY: names ours; the context is current.
        unsafe {
            self.draw_quad(&self.canvas_quad, false);
            (self.gl.eglSwapBuffers)(self.display, self.surface);
        }
    }

    /// The target's rows as the GL holds them.
    fn read_target(&mut self) -> Vec<u8> {
        let (width, height) = (self.width, self.height);
        let mut out = vec![0u8; width as usize * height as usize * 4];
        // SAFETY: `out` holds the target's pixels.
        unsafe {
            (self.gl.glBindFramebuffer)(FRAMEBUFFER, self.framebuffer);
            (self.gl.glReadPixels)(
                0,
                0,
                width,
                height,
                RGBA,
                UNSIGNED_BYTE,
                out.as_mut_ptr().cast(),
            );
        }
        out
    }

    fn run(mut self, jobs: Receiver<Job>) {
        for job in jobs {
            match job {
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
    written: Option<Written>,
    size: (i32, i32),
}

enum Job {
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
    known: HashMap<usize, (usize, usize)>,
    size: (i32, i32),
}

/// Jobs waiting at most: a frame's part, its end, a little more; then the game waits.
const JOBS_WAITING: usize = 2;

impl Front {
    fn send(&self, job: Job) {
        if self.jobs.send(job).is_err() {
            crate::fatal::raise::end(
                "render::gles",
                crate::errors::BadInput("the GL's thread has ended"),
            );
        }
    }

    /// Hands over the commands and the canvas's written rows, which it marks again.
    fn draw(&mut self, canvas: &mut [u8], commands: Vec<Command>) {
        for pixels in super::take_changed_textures() {
            self.known.remove(&pixels);
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
        let written = written_rows(canvas, width as usize).map(|(first, last)| {
            let rows = &mut canvas[first * stride..(last + 1) * stride];
            let bytes = rows.to_vec();
            mark_unwritten(rows);
            Written { first, last, bytes }
        });
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

/// The first and last rows of the canvas holding a written pixel.
fn written_rows(canvas: &[u8], width: usize) -> Option<(usize, usize)> {
    const BLOCK: usize = 256;
    let unwritten = UNWRITTEN.to_le_bytes();
    let blank: [u8; BLOCK] = std::array::from_fn(|n| unwritten[n % 4]);
    // By blocks, as the library compares memory: this looks at most of the screen's pixels
    // every frame.
    let row_written = |row: &[u8]| {
        let (blocks, rest) = row.as_chunks::<BLOCK>();
        blocks.iter().any(|block| *block != blank) || *rest != blank[..rest.len()]
    };
    let stride = width * 4;
    let first = canvas.chunks_exact(stride).position(row_written)?;
    let last = canvas.chunks_exact(stride).rposition(row_written)?;
    Some((first, last))
}

/// Every pixel of the canvas marked as not written.
fn mark_unwritten(canvas: &mut [u8]) {
    for pixel in canvas.as_chunks_mut::<4>().0 {
        *pixel = UNWRITTEN.to_le_bytes();
    }
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
        }
        Err(why) => log(&format!("render: the GPU backend does not start: {why}")),
    }
}

/// It is asked for once: failing, the rasteriser goes on.
static TRIED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl super::Backend for Front {
    fn draw(&mut self, target: &Target, commands: Vec<Command>) {
        self.size = (target.width as i32, target.height as i32);
        // SAFETY: the game's code waits for this call; nothing else writes the canvas meanwhile.
        Front::draw(self, unsafe { target.bytes() }, commands);
    }
}
