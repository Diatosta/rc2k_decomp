//! SDL3 (`sdl3-sys`): the window, its renderer and frame texture, the events, the audio
//! device and the clock. The `unsafe` of those calls is here and nowhere else in the
//! platform files.
//!
//! What the wrappers promise: handles (`Window`, `Renderer`, `Texture`) are made only by
//! SDL's own constructors here and keep their pointer private; a null result is `None` or
//! `false`, never a handle; a texture remembers its size, so a pixel upload is checked
//! against it. SDL's video calls belong to the thread that opened the window (the
//! callers' rule, as before); the handles are `Send` only so `platform::SCREEN` can hold
//! them in a `Mutex`.

#![allow(unsafe_code)]

use sdl3_sys::everything::*;

/// How a window is shown on its display.
#[cfg_attr(handheld, allow(dead_code))] // the handheld's panel is always the whole screen
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FullScreen {
    /// As a window.
    Off,
    /// Over the whole desktop, without a mode change (borderless).
    Desktop,
    /// With the display set to the mode nearest this size.
    Exclusive(u32, u32),
}

// ---- handles ----

/// An SDL window.
#[derive(Clone, Copy)]
pub struct Window(pub(super) *mut SDL_Window);

/// A window's renderer.
#[derive(Clone, Copy)]
pub struct Renderer(pub(super) *mut SDL_Renderer);

/// A streaming texture of a renderer (its size kept for the upload's check).
#[cfg_attr(handheld, allow(dead_code))] // the handheld shows frames on its framebuffer
pub struct Texture {
    raw: *mut SDL_Texture,
    width: usize,
    height: usize,
}

// SAFETY: the handles are plain SDL pointers, used only by the thread that opened the window
// (the platform's rule); they are `Send` so the `Mutex` that holds the screen is.
unsafe impl Send for Window {}
// SAFETY: as `Window`.
unsafe impl Send for Renderer {}
// SAFETY: as `Window`.
unsafe impl Send for Texture {}

impl Window {
    /// The part of the window clear of the screen's notch, rounded corners and home indicator, as
    /// (x, y, width, height) in the window's pixels (SDL gives it in points, which on a
    /// high-density screen are fewer than the pixels); None where SDL has none to say.
    #[cfg(touch_screen)]
    pub fn safe_area(self) -> Option<[f32; 4]> {
        let mut rect = SDL_Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
        // SAFETY: the window SDL made; SDL writes the one rectangle.
        let known = unsafe { SDL_GetWindowSafeArea(self.0, &raw mut rect) };
        if !known || rect.w <= 0 || rect.h <= 0 {
            return None;
        }
        let ((pixels_wide, pixels_high), (points_wide, points_high)) =
            (self.pixel_size()?, self.point_size()?);
        let across = pixels_wide as f32 / points_wide as f32;
        let down = pixels_high as f32 / points_high as f32;
        Some([
            rect.x as f32 * across,
            rect.y as f32 * down,
            rect.w as f32 * across,
            rect.h as f32 * down,
        ])
    }

    /// The window restored when it is minimised.
    pub fn restore_if_minimised(self) {
        // SAFETY: the window SDL made, still alive (the screen holds it).
        unsafe {
            if SDL_GetWindowFlags(self.0).0 & SDL_WINDOW_MINIMIZED.0 != 0 {
                SDL_RestoreWindow(self.0);
            }
        }
    }

    /// SDL's text input on or off for the window (typed characters arrive as text events; on
    /// a device with a screen keyboard, it is shown).
    pub fn set_text_input(self, on: bool) {
        // SAFETY: as `restore_if_minimised`.
        unsafe {
            if on {
                SDL_StartTextInput(self.0);
            } else {
                SDL_StopTextInput(self.0);
            }
        }
    }

    /// Where the field being edited is, for the input method's candidate list and a screen
    /// keyboard to keep clear of it: a band across the middle of the window at `y` on the
    /// game's 800x600 screen.
    pub fn set_text_input_area(self, y: i32) {
        // SAFETY: as `restore_if_minimised`; the sizes are written to locals.
        unsafe {
            let (mut width, mut height) = (0, 0);
            if !SDL_GetWindowSize(self.0, &raw mut width, &raw mut height) {
                return;
            }
            let area = SDL_Rect {
                x: width / 8,
                y: y.clamp(0, 600) * height / 600,
                w: width * 3 / 4,
                h: (height / 20).max(1),
            };
            SDL_SetTextInputArea(self.0, &raw const area, 0);
        }
    }

    /// Whether the system has a screen keyboard SDL can show for the window.
    pub fn has_screen_keyboard(self) -> bool {
        // SAFETY: as `restore_if_minimised`.
        unsafe { SDL_HasScreenKeyboardSupport() }
    }

    /// The window shown (and raised, so that it takes the keyboard).
    #[cfg(not(handheld))]
    pub fn show_raised(self) {
        // SAFETY: as `restore_if_minimised`.
        unsafe {
            SDL_ShowWindow(self.0);
            SDL_RaiseWindow(self.0);
        }
    }

    /// Whether both handles are the one window.
    #[cfg(not(handheld))]
    pub fn is_same(self, other: Window) -> bool {
        self.0 == other.0
    }

    /// Whether the window is hidden.
    #[cfg(not(handheld))]
    pub fn is_hidden(self) -> bool {
        // SAFETY: as `restore_if_minimised`.
        unsafe { SDL_GetWindowFlags(self.0).0 & SDL_WINDOW_HIDDEN.0 != 0 }
    }

    /// The window hidden.
    #[cfg(not(handheld))]
    pub fn hide(self) {
        // SAFETY: as `restore_if_minimised`.
        unsafe { SDL_HideWindow(self.0) };
    }

    /// Where the window is on the desktop.
    #[cfg(not(handheld))]
    pub fn position(self) -> (i32, i32) {
        let (mut x, mut y) = (0, 0);
        // SAFETY: as `restore_if_minimised`; SDL writes the two integers.
        unsafe { SDL_GetWindowPosition(self.0, &raw mut x, &raw mut y) };
        (x, y)
    }

    /// The window moved to `(x, y)` on the desktop.
    #[cfg(not(handheld))]
    pub fn set_position(self, (x, y): (i32, i32)) {
        // SAFETY: as `restore_if_minimised`.
        unsafe { SDL_SetWindowPosition(self.0, x, y) };
    }

    /// The window's size in the screen's pixels.
    pub fn set_size(self, width: usize, height: usize) {
        // SAFETY: as `restore_if_minimised`.
        unsafe { SDL_SetWindowSize(self.0, width as i32, height as i32) };
    }

    /// The window's size in the screen's pixels, as `set_size` takes it.
    #[cfg(not(handheld))]
    pub fn window_size(self) -> Option<(u32, u32)> {
        let (mut width, mut height) = (0, 0);
        // SAFETY: as `restore_if_minimised`; SDL writes the two integers.
        let known = unsafe { SDL_GetWindowSize(self.0, &raw mut width, &raw mut height) };
        (known && width > 0 && height > 0).then_some((width as u32, height as u32))
    }

