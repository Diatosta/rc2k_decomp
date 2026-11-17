#![allow(unsafe_code)]
//! OpenGL ES 3 and EGL through the handheld's `libmali` (opened at run time, the entry points
//! looked up by name), the only place that calls them. `Gles` is the context made current on
//! the thread that made it (`Gles::open_panel`): it is not `Send`, and every method must be
//! called on that thread, which is what makes the calls valid. Names (textures, buffers,
//! programs) are plain numbers the caller keeps; the context checks them and sets an error for a
//! wrong one. Each method is the one GL call, or a small group that always goes together, with
//! the arguments as they are; the ones that take pixels or vertices take slices and check that
//! the call reads no more than the slice holds. The methods are `#[inline]`: the draw loop makes
//! thousands of them a frame.
//!
//! One thing the wrapper cannot check: `draw_arrays` reads the vertex buffers bound at the time
//! (the GL has no bounds check there without robust access); the caller draws only what it sent.

use std::ffi::{CStr, CString, c_char, c_void};

type P = *mut c_void;

macro_rules! library {
    ($table:ident { $($name:ident: fn($($arg:ty),*) $(-> $ret:ty)?;)* }) => {
        #[allow(non_snake_case)] // the library's names
        struct $table { $($name: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)* }
        impl $table {
            /// Every entry point, each found by `lookup` (a name ended by a NUL, a null pointer
            /// for one that does not exist).
            fn load(lookup: &dyn Fn(&CStr) -> P) -> Result<$table, String> {
                // SAFETY: each symbol is cast to the type the Khronos headers give it; the
                // library it comes from stays open for the program's life.
                unsafe {
                    Ok($table { $($name: {
                        let symbol = lookup(
                            CStr::from_bytes_with_nul_unchecked(
                                concat!(stringify!($name), "\0").as_bytes(),
                            ),
                        );
                        if symbol.is_null() {
                            return Err(concat!("the GL has no ", stringify!($name)).into());
                        }
                        std::mem::transmute::<*mut c_void, unsafe extern "C" fn($($arg),*) $(-> $ret)?>(symbol)
                    },)* })
                }
            }
        }
    };
}
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
library! { Egl {
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
} }
library! { Table {
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
    glDeleteTextures: fn(i32, *const u32);
    glBindTexture: fn(u32, u32);
    glTexImage2D: fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
    glTexSubImage2D: fn(u32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
    glTexParameteri: fn(u32, u32, i32);
    glTexImage3D: fn(u32, i32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
    glTexSubImage3D: fn(u32, i32, i32, i32, i32, i32, i32, i32, u32, u32, *const c_void);
    glFramebufferTextureLayer: fn(u32, u32, u32, i32, i32);
    glCopyTexSubImage3D: fn(u32, i32, i32, i32, i32, i32, i32, i32, i32);
    glGenerateMipmap: fn(u32);
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
    glBlitFramebuffer: fn(i32, i32, i32, i32, i32, i32, i32, i32, u32, u32);
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
    glFinish: fn();
    glActiveTexture: fn(u32);
    glDeleteBuffers: fn(i32, *const u32);
    glDeleteVertexArrays: fn(i32, *const u32);
} }

/// Where the context's frames go, and what keeps the context alive.
enum Surface {
    /// The handheld's framebuffer device through libmali's EGL.
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    Panel { egl: Egl, display: P, surface: P },
    /// An SDL window's context (a hidden one: the frames are read back).
    #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
    Window(
        *mut sdl3_sys::everything::SDL_Window,
        sdl3_sys::everything::SDL_GLContext,
    ),
}

/// The GL's entry points and the surface its context draws to, with the context current on the
/// thread that made it.
pub struct Gles {
    table: Table,
    surface: Surface,
    /// Whether this is a desktop OpenGL context, whose shader language names its version
    /// differently from ES's.
    desktop: bool,
    /// Keeps the type off `Send` and `Sync`: the context is current on one thread.
    _one_thread: std::marker::PhantomData<*const ()>,
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
/// A hidden SDL window made for an OpenGL context, handed to the thread that makes the context
/// current there (`Gles::open_window`).
pub struct GlWindow(*mut sdl3_sys::everything::SDL_Window);

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
// SAFETY: the pointer is a window nothing else uses; it moves to the one thread that makes the
// context on it and keeps it.
unsafe impl Send for GlWindow {}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
impl GlWindow {
    /// A hidden window with an OpenGL 3.3 core context asked for, made on the calling thread
    /// (the one that made the game's window). `None` when SDL cannot, with its message.
    pub fn new() -> Result<GlWindow, String> {
        use sdl3_sys::everything::*;
        // SAFETY: plain attribute values and a NUL-ended title; SDL is initialised (the game's
        // window exists).
        unsafe {
            SDL_GL_ResetAttributes();
            SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
            SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
            SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE.0);
            SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 0);
            let window = SDL_CreateWindow(
                c"rc2k GL".as_ptr(),
                64,
                64,
                SDL_WindowFlags(SDL_WINDOW_OPENGL.0 | SDL_WINDOW_HIDDEN.0),
            );
            if window.is_null() {
                return Err(format!("SDL makes no OpenGL window: {}", sdl_error()));
            }
            Ok(GlWindow(window))
        }
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
/// SDL's last error message.
fn sdl_error() -> String {
    // SAFETY: SDL gives a NUL-ended string that stays valid until its next call; copied here.
    unsafe {
        CStr::from_ptr(sdl3_sys::everything::SDL_GetError())
            .to_string_lossy()
            .into_owned()
    }
}

/// The bytes of `width` x `height` RGBA pixels, for a call that reads them.
#[inline]
fn pixel_bytes(width: i32, height: i32) -> usize {
    width.max(0) as usize * height.max(0) as usize * 4
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
impl Gles {
    /// Makes an OpenGL context on `window` current on this thread and finds the entry points
    /// by SDL.
    pub fn open_window(window: GlWindow) -> Result<Gles, (String, GlWindow)> {
        use sdl3_sys::everything::*;
        let failed = |why: String| (why, GlWindow(window.0));
        let window = window.0;
        // SAFETY: the window is ours alone and made for OpenGL; the context stays current on
        // this thread, which keeps the `Gles`; each name is NUL-ended.
        unsafe {
            let context = SDL_GL_CreateContext(window);
            if context.is_null() {
                return Err(failed(format!("no OpenGL context: {}", sdl_error())));
            }
            if !SDL_GL_MakeCurrent(window, context) {
                let why = format!("the OpenGL context is not made current: {}", sdl_error());
                SDL_GL_DestroyContext(context);
                return Err(failed(why));
            }
            SDL_GL_SetSwapInterval(0);
            let table = Table::load(&|name| match SDL_GL_GetProcAddress(name.as_ptr()) {
                Some(function) => function as P,
                None => std::ptr::null_mut(),
            });
            let table = match table {
                Ok(table) => table,
                Err(why) => {
                    SDL_GL_MakeCurrent(window, std::ptr::null_mut());
                    SDL_GL_DestroyContext(context);
                    return Err(failed(why));
                }
            };
            Ok(Gles {
                table,
                surface: Surface::Window(window, context),
                desktop: true,
                _one_thread: std::marker::PhantomData,
            })
        }
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
impl Gles {
    /// The context destroyed (it is current on the calling thread, the one that made it),
    /// and its window handed back: it is destroyed on the thread that made it.
    pub fn close(self) -> GlWindow {
        use sdl3_sys::everything::*;
        let Surface::Window(window, context) = self.surface;
        // SAFETY: the context `open_window` made, current on this thread, which owns the
        // `Gles`; nothing draws with it after this; the window is the one it was made on.
        unsafe {
            SDL_GL_MakeCurrent(window, std::ptr::null_mut());
            SDL_GL_DestroyContext(context);
        }
        GlWindow(window)
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
impl GlWindow {
    /// The hidden window destroyed, on the thread that made it, once the context on it is gone.
    pub fn destroy(self) {
        // SAFETY: a window `new` made, whose context is destroyed; not used again.
        unsafe { sdl3_sys::everything::SDL_DestroyWindow(self.0) };
    }
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
impl Gles {
    /// The panel's context stays for the program's life: nothing comes back.
    pub fn close_after_failure(self) -> Option<()> {
        None
    }
}

#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
impl Gles {
    /// `close`, for a start that failed after the context was made.
    pub fn close_after_failure(self) -> Option<GlWindow> {
        Some(self.close())
    }
}

impl Gles {
    /// Opens the library, makes an OpenGL ES 3 context current on the framebuffer device's
    /// `width` x `height` window (leaked: libmali holds on to it).
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    pub fn open_panel(width: u16, height: u16) -> Result<Gles, String> {
        // SAFETY: a constant name; each symbol is looked up by name and the library stays open
        // for the program's life.
        let handle = unsafe { libc::dlopen(c"/usr/lib/libmali.so".as_ptr(), libc::RTLD_NOW) };
        if handle.is_null() {
            return Err("no /usr/lib/libmali.so".into());
        }
        // SAFETY: `handle` is the open library; the names are NUL-ended.
        let lookup = |name: &CStr| unsafe { libc::dlsym(handle, name.as_ptr()) };
        let egl = Egl::load(&lookup)?;
        let table = Table::load(&lookup)?;
        // SAFETY: EGL's calls with the values their documents ask for; the attribute lists are
        // ended by EGL_NONE and live through their calls; the window record's lifetime is the
        // program's.
        unsafe {
            let display = (egl.eglGetDisplay)(std::ptr::null_mut());
            let (mut major, mut minor) = (0, 0);
            if (egl.eglInitialize)(display, &raw mut major, &raw mut minor) == 0 {
                return Err(format!("eglInitialize: {:#x}", (egl.eglGetError)()));
            }
            let wanted = [
                0x3024, 8, 0x3023, 8, 0x3022, 8, 0x3033, 4, 0x3040, 0x40, 0x3038i32,
            ];
            let (mut config, mut found) = (std::ptr::null_mut(), 0);
            (egl.eglChooseConfig)(display, wanted.as_ptr(), &raw mut config, 1, &raw mut found);
            if found == 0 {
                return Err("EGL has no such config".into());
            }
            (egl.eglBindAPI)(0x30A0);
            #[repr(C)]
            struct FbdevWindow {
                width: u16,
                height: u16,
            }
            let window = Box::leak(Box::new(FbdevWindow { width, height }));
            let surface = (egl.eglCreateWindowSurface)(
                display,
                config,
                std::ptr::from_mut(window).cast(),
                std::ptr::null(),
            );
            let attributes = [0x3098, 3, 0x3038i32];
            let context =
                (egl.eglCreateContext)(display, config, std::ptr::null_mut(), attributes.as_ptr());
            if surface.is_null()
                || context.is_null()
                || (egl.eglMakeCurrent)(display, surface, surface, context) == 0
            {
                return Err(format!("the EGL context: {:#x}", (egl.eglGetError)()));
            }
            (egl.eglSwapInterval)(display, 0);
            Ok(Gles {
                table,
                surface: Surface::Panel {
                    egl,
                    display,
                    surface,
                },
                desktop: false,
                _one_thread: std::marker::PhantomData,
            })
        }
    }

    #[inline]
    pub fn swap_buffers(&self) {
        match &self.surface {
            #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
            Surface::Panel {
                egl,
                display,
                surface,
            } => {
                // SAFETY: the display and surface made in `open_panel`, current on this thread.
                unsafe { (egl.eglSwapBuffers)(*display, *surface) };
            }
            #[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
            Surface::Window(window, _) => {
                // SAFETY: the window `open_window` made the context on, current on this thread.
                unsafe { sdl3_sys::everything::SDL_GL_SwapWindow(*window) };
            }
        }
    }

    /// A shader of `kind` made from `text`; the compiler's log (up to 600 bytes) on failure.
    ///
    /// The sources are written for ES 3.0 (`#version 300 es`); a desktop context gets the same
    /// text under desktop GLSL 3.30's version line, which takes the rest unchanged.
    pub fn compile_shader(&self, kind: u32, text: &CStr) -> Result<u32, String> {
        let desktop_text;
        let text = match text.to_bytes().strip_prefix(b"#version 300 es") {
            Some(rest) if self.desktop => {
                let mut bytes = b"#version 330 core".to_vec();
                bytes.extend_from_slice(rest);
                desktop_text = CString::new(bytes).map_err(|_| "a shader holds a NUL")?;
                desktop_text.as_c_str()
            }
            _ => text,
        };
        // SAFETY: the text lives through the call, ended by its NUL; the log's room is given.
        unsafe {
            let shader = (self.table.glCreateShader)(kind);
            let pointer = text.as_ptr();
            (self.table.glShaderSource)(shader, 1, &raw const pointer, std::ptr::null());
            (self.table.glCompileShader)(shader);
            let mut done = 0;
            (self.table.glGetShaderiv)(shader, 0x8B81, &raw mut done);
            if done == 0 {
                let mut log = [0 as c_char; 600];
                (self.table.glGetShaderInfoLog)(
                    shader,
                    600,
                    std::ptr::null_mut(),
                    log.as_mut_ptr(),
                );
                return Err(CStr::from_ptr(log.as_ptr()).to_string_lossy().into_owned());
            }
            Ok(shader)
        }
    }

    /// A program linking the two shaders.
    pub fn link_program(&self, vertex: u32, fragment: u32) -> u32 {
        // SAFETY: plain numbers; the GL checks the names.
        unsafe {
            let program = (self.table.glCreateProgram)();
            (self.table.glAttachShader)(program, vertex);
            (self.table.glAttachShader)(program, fragment);
            (self.table.glLinkProgram)(program);
            program
        }
    }

    #[inline]
    pub fn use_program(&self, program: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glUseProgram)(program) };
    }

    pub fn uniform_location(&self, program: u32, name: &str) -> i32 {
        let name = CString::new(name).expect("no NUL in a uniform's name");
        // SAFETY: the name lives through the call.
        unsafe { (self.table.glGetUniformLocation)(program, name.as_ptr()) }
    }

    #[inline]
    pub fn uniform_1i(&self, location: i32, value: i32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glUniform1i)(location, value) };
    }

    #[inline]
    pub fn uniform_1f(&self, location: i32, value: f32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glUniform1f)(location, value) };
    }

    #[inline]
    pub fn uniform_2f(&self, location: i32, x: f32, y: f32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glUniform2f)(location, x, y) };
    }

    #[inline]
    pub fn uniform_4fv(&self, location: i32, value: &[f32; 4]) {
        // SAFETY: four floats, read in the call.
        unsafe { (self.table.glUniform4fv)(location, 1, value.as_ptr()) };
    }

    pub fn gen_texture(&self) -> u32 {
        let mut name = 0;
        // SAFETY: one out-value.
        unsafe { (self.table.glGenTextures)(1, &raw mut name) };
        name
    }

    pub fn delete_texture(&self, name: u32) {
        // SAFETY: one value, read in the call.
        unsafe { (self.table.glDeleteTextures)(1, &raw const name) };
    }

    #[inline]
    pub fn bind_texture(&self, target: u32, name: u32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glBindTexture)(target, name) };
    }

    #[inline]
    pub fn active_texture(&self, unit: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glActiveTexture)(unit) };
    }

    /// `glTexImage2D` of level 0 of the bound 2D texture: `width` x `height` pixels, from `pixels`
    /// (at least that many bytes of four) or, with none, left unset.
    pub fn tex_image_2d(
        &self,
        internal_format: i32,
        width: i32,
        height: i32,
        format: u32,
        kind: u32,
        pixels: Option<&[u8]>,
    ) {
        if let Some(pixels) = pixels {
            assert!(
                pixels.len() >= pixel_bytes(width, height),
                "a short picture"
            );
        }
        // SAFETY: the pixels, when given, hold the picture (checked above), read in the call.
        unsafe {
            (self.table.glTexImage2D)(
                0x0DE1,
                0,
                internal_format,
                width,
                height,
                0,
                format,
                kind,
                pixels.map_or(std::ptr::null(), |p| p.as_ptr().cast()),
            );
        }
    }

    /// `glTexSubImage2D` of level 0 of the bound 2D texture: `width` x `height` pixels at (`x`, `y`) from `pixels`.
    /// A `TEXTURE_2D_ARRAY` of `layers` pictures of `width` x `height`, no pixels given.
    pub fn tex_image_3d_empty(&self, internal_format: i32, width: i32, height: i32, layers: i32) {
        self.tex_image_3d_level_empty(0, internal_format, width, height, layers);
    }

    /// Level `level` of the bound `TEXTURE_2D_ARRAY`: `layers` pictures of `width` x `height`,
    /// no pixels given.
    pub fn tex_image_3d_level_empty(
        &self,
        level: i32,
        internal_format: i32,
        width: i32,
        height: i32,
        layers: i32,
    ) {
        // SAFETY: plain numbers; no pixels are read (a null pointer).
        unsafe {
            (self.table.glTexImage3D)(
                0x8C1A,
                level,
                internal_format,
                width,
                height,
                layers,
                0,
                0x1908,
                0x1401,
                std::ptr::null(),
            );
        }
    }

    /// One layer of the bound `TEXTURE_2D_ARRAY`: `width` x `height` RGBA bytes.
    pub fn tex_sub_image_layer(&self, layer: i32, width: i32, height: i32, pixels: &[u8]) {
        self.tex_sub_image_layer_level(0, layer, width, height, pixels);
    }

    /// One layer of level `level` of the bound `TEXTURE_2D_ARRAY`: `width` x `height` RGBA bytes.
    pub fn tex_sub_image_layer_level(
        &self,
        level: i32,
        layer: i32,
        width: i32,
        height: i32,
        pixels: &[u8],
    ) {
        assert!(
            pixels.len() >= pixel_bytes(width, height),
            "a short picture"
        );
        // SAFETY: the pixels hold the picture (checked above), read in the call.
        unsafe {
            (self.table.glTexSubImage3D)(
                0x8C1A,
                level,
                0,
                0,
                layer,
                width,
                height,
                1,
                0x1908,
                0x1401,
                pixels.as_ptr().cast(),
            );
        }
    }

    /// Every smaller level of every layer of the bound `TEXTURE_2D_ARRAY` made from level 0.
    pub fn generate_mipmap_array(&self) {
        // SAFETY: plain number.
        unsafe { (self.table.glGenerateMipmap)(0x8C1A) };
    }

    /// Layer `layer` of the texture array `texture` as the bound `target` framebuffer's
    /// colour.
    pub fn framebuffer_texture_layer(&self, target: u32, texture: u32, layer: i32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glFramebufferTextureLayer)(target, 0x8CE0, texture, 0, layer) };
    }

    /// The read framebuffer's `width` x `height` pixels into layer `layer` of the bound
    /// `TEXTURE_2D_ARRAY`.
    pub fn copy_into_layer(&self, layer: i32, width: i32, height: i32) {
        // SAFETY: plain numbers; the GL reads its own framebuffer.
        unsafe { (self.table.glCopyTexSubImage3D)(0x8C1A, 0, 0, 0, layer, 0, 0, width, height) };
    }

    pub fn tex_sub_image_2d(
        &self,
        (x, y): (i32, i32),
        (width, height): (i32, i32),
        format: u32,
        kind: u32,
        pixels: &[u8],
    ) {
        assert!(
            pixels.len() >= pixel_bytes(width, height),
            "a short picture"
        );
        // SAFETY: the pixels hold the picture (checked above), read in the call.
        unsafe {
            (self.table.glTexSubImage2D)(
                0x0DE1,
                0,
                x,
                y,
                width,
                height,
                format,
                kind,
                pixels.as_ptr().cast(),
            );
        }
    }

    #[inline]
    pub fn tex_parameter(&self, target: u32, parameter: u32, value: i32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glTexParameteri)(target, parameter, value) };
    }

    /// Four new sampler objects.
    pub fn gen_samplers(&self) -> [u32; 8] {
        let mut names = [0; 8];
        // SAFETY: room for the eight out-values.
        unsafe { (self.table.glGenSamplers)(8, names.as_mut_ptr()) };
        names
    }

    #[inline]
    pub fn bind_sampler(&self, unit: u32, sampler: u32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glBindSampler)(unit, sampler) };
    }

    pub fn sampler_parameter(&self, sampler: u32, parameter: u32, value: i32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glSamplerParameteri)(sampler, parameter, value) };
    }

    pub fn gen_buffer(&self) -> u32 {
        let mut name = 0;
        // SAFETY: one out-value.
        unsafe { (self.table.glGenBuffers)(1, &raw mut name) };
        name
    }

    pub fn delete_buffer(&self, name: u32) {
        // SAFETY: one value, read in the call.
        unsafe { (self.table.glDeleteBuffers)(1, &raw const name) };
    }

    #[inline]
    pub fn bind_buffer(&self, target: u32, name: u32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glBindBuffer)(target, name) };
    }

    /// `glBufferData` of the records, which the GL copies in the call.
    #[inline]
    pub fn buffer_data<const N: usize>(&self, target: u32, records: &[[u32; N]], usage: u32) {
        // SAFETY: the records' memory, `size_of_val` bytes of it, read in the call.
        unsafe {
            (self.table.glBufferData)(
                target,
                size_of_val(records) as isize,
                records.as_ptr().cast(),
                usage,
            );
        }
    }

    /// `glBufferData` of the first `bytes` bytes of the records (the bench's small buffer).
    pub fn buffer_data_first<const N: usize>(
        &self,
        target: u32,
        records: &[[u32; N]],
        bytes: usize,
        usage: u32,
    ) {
        assert!(bytes <= size_of_val(records), "a short buffer");
        // SAFETY: `bytes` of the records' memory (checked above), read in the call.
        unsafe {
            (self.table.glBufferData)(target, bytes as isize, records.as_ptr().cast(), usage);
        }
    }

    pub fn gen_vertex_array(&self) -> u32 {
        let mut name = 0;
        // SAFETY: one out-value.
        unsafe { (self.table.glGenVertexArrays)(1, &raw mut name) };
        name
    }

    pub fn delete_vertex_array(&self, name: u32) {
        // SAFETY: one value, read in the call.
        unsafe { (self.table.glDeleteVertexArrays)(1, &raw const name) };
    }

    #[inline]
    pub fn bind_vertex_array(&self, name: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glBindVertexArray)(name) };
    }

    /// An attribute enabled and read from the bound buffer at byte `offset` (a position in the
    /// buffer, not a pointer, while a buffer is bound).
    pub fn attribute(
        &self,
        index: u32,
        count: i32,
        kind: u32,
        normalised: bool,
        stride: i32,
        offset: usize,
    ) {
        // SAFETY: the offset is a position in the bound array buffer; nothing is read before
        // a draw.
        unsafe {
            (self.table.glEnableVertexAttribArray)(index);
            (self.table.glVertexAttribPointer)(
                index,
                count,
                kind,
                normalised as u8,
                stride,
                offset as *const c_void,
            );
        }
    }

    pub fn gen_framebuffer(&self) -> u32 {
        let mut name = 0;
        // SAFETY: one out-value.
        unsafe { (self.table.glGenFramebuffers)(1, &raw mut name) };
        name
    }

    #[inline]
    pub fn bind_framebuffer(&self, target: u32, name: u32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glBindFramebuffer)(target, name) };
    }

    pub fn framebuffer_texture_2d(&self, target: u32, attachment: u32, texture: u32) {
        // SAFETY: plain numbers; level 0 of a 2D texture.
        unsafe { (self.table.glFramebufferTexture2D)(target, attachment, 0x0DE1, texture, 0) };
    }

    pub fn gen_renderbuffer(&self) -> u32 {
        let mut name = 0;
        // SAFETY: one out-value.
        unsafe { (self.table.glGenRenderbuffers)(1, &raw mut name) };
        name
    }

    pub fn bind_renderbuffer(&self, target: u32, name: u32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glBindRenderbuffer)(target, name) };
    }

    pub fn renderbuffer_storage(&self, target: u32, format: u32, width: i32, height: i32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glRenderbufferStorage)(target, format, width, height) };
    }

    pub fn framebuffer_renderbuffer(&self, target: u32, attachment: u32, renderbuffer: u32) {
        // SAFETY: plain numbers; the target is a renderbuffer's.
        unsafe {
            (self.table.glFramebufferRenderbuffer)(target, attachment, 0x8D41, renderbuffer);
        }
    }

    pub fn framebuffer_status(&self, target: u32) -> u32 {
        // SAFETY: a plain number.
        unsafe { (self.table.glCheckFramebufferStatus)(target) }
    }

    /// The colour of the read framebuffer's rectangle (0, 0) to `from` copied to the draw
    /// framebuffer's (0, 0) to `to`, each pixel the nearest.
    pub fn blit_colour_nearest(&self, from: (i32, i32), to: (i32, i32)) {
        // SAFETY: plain numbers; the framebuffers are bound by the caller.
        unsafe {
            (self.table.glBlitFramebuffer)(0, 0, from.0, from.1, 0, 0, to.0, to.1, 0x4000, 0x2600);
        }
    }

    #[inline]
    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glViewport)(x, y, width, height) };
    }

    #[inline]
    pub fn scissor(&self, x: i32, y: i32, width: i32, height: i32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glScissor)(x, y, width, height) };
    }

    pub fn clear_colour(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glClearColor)(red, green, blue, alpha) };
    }

    pub fn clear_depth(&self, depth: f32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glClearDepthf)(depth) };
    }

    pub fn clear(&self, bits: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glClear)(bits) };
    }

    #[inline]
    pub fn enable(&self, capability: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glEnable)(capability) };
    }

    #[inline]
    pub fn disable(&self, capability: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glDisable)(capability) };
    }

    #[inline]
    pub fn blend_func(&self, source: u32, target: u32) {
        // SAFETY: plain numbers.
        unsafe { (self.table.glBlendFunc)(source, target) };
    }

    #[inline]
    pub fn depth_func(&self, function: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glDepthFunc)(function) };
    }

    #[inline]
    pub fn depth_mask(&self, write: bool) {
        // SAFETY: a plain number.
        unsafe { (self.table.glDepthMask)(write as u8) };
    }

    #[inline]
    pub fn colour_mask(&self, write: bool) {
        let c = write as u8;
        // SAFETY: plain numbers.
        unsafe { (self.table.glColorMask)(c, c, c, c) };
    }

    #[inline]
    pub fn cull_face(&self, face: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glCullFace)(face) };
    }

    pub fn front_face(&self, direction: u32) {
        // SAFETY: a plain number.
        unsafe { (self.table.glFrontFace)(direction) };
    }

    /// Draws `count` vertices from `first` of the bound arrays (see the header: the caller
    /// sent that many).
    #[inline]
    pub fn draw_arrays(&self, mode: u32, first: i32, count: i32) {
        // SAFETY: plain numbers; the arrays' size is the caller's, as the header says.
        unsafe { (self.table.glDrawArrays)(mode, first, count) };
    }

    /// The bound framebuffer's `width` x `height` pixels as RGBA bytes into `out`.
    pub fn read_pixels(&self, width: i32, height: i32, format: u32, kind: u32, out: &mut [u8]) {
        assert!(out.len() >= pixel_bytes(width, height), "a short picture");
        // SAFETY: `out` holds the picture (checked above), written in the call.
        unsafe {
            (self.table.glReadPixels)(0, 0, width, height, format, kind, out.as_mut_ptr().cast());
        }
    }

    pub fn finish(&self) {
        // SAFETY: no arguments.
        unsafe { (self.table.glFinish)() };
    }
}
