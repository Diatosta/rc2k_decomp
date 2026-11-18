//! SDL3's camera (`sdl3-sys`), for reading a QR code shown on another screen. Only where the
//! SDL build has a camera subsystem (Linux on x86-64, Windows, Android); elsewhere there are no cameras.
//!
//! The handle is an opaque newtype made only by `open` and closed once by its `Drop`; it is
//! used on the thread that opened it.

#![allow(unsafe_code)]

/// A frame of the camera as 8-bit RGBA, row after row.
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

/// What the person using the machine has said of the camera's use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Permission {
    #[cfg_attr(
        not(any(
            windows,
            target_os = "android",
            all(target_os = "linux", not(target_arch = "aarch64"))
        )),
        allow(
            dead_code,
            reason = "only a real camera is ever still waiting for an answer"
        )
    )]
    Pending,
    Approved,
    Denied,
}

#[cfg(any(
    windows,
    target_os = "android",
    all(target_os = "linux", not(target_arch = "aarch64"))
))]
mod real {
    use super::{Frame, Permission};
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
            let mut copied = std::slice::from_raw_parts(ids, count.max(0) as usize).to_vec();
            SDL_free(ids.cast());
            // The camera facing away from the person first (a phone's back camera reads a
            // code on another screen); the sort is stable, so the others keep SDL's order.
            copied.sort_by_key(|&id| SDL_GetCameraPosition(id) != SDL_CAMERA_POSITION_BACK_FACING);
            copied.into_iter().map(|id| id.0).collect()
        }
    }

    /// An open camera.
    pub struct Camera(*mut SDL_Camera);

    /// Opens camera `id` in the format it prefers.
    pub fn open(id: u32) -> Option<Camera> {
        // SAFETY: an id from `devices`; a null spec asks for the camera's own choice.
        let raw = unsafe { SDL_OpenCamera(SDL_CameraID(id), std::ptr::null()) };
        (!raw.is_null()).then_some(Camera(raw))
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

        /// The newest frame, if the camera has one ready.
        pub fn frame(&mut self) -> Option<Frame> {
            let mut stamp = 0;
            // SAFETY: the handle is open; the frame is SDL's until released below, and is
            // only read (converted into a surface of ours) in between.
            unsafe {
                let frame = SDL_AcquireCameraFrame(self.0, &raw mut stamp);
                if frame.is_null() {
                    return None;
                }
                let converted = SDL_ConvertSurface(frame, SDL_PIXELFORMAT_RGBA32);
                SDL_ReleaseCameraFrame(self.0, frame);
                if converted.is_null() {
                    return None;
                }
                let (w, h, pitch) = (
                    (*converted).w.max(0) as usize,
                    (*converted).h.max(0) as usize,
                    (*converted).pitch.max(0) as usize,
                );
                let pixels = (*converted).pixels.cast::<u8>();
                let mut rgba = Vec::with_capacity(w * h * 4);
                if !pixels.is_null() && pitch >= w * 4 {
                    for row in 0..h {
                        // SAFETY (row): the converted surface holds `h` rows of `pitch` bytes,
                        // each with `w` pixels of 4 bytes.
                        rgba.extend_from_slice(std::slice::from_raw_parts(
                            pixels.add(row * pitch),
                            w * 4,
                        ));
                    }
                }
                SDL_DestroySurface(converted);
                (rgba.len() == w * h * 4 && w > 0).then_some(Frame {
                    width: w,
                    height: h,
                    rgba,
                })
            }
        }
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
    target_os = "android",
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

pub use real::{Camera, devices, init, open};