    /// The least size the player can drag the window to.
    #[cfg(not(handheld))]
    pub fn set_minimum_size(self, width: usize, height: usize) {
        // SAFETY: as `restore_if_minimised`.
        unsafe { SDL_SetWindowMinimumSize(self.0, width as i32, height as i32) };
    }

    /// The window's size in its own pixels (what it is drawn on), if SDL knows it.
    #[cfg_attr(handheld, allow(dead_code))] // the handheld's panel has no window
    pub fn pixel_size(self) -> Option<(u32, u32)> {
        let (mut width, mut height) = (0, 0);
        // SAFETY: as `restore_if_minimised`; SDL writes the two integers.
        let known = unsafe { SDL_GetWindowSizeInPixels(self.0, &raw mut width, &raw mut height) };
        (known && width > 0 && height > 0).then_some((width as u32, height as u32))
    }

    /// The window's size in points (its own coordinates; on a high-density screen smaller than
    /// its pixels), if SDL knows it.
    #[cfg(any(touch_screen, target_os = "ios"))]
    pub fn point_size(self) -> Option<(u32, u32)> {
        let (mut width, mut height) = (0, 0);
        // SAFETY: as `restore_if_minimised`; SDL writes the two integers.
        let known = unsafe { SDL_GetWindowSize(self.0, &raw mut width, &raw mut height) };
        (known && width > 0 && height > 0).then_some((width as u32, height as u32))
    }

    /// The part of the window clear of the screen's notch, rounded corners and home indicator, as
    /// (x, y, width, height) in the window's pixels, if SDL knows it. The window itself where
    /// nothing intrudes.
    #[cfg(target_os = "ios")]
    pub fn safe_area_pixels(self) -> Option<[u32; 4]> {
        let (pixels, points) = (self.pixel_size()?, self.point_size()?);
        let mut area = SDL_Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
        // SAFETY: as `restore_if_minimised`; SDL writes the rectangle.
        if !unsafe { SDL_GetWindowSafeArea(self.0, &raw mut area) } || area.w <= 0 || area.h <= 0 {
            return None;
        }
        let across = f64::from(pixels.0) / f64::from(points.0);
        let down = f64::from(pixels.1) / f64::from(points.1);
        let scaled = |value: i32, factor: f64| (f64::from(value.max(0)) * factor).round() as u32;
        let x = scaled(area.x, across).min(pixels.0 - 1);
        let y = scaled(area.y, down).min(pixels.1 - 1);
        let width = scaled(area.w, across).clamp(1, pixels.0 - x);
        let height = scaled(area.h, down).clamp(1, pixels.1 - y);
        Some([x, y, width, height])
    }

    /// The window's full-screen state set. False when SDL refuses (the window stays as it was).
    #[cfg_attr(handheld, allow(dead_code))] // the handheld's panel is always the whole screen
    pub fn set_full_screen(self, how: FullScreen) -> bool {
        // The browser's full screen takes the whole page, both canvases (SDL's own request
        // would take only the window's canvas, leaving the GL's out: a black picture).
        #[cfg(web)]
        {
            crate::platform::web::set_full_screen(how != FullScreen::Off)
        }
        #[cfg(not(web))]
        {
            self.set_full_screen_sdl(how)
        }
    }

    #[cfg(not(web))]
    #[cfg_attr(handheld, allow(dead_code))] // the handheld's panel is always the whole screen
    fn set_full_screen_sdl(self, how: FullScreen) -> bool {
        // SAFETY: as `restore_if_minimised`; the mode is a record of this call's own that SDL
        // fills in (all zero is the "none" it starts from) and reads during the call.
        unsafe {
            match how {
                FullScreen::Off => SDL_SetWindowFullscreen(self.0, false),
                FullScreen::Desktop => {
                    SDL_SetWindowFullscreenMode(self.0, std::ptr::null())
                        && SDL_SetWindowFullscreen(self.0, true)
                }
                FullScreen::Exclusive(width, height) => {
                    let mut mode: SDL_DisplayMode = std::mem::zeroed();
                    SDL_GetClosestFullscreenDisplayMode(
                        SDL_GetDisplayForWindow(self.0),
                        width as i32,
                        height as i32,
                        0.0,
                        false,
                        &raw mut mode,
                    ) && SDL_SetWindowFullscreenMode(self.0, &raw const mode)
                        && SDL_SetWindowFullscreen(self.0, true)
                }
            }
        }
    }

    /// Whether the window is full screen now.
    pub fn is_full_screen(self) -> bool {
        #[cfg(web)]
        {
            crate::platform::web::is_full_screen()
        }
        #[cfg(not(web))]
        {
            // SAFETY: as `restore_if_minimised`.
            unsafe { SDL_GetWindowFlags(self.0).0 & SDL_WINDOW_FULLSCREEN.0 != 0 }
        }
    }

    /// The refreshes a second of the display the window is on, if SDL knows them.
    pub fn display_refresh(self) -> Option<f64> {
        // The browser's display is not SDL's to know: the page measures its frames.
        #[cfg(web)]
        {
            crate::platform::web::display_refresh()
        }
        #[cfg(not(web))]
        {
            self.display_refresh_sdl()
        }
    }

    #[cfg(not(web))]
    fn display_refresh_sdl(self) -> Option<f64> {
        // SAFETY: as `restore_if_minimised`; the mode is SDL's own record, read at once and
        // only when not null.
        let (rate, numerator, denominator) = unsafe {
            let mode = SDL_GetCurrentDisplayMode(SDL_GetDisplayForWindow(self.0));
            if mode.is_null() {
                return None;
            }
            (
                (*mode).refresh_rate,
                (*mode).refresh_rate_numerator,
                (*mode).refresh_rate_denominator,
            )
        };
        if numerator > 0 && denominator > 0 {
            Some(f64::from(numerator) / f64::from(denominator))
        } else {
            (rate > 0.0).then_some(f64::from(rate))
        }
    }
}

