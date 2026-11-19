//! SDL3's camera (`sdl3-sys`), for reading a QR code shown on another screen. Only where the
//! SDL build has a camera subsystem (Linux on x86-64, Windows, iOS, Android); elsewhere there are no cameras.
//!
//! The handle is an opaque newtype made only by `open` and closed once by its `Drop`; it is
//! used on the thread that opened it.

#![allow(unsafe_code)]

/// A frame of the camera as 8-bit RGBA, row after row.
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
    /// The camera faces the person using the machine (a phone's front camera, a laptop's
    /// webcam): people expect its picture mirrored, as in a mirror.
    pub facing_user: bool,
}

/// What the person using the machine has said of the camera's use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    #[cfg_attr(
        not(any(
            windows,
            target_os = "ios",
            target_os = "android",
            web,
            all(target_os = "linux", not(target_arch = "aarch64"))
        )),
        allow(
            dead_code,
            reason = "only a real camera is ever still waiting for an answer"
        )
    )]
    Pending,
    #[cfg_attr(
        handheld,
        allow(dead_code, reason = "the handheld has no camera to approve")
    )]
    Approved,
    Denied,
}

#[cfg(any(
    windows,
    target_os = "ios",
    target_os = "android",
    web,
    all(target_os = "linux", not(target_arch = "aarch64"))
))]
mod real {
    use super::{Frame, Permission};
    use rc2k_core::gfx::camera_frame::{Layout, frame_len, rotate_rgba, to_rgba};
    use sdl3_sys::everything::*;

    /// The camera subsystem started (it is left started); false when SDL has none.
    pub fn init() -> bool {
        // SAFETY: a plain flag; SDL itself is started by then.
        unsafe { SDL_InitSubSystem(SDL_INIT_CAMERA) }
    }

    /// The ids of the cameras SDL sees now, the back-facing ones first.
    pub fn devices() -> Vec<u32> {
        let mut count = 0;
        // SAFETY: SDL's array of `count` ids (null checked), copied and freed here; the position
        // query takes an id SDL has just listed.
        unsafe {
            let ids = SDL_GetCameras(&raw mut count);
            if ids.is_null() {
                return Vec::new();
            }
            let copied = std::slice::from_raw_parts(ids, count.max(0) as usize).to_vec();
            SDL_free(ids.cast());
            let mut ids: Vec<u32> = copied.into_iter().map(|id| id.0).collect();
            // A stable sort: cameras of one position stay in SDL's order.
            ids.sort_by_key(|&id| offer_rank(id));
            ids
        }
    }

    /// An open camera, and whether it faces the person using the machine.
    pub struct Camera(*mut SDL_Camera, bool);

