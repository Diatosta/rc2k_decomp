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

// ---- handles ----

/// An SDL window.
#[derive(Clone, Copy)]
pub struct Window(pub(super) *mut SDL_Window);

/// A window's renderer.
#[derive(Clone, Copy)]
pub struct Renderer(pub(super) *mut SDL_Renderer);

/// A streaming texture of a renderer (its size kept for the upload's check).
#[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))] // the handheld shows frames on its framebuffer
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
    /// The window restored when it is minimised.
    pub fn restore_if_minimised(self) {
        // SAFETY: the window SDL made, still alive (the screen holds it).
        unsafe {
            if SDL_GetWindowFlags(self.0).0 & SDL_WINDOW_MINIMIZED.0 != 0 {
                SDL_RestoreWindow(self.0);
            }
        }
    }

    /// The window's size in the screen's pixels.
    pub fn set_size(self, width: usize, height: usize) {
        // SAFETY: as `restore_if_minimised`.
        unsafe { SDL_SetWindowSize(self.0, width as i32, height as i32) };
    }

    /// The refreshes a second of the display the window is on, if SDL knows them.
    pub fn display_refresh(self) -> Option<f64> {
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

    /// What was drawn put on the window.
    pub fn present(self) {
        // SAFETY: as `clear_black`.
        unsafe { SDL_RenderPresent(self.0) };
    }

    /// The texture drawn over the whole target.
    #[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))] // the handheld shows frames on its framebuffer
    pub fn draw_whole(self, texture: &Texture) {
        // SAFETY: the renderer and the texture made by it, both alive (null rectangles
        // mean the whole texture and the whole target).
        unsafe {
            SDL_RenderTexture(self.0, texture.raw, std::ptr::null(), std::ptr::null());
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
    #[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))] // the handheld shows frames on its framebuffer
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
    /// The whole texture replaced by `frame`'s rows of `pitch` bytes. False when SDL
    /// refuses, or when `frame` is shorter than the texture's rows.
    #[cfg_attr(all(target_os = "linux", target_arch = "aarch64"), allow(dead_code))] // the handheld shows frames on its framebuffer
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
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn use_dummy_video_driver() {
    // SAFETY: two constant C texts.
    unsafe { SDL_SetHint(SDL_HINT_VIDEO_DRIVER, c"dummy".as_ptr()) };
}

/// SDL's video started; false when it does not start.
pub fn init_video() -> bool {
    // SAFETY: SDL's own start-up, on the thread that will own the window.
    unsafe { SDL_Init(SDL_INIT_VIDEO) }
}

/// A window of the given size with a title.
pub fn create_window(title: &std::ffi::CStr, width: usize, height: usize) -> Option<Window> {
    // SAFETY: a NUL-ended title; plain sizes and no flags.
    let raw = unsafe {
        SDL_CreateWindow(
            title.as_ptr(),
            width as i32,
            height as i32,
            SDL_WindowFlags(0),
        )
    };
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

/// The window's own renderer (SDL's choice of driver).
pub fn create_renderer(window: Window) -> Option<Renderer> {
    // SAFETY: the window SDL made; a null name lets SDL choose.
    let raw = unsafe { SDL_CreateRenderer(window.0, std::ptr::null()) };
    (!raw.is_null()).then_some(Renderer(raw))
}

/// The window's renderer on the GPU device the 3D shares. `device` is the GPU module's
/// device; nothing in Rust dereferences it.
#[cfg(not(all(target_os = "linux", target_arch = "aarch64")))]
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

    /// The joystick id of a joystick added or removed event (else 0).
    pub fn joystick_id(&self) -> SDL_JoystickID {
        if self.kind() == SDL_EVENT_JOYSTICK_ADDED || self.kind() == SDL_EVENT_JOYSTICK_REMOVED {
            // SAFETY: the type says the `jdevice` record is the one SDL filled.
            unsafe { self.0.jdevice.which }
        } else {
            SDL_JoystickID(0)
        }
    }

    /// What a joystick event says (its joystick, then a hat's bits, a button going down or up,
    /// or an axis's value), else `None`. A gamepad's joystick sends these too.
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
            } else if kind == SDL_EVENT_JOYSTICK_AXIS_MOTION {
                let e = self.0.jaxis;
                Some((e.which, JoystickInput::Axis(e.axis, e.value)))
            } else {
                None
            }
        }
    }

    /// The button and the direction of a gamepad button event (else `None`).
    pub fn gamepad_button(&self) -> Option<(SDL_GamepadButton, bool)> {
        if self.kind() != SDL_EVENT_GAMEPAD_BUTTON_DOWN
            && self.kind() != SDL_EVENT_GAMEPAD_BUTTON_UP
        {
            return None;
        }
        // SAFETY: the type says the `gbutton` record is the one SDL filled.
        let button = unsafe { self.0.gbutton };
        Some((SDL_GamepadButton(button.button as i32), button.down))
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
    /// An axis and its value.
    Axis(u8, i16),
}