impl Renderer {
    /// The name of SDL's driver the renderer runs on (`gpu`, `opengl`, `software`, ...).
    pub fn name(self) -> String {
        // SAFETY: the renderer SDL made; the name is SDL's own constant text, copied at once.
        let name = unsafe { SDL_GetRendererName(self.0) };
        if name.is_null() {
            return "unnamed".into();
        }
        // SAFETY: not null, a NUL-terminated text that lives as long as the renderer.
        unsafe { std::ffi::CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned()
    }

    /// The renderer's target cleared to black.
    pub fn clear_black(self) {
        // SAFETY: the renderer SDL made, alive while the screen holds it.
        unsafe {
            SDL_SetRenderDrawColor(self.0, 0, 0, 0, 255);
            SDL_RenderClear(self.0);
        }
    }

    /// Whether `present` waits for the display's refresh (off unless SDL was asked for it,
    /// as its `SDL_RENDER_VSYNC` hint does).
    pub fn present_waits(self) -> bool {
        let mut interval = 0;
        // SAFETY: the renderer SDL made, alive while the screen holds it; SDL writes the one
        // integer.
        unsafe { SDL_GetRenderVSync(self.0, &raw mut interval) && interval != 0 }
    }

    #[cfg_attr(
        target_os = "ios",
        allow(
            dead_code,
            reason = "used by the folder page, which iOS replaces with a message"
        )
    )]
    /// Everything drawn from now on magnified `factor` times (the first-run page's text).
    pub fn set_scale(self, factor: f32) {
        // SAFETY: the renderer SDL made.
        unsafe { SDL_SetRenderScale(self.0, factor, factor) };
    }

    #[cfg_attr(
        target_os = "ios",
        allow(
            dead_code,
            reason = "used by the folder page, which iOS replaces with a message"
        )
    )]
    /// The target cleared to a colour.
    pub fn clear(self, [r, g, b]: [u8; 3]) {
        // SAFETY: the renderer SDL made.
        unsafe {
            SDL_SetRenderDrawColor(self.0, r, g, b, 255);
            SDL_RenderClear(self.0);
        }
    }

    #[cfg_attr(
        target_os = "ios",
        allow(
            dead_code,
            reason = "used by the folder page, which iOS replaces with a message"
        )
    )]
    /// A filled rectangle in a colour (in the scaled units).
    pub fn fill(self, [r, g, b]: [u8; 3], [x, y, w, h]: [f32; 4]) {
        let rect = SDL_FRect { x, y, w, h };
        // SAFETY: the renderer SDL made; a rectangle that lives over the call.
        unsafe {
            SDL_SetRenderDrawColor(self.0, r, g, b, 255);
            SDL_RenderFillRect(self.0, &raw const rect);
        }
    }

    /// A rectangle blended over what is drawn (red, green, blue, alpha).
    #[cfg(phone)]
    pub fn fill_blended(self, [r, g, b, a]: [u8; 4], [x, y, w, h]: [f32; 4]) {
        let rect = SDL_FRect { x, y, w, h };
        // SAFETY: the renderer SDL made; a rectangle that lives over the call. The blend mode is
        // put back so that the frame's own drawing after this stays opaque.
        unsafe {
            SDL_SetRenderDrawBlendMode(self.0, SDL_BLENDMODE_BLEND);
            SDL_SetRenderDrawColor(self.0, r, g, b, a);
            SDL_RenderFillRect(self.0, &raw const rect);
            SDL_SetRenderDrawBlendMode(self.0, SDL_BLENDMODE_NONE);
        }
    }

    /// SDL's built-in font `text` at `scale` times its size (a text of 8 pixels a letter at 1),
    /// with its top left corner at (x, y), in a colour with alpha.
    #[cfg(phone)]
    pub fn text_blended(self, [r, g, b, a]: [u8; 4], x: f32, y: f32, scale: f32, text: &str) {
        let Ok(text) = std::ffi::CString::new(text.replace('\0', "")) else {
            return;
        };
        // SAFETY: the renderer SDL made; a NUL-ended text read during the call; the scale is
        // put back to what it was.
        unsafe {
            let (mut sx, mut sy) = (1.0, 1.0);
            SDL_GetRenderScale(self.0, &raw mut sx, &raw mut sy);
            SDL_SetRenderScale(self.0, scale, scale);
            SDL_SetRenderDrawBlendMode(self.0, SDL_BLENDMODE_BLEND);
            SDL_SetRenderDrawColor(self.0, r, g, b, a);
            SDL_RenderDebugText(self.0, x / scale, y / scale, text.as_ptr());
            SDL_SetRenderDrawBlendMode(self.0, SDL_BLENDMODE_NONE);
            SDL_SetRenderScale(self.0, sx, sy);
        }
    }

    #[cfg_attr(
        target_os = "ios",
        allow(
            dead_code,
            reason = "used by the folder page, which iOS replaces with a message"
        )
    )]
    /// SDL's built-in 8 by 8 font: `text` with its top left corner at (x, y).
    pub fn text(self, [r, g, b]: [u8; 3], x: f32, y: f32, text: &str) {
        let Ok(text) = std::ffi::CString::new(text.replace('\0', "")) else {
            return;
        };
        // SAFETY: the renderer SDL made; a NUL-ended text read during the call.
        unsafe {
            SDL_SetRenderDrawColor(self.0, r, g, b, 255);
            SDL_RenderDebugText(self.0, x, y, text.as_ptr());
        }
    }

    /// The pixels the renderer has drawn on its target, BGRA, top row first (a test run's shot of
    /// the window).
    #[cfg(feature = "remote")]
    #[cfg_attr(handheld, allow(dead_code))] // the handheld's panel is the frame's size: its frames are not on an SDL window
    pub fn read_window(self) -> Option<(u32, u32, Vec<u8>)> {
        // SAFETY: the renderer SDL made; the surface it returns is ours, converted into a second
        // one, and both are destroyed after their rows are copied (rows of `pitch` bytes,
        // `width` pixels of four in each).
        unsafe {
            let read = SDL_RenderReadPixels(self.0, std::ptr::null());
            if read.is_null() {
                return None;
            }
            let converted = SDL_ConvertSurface(read, SDL_PIXELFORMAT_ARGB8888);
            SDL_DestroySurface(read);
            if converted.is_null() {
                return None;
            }
            let surface = &*converted;
            let (width, height, pitch) = (
                surface.w as usize,
                surface.h as usize,
                surface.pitch as usize,
            );
            let mut out = Vec::with_capacity(width * height * 4);
            for y in 0..height {
                let row = std::slice::from_raw_parts(
                    surface.pixels.cast::<u8>().add(y * pitch),
                    width * 4,
                );
                out.extend_from_slice(row);
            }
            SDL_DestroySurface(converted);
            Some((width as u32, height as u32, out))
        }
    }

    /// What was drawn put on the window.
    pub fn present(self) {
        // SAFETY: as `clear_black`.
        unsafe { SDL_RenderPresent(self.0) };
    }

    /// The texture drawn over the whole target.
    #[cfg_attr(handheld, allow(dead_code))] // the handheld shows frames on its framebuffer
    pub fn draw_whole(self, texture: &Texture) {
        // SAFETY: the renderer and the texture made by it, both alive (null rectangles
        // mean the whole texture and the whole target).
        unsafe {
            SDL_RenderTexture(self.0, texture.raw, std::ptr::null(), std::ptr::null());
        }
    }

    /// The size of the target the renderer draws on (the window's pixels), if SDL knows it.
    #[cfg_attr(handheld, allow(dead_code))] // the handheld shows frames on its framebuffer
    pub fn window_pixels(self) -> Option<(u32, u32)> {
        let (mut width, mut height) = (0, 0);
        // SAFETY: the renderer SDL made; SDL writes the two integers.
        let known =
            unsafe { SDL_GetCurrentRenderOutputSize(self.0, &raw mut width, &raw mut height) };
        (known && width > 0 && height > 0).then_some((width as u32, height as u32))
    }

    /// `part` of the texture (x, y, width, height; `None`: all of it) drawn over `to`, a
    /// rectangle of the target.
    #[cfg_attr(handheld, allow(dead_code))] // the handheld shows frames on its framebuffer
    pub fn draw_part(self, texture: &Texture, part: Option<[u32; 4]>, to: [u32; 4]) {
        let rect = |[x, y, w, h]: [u32; 4]| SDL_FRect {
            x: x as f32,
            y: y as f32,
            w: w as f32,
            h: h as f32,
        };
        let (part, to) = (part.map(rect), rect(to));
        // SAFETY: the renderer and the texture made by it, both alive; the rectangles are read
        // during the call (a null source means the whole texture).
        unsafe {
            SDL_RenderTexture(
                self.0,
                texture.raw,
                part.as_ref().map_or(std::ptr::null(), std::ptr::from_ref),
                &raw const to,
            );
        }
    }

    /// An XRGB8888 streaming texture of the given size.
    pub fn create_frame_texture(self, width: usize, height: usize) -> Option<Texture> {
        // SAFETY: the renderer SDL made; the arguments are plain values.
        let raw = unsafe {
            SDL_CreateTexture(
                self.0,
                SDL_PIXELFORMAT_XRGB8888,
                SDL_TEXTUREACCESS_STREAMING,
                width as i32,
                height as i32,
            )
        };
        (!raw.is_null()).then_some(Texture { raw, width, height })
    }

    /// A texture that shows the GPU's colour target `colour` (ARGB8888, no blending).
    /// `colour` is a texture of the GPU device the renderer shares; nothing in Rust
    /// dereferences it.
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // an opaque handle only SDL reads
    #[cfg_attr(handheld, allow(dead_code))] // the handheld shows frames on its framebuffer
    pub fn create_gpu_view(
        self,
        colour: *mut SDL_GPUTexture,
        width: usize,
        height: usize,
    ) -> Option<Texture> {
        // SAFETY: SDL's properties, set and read in this block; the renderer and the
        // target are alive (the caller's screen and GPU); the properties are destroyed
        // after the texture takes them.
        unsafe {
            let properties = SDL_CreateProperties();
            SDL_SetPointerProperty(
                properties,
                SDL_PROP_TEXTURE_CREATE_GPU_TEXTURE_POINTER,
                colour.cast(),
            );
            SDL_SetNumberProperty(
                properties,
                SDL_PROP_TEXTURE_CREATE_FORMAT_NUMBER,
                SDL_PIXELFORMAT_ARGB8888.0 as i64,
            );
            SDL_SetNumberProperty(
                properties,
                SDL_PROP_TEXTURE_CREATE_WIDTH_NUMBER,
                width as i64,
            );
            SDL_SetNumberProperty(
                properties,
                SDL_PROP_TEXTURE_CREATE_HEIGHT_NUMBER,
                height as i64,
            );
            let made = SDL_CreateTextureWithProperties(self.0, properties);
            SDL_DestroyProperties(properties);
            if made.is_null() {
                return None;
            }
            SDL_SetTextureBlendMode(made, SDL_BLENDMODE_NONE);
            Some(Texture {
                raw: made,
                width,
                height,
            })
        }
    }
}