    /// The camera's name as the system gives it.
    pub fn name(id: u32) -> String {
        // SAFETY: SDL's string for the id (null when it is not a camera), owned by SDL and
        // copied here at once.
        unsafe {
            let name = SDL_GetCameraName(SDL_CameraID(id));
            if name.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(name)
                    .to_string_lossy()
                    .into_owned()
            }
        }
    }

    /// Whether camera `id` faces the person using the machine.
    fn faces_user(id: u32) -> bool {
        // SAFETY: a plain query of an id from `devices`.
        unsafe { SDL_GetCameraPosition(SDL_CameraID(id)) == SDL_CAMERA_POSITION_FRONT_FACING }
    }

    /// Where a camera ranks in the order they are offered: the back-facing ones first (the
    /// ones that can be held up to a screen), then those of unknown position, the
    /// user-facing ones last.
    fn offer_rank(id: u32) -> u32 {
        // SAFETY: a plain query of an id from `devices`.
        match unsafe { SDL_GetCameraPosition(SDL_CameraID(id)) } {
            SDL_CAMERA_POSITION_BACK_FACING => 0,
            SDL_CAMERA_POSITION_FRONT_FACING => 2,
            _ => 1,
        }
    }

    /// How well a camera format suits (lower is better): the layouts `camera_frame` reads
    /// itself first, the ones SDL converts after, the compressed one last (SDL decodes it, and
    /// the frames have been seen garbled that way).
    fn rank(format: SDL_PixelFormat) -> u32 {
        if layout_of(format).is_some() {
            0
        } else if format == SDL_PIXELFORMAT_MJPG {
            2
        } else {
            1
        }
    }

    /// The layout of a pixel format `camera_frame` reads.
    fn layout_of(format: SDL_PixelFormat) -> Option<Layout> {
        let packed = |r, g, b| Some(Layout::Packed32 { r, g, b });
        match format {
            SDL_PIXELFORMAT_RGBA32 | SDL_PIXELFORMAT_RGBX32 => packed(0, 1, 2),
            SDL_PIXELFORMAT_BGRA32 | SDL_PIXELFORMAT_BGRX32 => packed(2, 1, 0),
            SDL_PIXELFORMAT_ARGB32 | SDL_PIXELFORMAT_XRGB32 => packed(1, 2, 3),
            SDL_PIXELFORMAT_ABGR32 | SDL_PIXELFORMAT_XBGR32 => packed(3, 2, 1),
            SDL_PIXELFORMAT_RGB24 => Some(Layout::Rgb24 { r: 0, g: 1, b: 2 }),
            SDL_PIXELFORMAT_BGR24 => Some(Layout::Rgb24 { r: 2, g: 1, b: 0 }),
            SDL_PIXELFORMAT_YUY2 => Some(Layout::Yuy2),
            SDL_PIXELFORMAT_YVYU => Some(Layout::Yvyu),
            SDL_PIXELFORMAT_UYVY => Some(Layout::Uyvy),
            SDL_PIXELFORMAT_NV12 => Some(Layout::Nv12),
            SDL_PIXELFORMAT_NV21 => Some(Layout::Nv21),
            SDL_PIXELFORMAT_IYUV => Some(Layout::I420),
            SDL_PIXELFORMAT_YV12 => Some(Layout::Yv12),
            _ => None,
        }
    }

    /// The size a QR code is read at: a code on another screen fills half the picture's height
    /// or less, and its modules need about four pixels each; the large codes (over a hundred
    /// modules a side) need a full-HD picture for that.
    const WANTED_AREA: i64 = 1920 * 1080;
    /// The fewest frames a second a size must give to be chosen over a smaller one.
    const SLOWEST: i64 = 10;

    /// The format the camera is opened with: of the ones it lists, the best-ranked, and in that
    /// rank the one nearest 1920 x 1080 among those that give at least `SLOWEST` frames a
    /// second (all of them when none do), with the most frames a second.
    fn best_spec(id: u32) -> Option<SDL_CameraSpec> {
        let mut count = 0;
        // SAFETY: SDL's array of `count` pointers to specs (null checked), each read once and
        // the array freed here.
        unsafe {
            let list = SDL_GetCameraSupportedFormats(SDL_CameraID(id), &raw mut count);
            if list.is_null() {
                return None;
            }
            let specs: Vec<SDL_CameraSpec> =
                std::slice::from_raw_parts(list, count.max(0) as usize)
                    .iter()
                    .filter(|p| !p.is_null())
                    .map(|&p| *p)
                    .collect();
            SDL_free(list.cast());
            specs.into_iter().min_by_key(|s| {
                let area = i64::from(s.width) * i64::from(s.height);
                let fps =
                    i64::from(s.framerate_numerator) / i64::from(s.framerate_denominator.max(1));
                (
                    rank(s.format),
                    fps < SLOWEST,
                    (area - WANTED_AREA).abs(),
                    -fps,
                )
            })
        }
    }

    /// Opens camera `id` in the best format it lists (its own choice when it lists none).
    pub fn open(id: u32) -> Option<Camera> {
        let spec = best_spec(id);
        let spec_ptr = spec.as_ref().map_or(std::ptr::null(), std::ptr::from_ref);
        // SAFETY: an id from `devices`; the spec is one the camera listed, or null (its own
        // choice).
        let raw = unsafe { SDL_OpenCamera(SDL_CameraID(id), spec_ptr) };
        (!raw.is_null()).then(|| Camera(raw, faces_user(id)))
    }

    impl Camera {
        pub fn permission(&self) -> Permission {
            // SAFETY: the handle is open until `self` is dropped.
            match unsafe { SDL_GetCameraPermissionState(self.0) } {
                SDL_CAMERA_PERMISSION_STATE_APPROVED => Permission::Approved,
                SDL_CAMERA_PERMISSION_STATE_DENIED => Permission::Denied,
                _ => Permission::Pending,
            }
        }

        /// The newest frame as RGBA, if the camera has one ready: converted from the frame's
        /// own format with its own row pitch, by `camera_frame` where it reads the layout and
        /// by SDL's pixel conversion otherwise.
        pub fn frame(&mut self) -> Option<Frame> {
            let mut stamp = 0;
            // SAFETY: the handle is open; the frame is SDL's until released below, and is
            // only read (copied out) in between.
            unsafe {
                let frame = SDL_AcquireCameraFrame(self.0, &raw mut stamp);
                if frame.is_null() {
                    return None;
                }
                let converted = convert(&*frame, self.1);
                SDL_ReleaseCameraFrame(self.0, frame);
                converted
            }
        }
    }

    /// A camera frame's surface as RGBA.
    ///
    /// # Safety
    /// `surface` is a frame SDL holds for the caller: `pixels` points at a frame of its
    /// format, `pitch` and height (the planes after the first laid out as SDL lays them).
    unsafe fn convert(surface: &SDL_Surface, facing_user: bool) -> Option<Frame> {
        let (w, h) = (surface.w.max(0) as usize, surface.h.max(0) as usize);
        let pitch = surface.pitch.max(0) as usize;
        if surface.pixels.is_null() || w == 0 || h == 0 {
            return None;
        }
        let mut rgba = if let Some(layout) = layout_of(surface.format) {
            let len = frame_len(layout, pitch, h)?;
            // SAFETY: a frame of this format, pitch and height is `len` bytes (the caller's
            // promise), alive for this call.
            let data = unsafe { std::slice::from_raw_parts(surface.pixels.cast::<u8>(), len) };
            to_rgba(layout, w, h, pitch, data)?
        } else {
            // SDL's own conversion, told the format and the pitch.
            let mut out = vec![0u8; w * h * 4];
            // SAFETY: the source is the frame's pixels as SDL laid them out; the destination
            // is `w * h * 4` bytes at a pitch of `w * 4`.
            let done = unsafe {
                SDL_ConvertPixels(
                    surface.w,
                    surface.h,
                    surface.format,
                    surface.pixels,
                    surface.pitch,
                    SDL_PIXELFORMAT_RGBA32,
                    out.as_mut_ptr().cast(),
                    (w * 4) as i32,
                )
            };
            if !done {
                return None;
            }
            out
        };
        // The camera API says how far the frame is meant to be turned clockwise to stand
        // upright (it follows the phone's and the screen's orientation). Only a front camera is
        // turned: its picture was seen upside down while the back camera's was right, so the
        // back camera's frames are left as they have always been.
        let (mut w, mut h) = (w, h);
        if facing_user {
            // SAFETY: the surface is live; the properties id is SDL's, 0 when it has none.
            let degrees = unsafe {
                SDL_GetFloatProperty(
                    SDL_GetSurfaceProperties(std::ptr::from_ref(surface).cast_mut()),
                    SDL_PROP_SURFACE_ROTATION_FLOAT,
                    0.0,
                )
            };
            if degrees != 0.0 {
                (rgba, w, h) = rotate_rgba(&rgba, w, h, degrees);
            }
        }
        Some(Frame {
            width: w,
            height: h,
            rgba,
            facing_user,
        })
    }

    impl Drop for Camera {
        fn drop(&mut self) {
            // SAFETY: opened by `open`, closed only here.
            unsafe { SDL_CloseCamera(self.0) };
        }
    }
}

#[cfg(not(any(
    windows,
    target_os = "ios",
    target_os = "android",
    web,
    all(target_os = "linux", not(target_arch = "aarch64"))
)))]
mod real {
    use super::{Frame, Permission};

    pub fn init() -> bool {
        false
    }

    pub fn devices() -> Vec<u32> {
        Vec::new()
    }

    pub struct Camera;

    pub fn name(_: u32) -> String {
        String::new()
    }

    pub fn open(_: u32) -> Option<Camera> {
        None
    }

    impl Camera {
        pub fn permission(&self) -> Permission {
            Permission::Denied
        }

        pub fn frame(&mut self) -> Option<Frame> {
            None
        }
    }
}

pub use real::{Camera, devices, init, name, open};