impl Texture {
    /// Whether the texture is magnified with no filter (sharp pixels) or with a linear one.
    #[cfg_attr(handheld, allow(dead_code))] // the handheld shows frames on its framebuffer
    pub fn set_sharp(&self, sharp: bool) {
        let mode = if sharp {
            SDL_SCALEMODE_NEAREST
        } else {
            SDL_SCALEMODE_LINEAR
        };
        // SAFETY: the texture SDL made, alive while `self` is.
        unsafe { SDL_SetTextureScaleMode(self.raw, mode) };
    }

    /// The whole texture replaced by `frame`'s rows of `pitch` bytes. False when SDL
    /// refuses, or when `frame` is shorter than the texture's rows.
    #[cfg_attr(handheld, allow(dead_code))] // the handheld shows frames on its framebuffer
    pub fn upload(&self, frame: &[u8], pitch: usize) -> bool {
        if frame.len() < pitch * self.height || pitch < self.width * 4 {
            return false;
        }
        // SAFETY: the texture SDL made; `frame` holds `height` rows of `pitch` bytes
        // (checked above), `width` XRGB8888 pixels in each; a null rectangle is all of it.
        unsafe {
            SDL_UpdateTexture(
                self.raw,
                std::ptr::null(),
                frame.as_ptr().cast(),
                pitch as i32,
            )
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        // SAFETY: the texture SDL made, owned by this value alone.
        unsafe { SDL_DestroyTexture(self.raw) };
    }
}

// ---- video ----

/// The handheld's frames go to the framebuffer: SDL's video driver is the dummy one.
#[cfg(handheld)]
pub fn use_dummy_video_driver() {
    // SAFETY: two constant C texts.
    unsafe { SDL_SetHint(SDL_HINT_VIDEO_DRIVER, c"dummy".as_ptr()) };
}

/// SDL's video driver is the offscreen one (a render to a file needs no window on a screen).
pub fn use_offscreen_video() {
    // SAFETY: two constant C texts.
    unsafe { SDL_SetHint(SDL_HINT_VIDEO_DRIVER, c"offscreen".as_ptr()) };
}

/// SDL's audio driver is the dummy one (no sound to a device).
pub fn use_dummy_audio() {
    // SAFETY: two constant C texts.
    unsafe { SDL_SetHint(SDL_HINT_AUDIO_DRIVER, c"dummy".as_ptr()) };
}

/// SDL's own log lines, at warning level and above for the GPU, video, render and error
/// categories, written to the program's log (`rc2k.log`), so a phone's log says why SDL refused a
/// GPU device or a window. SDL calls this from any thread.
#[cfg(target_os = "android")]
unsafe extern "C" fn sdl_log_to_program_log(
    _userdata: *mut std::ffi::c_void,
    category: std::ffi::c_int,
    priority: SDL_LogPriority,
    message: *const std::ffi::c_char,
) {
    if message.is_null() {
        return;
    }
    // SAFETY: SDL passes a NUL-ended text valid for the call; copied at once.
    let text = unsafe { std::ffi::CStr::from_ptr(message) }.to_string_lossy();
    crate::logfile::program_log::log(&format!(
        "sdl[category {category}, priority {}]: {text}",
        priority.0
    ));
}

/// Sends SDL's own log to the program's log, with the categories that explain a refused GPU
/// device or window at warning level (Android; elsewhere SDL's output reaches the terminal).
#[cfg(target_os = "android")]
fn route_sdl_log() {
    // SAFETY: the callback is a function with SDL's signature that uses no user data; the
    // priorities are plain values.
    unsafe {
        SDL_SetLogOutputFunction(Some(sdl_log_to_program_log), std::ptr::null_mut());
        for category in [
            SDL_LOG_CATEGORY_GPU,
            SDL_LOG_CATEGORY_VIDEO,
            SDL_LOG_CATEGORY_RENDER,
            SDL_LOG_CATEGORY_ERROR,
            SDL_LOG_CATEGORY_SYSTEM,
        ] {
            SDL_SetLogPriority(category.0, SDL_LOG_PRIORITY_WARN);
        }
    }
}

/// SDL's video started; false when it does not start.
pub fn init_video() -> bool {
    #[cfg(target_os = "android")]
    route_sdl_log();
    #[cfg(target_os = "ios")]
    // SAFETY: SDL's own hint name and NUL-ended value, read during the call. The game is drawn
    // for a wide screen.
    unsafe {
        SDL_SetHint(
            SDL_HINT_ORIENTATIONS,
            c"LandscapeLeft LandscapeRight".as_ptr(),
        )
    };
    // SAFETY: SDL's own start-up, on the thread that will own the window.
    let started = unsafe { SDL_Init(SDL_INIT_VIDEO) };
    // A phone left alone for a while must not dim in a race (a pad or the screen's own touches
    // are not counted as use by the system).
    #[cfg(target_os = "ios")]
    if started {
        // SAFETY: no arguments; SDL's video is started.
        unsafe { SDL_DisableScreenSaver() };
    }
    started
}

/// A display mode's size in pixels. SDL lists a high-density screen's modes in points (an
/// iPhone's 852 x 393 for its 2556 x 1179 pixels) and gives the points' size in pixels as the
/// mode's pixel density.
fn in_pixels(mode: &SDL_DisplayMode) -> (u32, u32) {
    let density = if cfg!(target_os = "ios") && mode.pixel_density >= 1.0 {
        f64::from(mode.pixel_density)
    } else {
        1.0
    };
    (
        (f64::from(mode.w) * density).round() as u32,
        (f64::from(mode.h) * density).round() as u32,
    )
}

/// A display's modes, each (width, height, refresh rate), and its desktop's size.
pub type DisplayModes = (Vec<(u32, u32, f32)>, Option<(u32, u32)>);

/// The modes of the display `window` is on (the primary display without a window): each
/// (width, height, refresh rate), and the desktop's size. SDL's video is started if it is not.
#[cfg_attr(any(test, handheld), allow(dead_code))] // the unit tests and the handheld ask no display
pub fn display_modes(window: Option<Window>) -> DisplayModes {
    if !init_video() {
        return (Vec::new(), None);
    }
    // SAFETY: SDL's video is up; the display id is SDL's own; the list SDL returns holds `count`
    // pointers to modes that live as long as the list, read at once and the list freed once
    // (the modes are not used after); the desktop mode is read at once and only when not null.
    unsafe {
        let display = match window {
            Some(w) => SDL_GetDisplayForWindow(w.0),
            None => SDL_GetPrimaryDisplay(),
        };
        let mut modes = Vec::new();
        let mut count = 0;
        let list = SDL_GetFullscreenDisplayModes(display, &raw mut count);
        if !list.is_null() {
            for n in 0..usize::try_from(count).unwrap_or(0) {
                let mode = *list.add(n);
                if !mode.is_null() && (*mode).w > 0 && (*mode).h > 0 {
                    let (w, h) = in_pixels(&*mode);
                    modes.push((w, h, (*mode).refresh_rate));
                }
            }
            SDL_free(list.cast());
        }
        let desktop = SDL_GetDesktopDisplayMode(display);
        let desktop = (!desktop.is_null() && (*desktop).w > 0 && (*desktop).h > 0)
            .then(|| in_pixels(&*desktop));
        (modes, desktop)
    }
}

/// What a window is made with: on iOS every window fills the screen and is drawn in the
/// screen's own pixels (two or three to a point); elsewhere nothing special.
fn window_flags() -> SDL_WindowFlags {
    if cfg!(target_os = "ios") {
        SDL_WindowFlags(SDL_WINDOW_FULLSCREEN.0 | SDL_WINDOW_HIGH_PIXEL_DENSITY.0)
    } else if cfg!(target_os = "android") {
        SDL_WINDOW_FULLSCREEN
    } else {
        SDL_WindowFlags(0)
    }
}
/// Whether the player can drag the window to another size: on the desktop systems (the others'
/// windows are the screen, or the page's canvas).
pub const RESIZABLE: bool = cfg!(any(
    target_os = "windows",
    target_os = "macos",
    all(target_os = "linux", not(target_arch = "aarch64"))
));

/// The least size of a window the player resizes (the least race size the options keep).
#[cfg(not(handheld))]
pub const MIN_WINDOW: (usize, usize) = (320, 240);

/// A window of the given size with a title.
pub fn create_window(title: &std::ffi::CStr, width: usize, height: usize) -> Option<Window> {
    // SAFETY: a NUL-ended title; plain sizes and SDL's own flags.
    let raw = unsafe {
        SDL_CreateWindow(
            title.as_ptr(),
            width as i32,
            height as i32,
            SDL_WindowFlags(window_flags().0 | if RESIZABLE { SDL_WINDOW_RESIZABLE.0 } else { 0 }),
        )
    };
    #[cfg(not(handheld))]
    if !raw.is_null() && RESIZABLE {
        Window(raw).set_minimum_size(MIN_WINDOW.0, MIN_WINDOW.1);
    }
    (!raw.is_null()).then_some(Window(raw))
}

/// The window's icon (title bar, task bar, window switcher): pixels three bytes each (red,
/// green, blue), rows without a gap. False when SDL does not take it.
pub fn set_window_icon(window: Window, width: usize, height: usize, rgb: &[u8]) -> bool {
    if width == 0 || height == 0 || rgb.len() < width * height * 3 {
        return false;
    }
    // SAFETY: the surface only borrows `rgb`, which outlives it (it is destroyed before this
    // returns; SDL copies the pixels it keeps for the icon); the sizes match the slice.
    unsafe {
        let surface = SDL_CreateSurfaceFrom(
            width as i32,
            height as i32,
            SDL_PIXELFORMAT_RGB24,
            rgb.as_ptr() as *mut core::ffi::c_void,
            (width * 3) as i32,
        );
        if surface.is_null() {
            return false;
        }
        let done = SDL_SetWindowIcon(window.0, surface);
        SDL_DestroySurface(surface);
        done
    }
}

/// The program's identifier for the desktop (Wayland's application id, X11's window class):
/// the name of its menu entry, so the desktop shows that entry's icon for the window.
pub fn set_app_identifier(id: &std::ffi::CStr) {
    // SAFETY: SDL's property name and a NUL-ended text, both read during the call.
    unsafe {
        SDL_SetAppMetadataProperty(SDL_PROP_APP_METADATA_IDENTIFIER_STRING, id.as_ptr());
    }
}

#[cfg_attr(
    target_os = "ios",
    allow(
        dead_code,
        reason = "used by the folder page, which iOS replaces with a message"
    )
)]
/// A window and its renderer given back (the first-run page's, before the game's own window).
pub fn destroy(window: Window, renderer: Renderer) {
    // SAFETY: both made by SDL here and not used again by the caller.
    unsafe {
        SDL_DestroyRenderer(renderer.0);
        SDL_DestroyWindow(window.0);
    }
}

/// The window's renderer destroyed, the window kept (the 3D's renderer is swapped). Every
/// texture made on it must be gone first; it is not used again.
#[cfg(not(handheld))]
pub fn destroy_renderer(renderer: Renderer) {
    // SAFETY: a renderer SDL made here, which the caller says nothing uses.
    unsafe { SDL_DestroyRenderer(renderer.0) };
}

/// The window's own renderer (SDL's choice of driver; the browser's is the software one, whose
/// frames SDL puts on the page's canvas from the page's thread: a WebGL context cannot be made
/// on the game's worker).
pub fn create_renderer(window: Window) -> Option<Renderer> {
    #[cfg(web)]
    let name = c"software".as_ptr();
    #[cfg(not(web))]
    let name = std::ptr::null();
    // SAFETY: the window SDL made; a null name lets SDL choose, else a constant C text.
    let raw = unsafe { SDL_CreateRenderer(window.0, name) };
    (!raw.is_null()).then_some(Renderer(raw))
}

/// The window's renderer on the GPU device the 3D shares. `device` is the GPU module's
/// device; nothing in Rust dereferences it.
#[cfg(not(handheld))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // an opaque handle only SDL reads
pub fn create_gpu_renderer(device: *mut SDL_GPUDevice, window: Window) -> Option<Renderer> {
    // SAFETY: a live GPU device (the caller made it) and the window SDL made.
    let raw = unsafe { SDL_CreateGPURenderer(device, window.0) };
    (!raw.is_null()).then_some(Renderer(raw))
}

// ---- events ----

/// An event SDL gave, with the reads of its members checked by its type.
pub struct PolledEvent(SDL_Event);

/// SDL's next event, if there is one.
pub fn poll_event() -> Option<PolledEvent> {
    // SAFETY: an all-zero `SDL_Event` is valid (a union of plain records); SDL fills it.
    let mut event: SDL_Event = unsafe { std::mem::zeroed() };
    // SAFETY: `event` is a whole `SDL_Event` SDL may write.
    if unsafe { SDL_PollEvent(&raw mut event) } {
        Some(PolledEvent(event))
    } else {
        None
    }
}

impl PolledEvent {
    #[cfg_attr(
        target_os = "ios",
        allow(
            dead_code,
            reason = "used by the folder page, which iOS replaces with a message"
        )
    )]
    /// A mouse button going down: where, in the window's pixels.
    pub fn click(&self) -> Option<(f32, f32)> {
        if self.kind() != SDL_EVENT_MOUSE_BUTTON_DOWN {
            return None;
        }
        // SAFETY: the type says the `button` record is the one SDL filled.
        let button = unsafe { self.0.button };
        Some((button.x, button.y))
    }

    /// A finger's change on the touch screen: its id, what it did and where, as a part of the
    /// window (0 .. 1 across and down).
    #[cfg(touch_screen)]
    pub fn finger(&self) -> Option<(u64, rc2k_core::input::menu_touch::Phase, f32, f32)> {
        use rc2k_core::input::menu_touch::Phase;
        let phase = match self.kind() {
            SDL_EVENT_FINGER_DOWN => Phase::Down,
            SDL_EVENT_FINGER_MOTION => Phase::Move,
            SDL_EVENT_FINGER_UP | SDL_EVENT_FINGER_CANCELED => Phase::Up,
            _ => return None,
        };
        // SAFETY: the type says the `tfinger` record is the one SDL filled.
        let finger = unsafe { self.0.tfinger };
        Some((finger.fingerID.0, phase, finger.x, finger.y))
    }

    #[cfg_attr(
        target_os = "ios",
        allow(
            dead_code,
            reason = "used by the folder page, which iOS replaces with a message"
        )
    )]
    /// A file or folder dropped on the window: its path.
    pub fn dropped(&self) -> Option<String> {
        if self.kind() != SDL_EVENT_DROP_FILE {
            return None;
        }
        // SAFETY: the type says the `drop` record is the one SDL filled.
        let data = unsafe { self.0.drop.data };
        if data.is_null() {
            return None;
        }
        // SAFETY: not null; SDL's NUL-ended text, valid while the event is.
        Some(
            unsafe { std::ffi::CStr::from_ptr(data) }
                .to_string_lossy()
                .into_owned(),
        )
    }

    /// SDL's event, for the debug UI's own reads.
    pub fn as_sdl(&self) -> &SDL_Event {
        &self.0
    }

    /// The event's type.
    pub fn kind(&self) -> SDL_EventType {
        // SAFETY: `type` is the first member of every record of the union.
        SDL_EventType(unsafe { self.0.r#type })
    }

    /// A key going down for the first time (no repeat) with Ctrl held, and its scancode.
    pub fn is_fresh_ctrl_key_down(&self, scancode: SDL_Scancode) -> bool {
        if self.kind() != SDL_EVENT_KEY_DOWN {
            return false;
        }
        // SAFETY: the type says the `key` record is the one SDL filled.
        let key = unsafe { self.0.key };
        key.scancode == scancode && !key.repeat && key.r#mod.0 & SDL_KMOD_CTRL.0 != 0
    }

    /// A key going down for the first time (no repeat) with Alt held, and its scancode.
    pub fn is_fresh_alt_key_down(&self, scancode: SDL_Scancode) -> bool {
        if self.kind() != SDL_EVENT_KEY_DOWN {
            return false;
        }
        // SAFETY: the type says the `key` record is the one SDL filled.
        let key = unsafe { self.0.key };
        key.scancode == scancode && !key.repeat && key.r#mod.0 & SDL_KMOD_ALT.0 != 0
    }

    /// The joystick id of a joystick added or removed event (else 0).
    pub fn joystick_id(&self) -> SDL_JoystickID {
        if self.kind() == SDL_EVENT_JOYSTICK_ADDED || self.kind() == SDL_EVENT_JOYSTICK_REMOVED {
            // SAFETY: the type says the `jdevice` record is the one SDL filled.
            unsafe { self.0.jdevice.which }
        } else {
            SDL_JoystickID(0)
        }
    }

    /// What a joystick event says (its joystick, then a hat's bits or a button going down or up),
    /// else `None`. A gamepad's joystick sends these too.
    pub fn joystick_input(&self) -> Option<(SDL_JoystickID, JoystickInput)> {
        let kind = self.kind();
        // SAFETY: each branch reads the record its type says SDL filled.
        unsafe {
            if kind == SDL_EVENT_JOYSTICK_HAT_MOTION {
                let e = self.0.jhat;
                Some((e.which, JoystickInput::Hat(e.hat, e.value)))
            } else if kind == SDL_EVENT_JOYSTICK_BUTTON_DOWN || kind == SDL_EVENT_JOYSTICK_BUTTON_UP
            {
                let e = self.0.jbutton;
                Some((e.which, JoystickInput::Button(e.button, e.down)))
            } else {
                None
            }
        }
    }

    /// The gamepad, the button and the direction of a gamepad button event (else `None`).
    pub fn gamepad_button(&self) -> Option<(SDL_JoystickID, SDL_GamepadButton, bool)> {
        if self.kind() != SDL_EVENT_GAMEPAD_BUTTON_DOWN
            && self.kind() != SDL_EVENT_GAMEPAD_BUTTON_UP
        {
            return None;
        }
        // SAFETY: the type says the `gbutton` record is the one SDL filled.
        let button = unsafe { self.0.gbutton };
        Some((
            button.which,
            SDL_GamepadButton(button.button as i32),
            button.down,
        ))
    }

    #[cfg_attr(
        target_os = "ios",
        allow(
            dead_code,
            reason = "used by the folder page, which iOS replaces with a message"
        )
    )]
    /// The text of a text-input event (UTF-8, as SDL gives it), else `None`.
    pub fn text(&self) -> Option<String> {
        if self.kind() != SDL_EVENT_TEXT_INPUT {
            return None;
        }
        // SAFETY: the type says the `text` record is the one SDL filled.
        let text = unsafe { self.0.text.text };
        if text.is_null() {
            return None;
        }
        // SAFETY: SDL's text is a NUL-ended string that lives until the next poll.
        Some(
            unsafe { std::ffi::CStr::from_ptr(text) }
                .to_string_lossy()
                .into_owned(),
        )
    }

    /// The axis and the value of a gamepad axis event (else `None`).
    pub fn gamepad_axis(&self) -> Option<(SDL_GamepadAxis, i16)> {
        if self.kind() != SDL_EVENT_GAMEPAD_AXIS_MOTION {
            return None;
        }
        // SAFETY: the type says the `gaxis` record is the one SDL filled.
        let axis = unsafe { self.0.gaxis };
        Some((SDL_GamepadAxis(axis.axis as i32), axis.value))
    }

    /// The joystick a gamepad axis event came from (0 for a stand-in's, which names none), else
    /// `None`.
    #[cfg(phone)]
    pub fn gamepad_axis_joystick(&self) -> Option<SDL_JoystickID> {
        if self.kind() != SDL_EVENT_GAMEPAD_AXIS_MOTION {
            return None;
        }
        // SAFETY: the type says the `gaxis` record is the one SDL filled.
        Some(unsafe { self.0.gaxis.which })
    }

    /// The scancode and the direction of a key event (else `None`).
    pub fn key(&self) -> Option<(i32, bool)> {
        if self.kind() != SDL_EVENT_KEY_DOWN && self.kind() != SDL_EVENT_KEY_UP {
            return None;
        }
        // SAFETY: the type says the `key` record is the one SDL filled.
        let key = unsafe { self.0.key };
        Some((key.scancode.0, key.down))
    }
}

// ---- audio ----

/// Why the audio output did not start.
pub enum AudioError {
    /// SDL's audio subsystem does not start.
    Init,
    /// SDL opens no playback device.
    NoDevice,
}

/// What the audio thread asks for: the bytes the device wants (stereo 16-bit frames); the
/// samples to give (two per frame), or none to give nothing this time.
static MIXER: std::sync::OnceLock<Mixer> = std::sync::OnceLock::new();

/// A mixer: the bytes wanted in, the samples out.
pub type Mixer = fn(wanted_bytes: i32) -> Option<Vec<i16>>;

/// SDL's audio callback: what `MIXER` makes goes to the stream.
extern "C" fn feed(_: *mut core::ffi::c_void, stream: *mut SDL_AudioStream, wanted: i32, _: i32) {
    let Some(mixer) = MIXER.get() else {
        return;
    };
    if let Some(pcm) = mixer(wanted) {
        // SAFETY: the stream SDL called back with; `pcm`'s bytes, their count.
        unsafe { SDL_PutAudioStreamData(stream, pcm.as_ptr().cast(), (pcm.len() * 2) as i32) };
    }
}

/// The default playback device opened for stereo 16-bit little-endian at `rate` and
/// started; SDL then calls `mixer` on its audio thread. A panic in `mixer` must not unwind
/// into SDL: the caller catches it.
pub fn start_audio_output(rate: u32, mixer: Mixer) -> Result<(), AudioError> {
    let _ = MIXER.set(mixer);
    // SAFETY: SDL started on the main thread; the spec is copied by SDL and `feed` is a
    // plain function that lives as long as the program.
    unsafe {
        if !SDL_InitSubSystem(SDL_INIT_AUDIO) {
            return Err(AudioError::Init);
        }
        let spec = SDL_AudioSpec {
            format: SDL_AUDIO_S16LE,
            channels: 2,
            freq: rate as i32,
        };
        let stream = SDL_OpenAudioDeviceStream(
            SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK,
            &raw const spec,
            Some(feed),
            std::ptr::null_mut(),
        );
        if stream.is_null() {
            return Err(AudioError::NoDevice);
        }
        SDL_ResumeAudioStreamDevice(stream);
    }
    Ok(())
}

// ---- the calendar, message boxes ----

/// The local date and time now, or None when the system gives none (no SDL start-up
/// needed).
pub fn local_date_time() -> Option<SDL_DateTime> {
    let mut now = 0;
    let mut dt = SDL_DateTime::default();
    // SAFETY: two out-parameters of documented calls that need no SDL start-up.
    let read =
        unsafe { SDL_GetCurrentTime(&raw mut now) && SDL_TimeToDateTime(now, &raw mut dt, true) };
    read.then_some(dt)
}

/// A simple message box with no parent window, shown until it is closed.
pub fn show_message_box(
    flags: SDL_MessageBoxFlags,
    caption: &std::ffi::CStr,
    text: &std::ffi::CStr,
) {
    // SAFETY: both texts are NUL-ended and live over the call; no window.
    unsafe {
        SDL_ShowSimpleMessageBox(flags, caption.as_ptr(), text.as_ptr(), std::ptr::null_mut())
    };
}

// ---- the dialogs ----

#[cfg_attr(
    target_os = "ios",
    allow(
        dead_code,
        reason = "used by the folder page, which iOS replaces with a message"
    )
)]
/// What a dialog gave: `None` while it is open, `Some(None)` when it was cancelled or failed.
static DIALOG: std::sync::Mutex<Option<Option<String>>> = std::sync::Mutex::new(None);

#[cfg_attr(
    target_os = "ios",
    allow(
        dead_code,
        reason = "used by the folder page, which iOS replaces with a message"
    )
)]
unsafe extern "C" fn dialog_done(
    _user: *mut core::ffi::c_void,
    list: *const *const core::ffi::c_char,
    _filter: core::ffi::c_int,
) {
    let first = if list.is_null() {
        None
    } else {
        // SAFETY: SDL's list ends with a null; its first item is read, if any.
        let item = unsafe { *list };
        // SAFETY: a non-null item is a NUL-ended text valid during the callback.
        (!item.is_null()).then(|| {
            // SAFETY: as above, checked not null.
            unsafe { std::ffi::CStr::from_ptr(item) }
                .to_string_lossy()
                .into_owned()
        })
    };
    *DIALOG
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(first);
}

#[cfg_attr(
    target_os = "ios",
    allow(
        dead_code,
        reason = "used by the folder page, which iOS replaces with a message"
    )
)]
/// The system's open dialog for a folder (or for a zip file) shown over `window`; its answer
/// comes by `dialog_answer`.
pub fn show_open_dialog(window: Window, folder: bool) {
    *DIALOG
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    let filters = [SDL_DialogFileFilter {
        name: c"Zip files".as_ptr(),
        pattern: c"zip".as_ptr(),
    }];
    // SAFETY: the callback is `extern "C"` and touches only a static; the filters live over
    // the call (SDL copies them); the window SDL made; no default location.
    unsafe {
        if folder {
            SDL_ShowOpenFolderDialog(
                Some(dialog_done),
                std::ptr::null_mut(),
                window.0,
                std::ptr::null(),
                false,
            );
        } else {
            SDL_ShowOpenFileDialog(
                Some(dialog_done),
                std::ptr::null_mut(),
                window.0,
                filters.as_ptr(),
                1,
                std::ptr::null(),
                false,
            );
        }
    }
}

/// What a picked `content://` address (Android's file picker) holds, copied to the file `to`
/// through SDL's own reading of it; the length copied.
#[cfg(target_os = "android")]
pub fn copy_content(uri: &str, to: &std::path::Path) -> Result<u64, String> {
    use std::io::Write;
    let name = std::ffi::CString::new(uri).map_err(|e| e.to_string())?;
    // SAFETY: a NUL-ended name that lives over the call; null (failure) checked.
    let stream = unsafe { SDL_IOFromFile(name.as_ptr(), c"rb".as_ptr()) };
    if stream.is_null() {
        return Err(format!("{uri} cannot be opened"));
    }
    let mut out = std::fs::File::create(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let mut buffer = vec![0u8; 1 << 20];
    let mut total = 0u64;
    let result = loop {
        // SAFETY: the stream just opened; SDL writes at most the buffer's length into it.
        let got = unsafe { SDL_ReadIO(stream, buffer.as_mut_ptr().cast(), buffer.len()) };
        if got == 0 {
            break Ok(total);
        }
        if let Err(e) = out.write_all(&buffer[..got]) {
            break Err(format!("{}: {e}", to.display()));
        }
        total += got as u64;
    };
    // SAFETY: the stream opened above, closed once.
    unsafe { SDL_CloseIO(stream) };
    result
}

#[cfg_attr(
    target_os = "ios",
    allow(
        dead_code,
        reason = "used by the folder page, which iOS replaces with a message"
    )
)]
/// The answer of the dialog shown, once it has one: the path, or none when it was cancelled.
pub fn dialog_answer() -> Option<Option<String>> {
    DIALOG
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

// ---- a whole sound played (the films') ----

/// A whole sound (the films') playing through SDL (stopped when dropped, and SDL's audio given back:
/// each start takes it once).
pub struct PlayedSound {
    stream: *mut SDL_AudioStream,
    /// The bytes given to the stream.
    total_bytes: usize,
    /// The bytes one second of the sound takes.
    bytes_a_second: f64,
}

impl PlayedSound {
    /// `pcm` (16-bit, `channels` interleaved, at `hz`) made ready on the default device, paused
    /// until `resume`; None when SDL's audio or the stream cannot be had.
    pub fn start(pcm: &[i16], hz: u32, channels: usize) -> Option<Self> {
        // SAFETY: SDL's audio started; a spec SDL copies; the stream checked for null; `pcm`
        // copied into it whole (its bytes and their count).
        unsafe {
            if !SDL_InitSubSystem(SDL_INIT_AUDIO) {
                return None;
            }
            let spec = SDL_AudioSpec {
                format: SDL_AUDIO_S16LE,
                channels: channels as i32,
                freq: hz as i32,
            };
            let stream = SDL_OpenAudioDeviceStream(
                SDL_AUDIO_DEVICE_DEFAULT_PLAYBACK,
                &raw const spec,
                None,
                std::ptr::null_mut(),
            );
            if stream.is_null() {
                SDL_QuitSubSystem(SDL_INIT_AUDIO);
                return None;
            }
            SDL_PutAudioStreamData(stream, pcm.as_ptr().cast(), (pcm.len() * 2) as i32);
            SDL_FlushAudioStream(stream);
            Some(Self {
                stream,
                total_bytes: pcm.len() * 2,
                bytes_a_second: hz as f64 * channels as f64 * 2.0,
            })
        }
    }

    /// Starts (or goes on with) the sound.
    pub fn resume(&self) {
        // SAFETY: the stream `start` opened, alive until this value drops.
        unsafe {
            SDL_ResumeAudioStreamDevice(self.stream);
        }
    }

    /// How much of the sound the device has taken from the stream so far: the film's clock
    /// (0 until the device starts pulling).
    pub fn played(&self) -> std::time::Duration {
        // SAFETY: the stream `start` opened, alive until this value drops.
        let queued = unsafe { SDL_GetAudioStreamQueued(self.stream) }.max(0) as usize;
        let taken = self.total_bytes.saturating_sub(queued);
        std::time::Duration::from_secs_f64(taken as f64 / self.bytes_a_second)
    }
}

impl Drop for PlayedSound {
    fn drop(&mut self) {
        // SAFETY: the stream `start` opened, destroyed once; the audio it started, given
        // back once.
        unsafe {
            SDL_DestroyAudioStream(self.stream);
            SDL_QuitSubSystem(SDL_INIT_AUDIO);
        }
    }
}

/// A joystick event's content (`Event::joystick_input`).
#[derive(Clone, Copy, Debug)]
pub enum JoystickInput {
    /// A hat and its direction bits (1 up, 2 right, 4 down, 8 left).
    Hat(u8, u8),
    /// A button (counted from 0) and whether it went down.
    Button(u8, bool),
}
