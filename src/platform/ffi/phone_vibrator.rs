//! The phone's own vibration motor: on Android SDL's "VIBRATOR_SERVICE" haptic device (the
//! manifest asks for the VIBRATE permission); on iOS UIKit's impact feedback generator, called
//! through the Objective-C runtime on the main thread (SDL has no haptic device for the phone
//! there). When the motor is not found every call does nothing.

#![allow(unsafe_code)]

#[cfg(target_os = "android")]
mod backend {
    use sdl3_sys::everything::*;
    use std::sync::Mutex;

    /// The opened device (null: none) and whether it has been looked for.
    struct Motor(*mut SDL_Haptic, bool);
    // SAFETY: the handle is only used while the lock is held.
    unsafe impl Send for Motor {}
    static MOTOR: Mutex<Motor> = Mutex::new(Motor(std::ptr::null_mut(), false));

    /// The phone's vibrator with its rumble effect set up, or null.
    fn open() -> *mut SDL_Haptic {
        let mut count = 0;
        // SAFETY: SDL's array of `count` ids (null checked), copied and freed here; the name of
        // an id SDL listed is SDL's string, read at once; a handle from `SDL_OpenHaptic` is
        // checked before use.
        unsafe {
            let ids = SDL_GetHaptics(&raw mut count);
            if ids.is_null() {
                return std::ptr::null_mut();
            }
            let listed = std::slice::from_raw_parts(ids, count.max(0) as usize).to_vec();
            SDL_free(ids.cast());
            for id in listed {
                let name = SDL_GetHapticNameForID(id);
                if name.is_null()
                    || std::ffi::CStr::from_ptr(name).to_bytes() != b"VIBRATOR_SERVICE"
                {
                    continue;
                }
                let haptic = SDL_OpenHaptic(id);
                if haptic.is_null() {
                    continue;
                }
                if SDL_InitHapticRumble(haptic) {
                    return haptic;
                }
                SDL_CloseHaptic(haptic);
            }
        }
        std::ptr::null_mut()
    }

    pub fn pulse(strength: f32, ms: u32) {
        let mut motor = MOTOR
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !motor.1 {
            motor.0 = open();
            motor.1 = true;
            crate::logfile::program_log::log(&format!(
                "platform: the phone's vibrator {}",
                if motor.0.is_null() {
                    "was not found"
                } else {
                    "is open"
                }
            ));
        }
        if !motor.0.is_null() {
            // SAFETY: an open device with its rumble effect set up, held under the lock.
            unsafe { SDL_PlayHapticRumble(motor.0, strength, ms) };
        }
    }

    pub fn stop() {
        let motor = MOTOR
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !motor.0.is_null() {
            // SAFETY: an open device held under the lock.
            unsafe { SDL_StopHapticRumble(motor.0) };
        }
    }
}

#[cfg(target_os = "ios")]
mod backend {
    use std::ffi::{c_char, c_void};

    type Id = *mut c_void;
    type Sel = *mut c_void;

    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn class_respondsToSelector(class: Id, sel: Sel) -> bool;
        fn objc_msgSend();
    }

    unsafe extern "C" {
        static _dispatch_main_q: c_void;
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
    }

    /// `UIImpactFeedbackStyleHeavy`: the strongest; the intensity softens it.
    const STYLE_HEAVY: isize = 2;

    thread_local! {
        /// The generator, made on the main thread when first needed (null: none to be had) and
        /// whether it has been looked for.
        static GENERATOR: std::cell::Cell<(Id, bool)> =
            const { std::cell::Cell::new((std::ptr::null_mut(), false)) };
    }

    /// Runs on the main thread (where UIKit's feedback generators are used): one impact of
    /// the intensity whose bits `context` holds.
    extern "C" fn impact(context: *mut c_void) {
        let intensity = f32::from_bits(context as usize as u32);
        // SAFETY: the Objective-C runtime's calls with the types of the methods named: the
        // class and selectors are looked up by name, the generator's method is checked with
        // `class_respondsToSelector` before it is sent, and `objc_msgSend` is cast to the
        // signature of each message (the arguments are passed in registers as typed here).
        unsafe {
            let class = objc_getClass(c"UIImpactFeedbackGenerator".as_ptr());
            let (mut generator, looked) = GENERATOR.with(std::cell::Cell::get);
            if !looked {
                if !class.is_null() {
                    let send_alloc: unsafe extern "C" fn(Id, Sel) -> Id =
                        std::mem::transmute::<unsafe extern "C" fn(), _>(objc_msgSend);
                    let send_init: unsafe extern "C" fn(Id, Sel, isize) -> Id =
                        std::mem::transmute::<unsafe extern "C" fn(), _>(objc_msgSend);
                    let allocated = send_alloc(class, sel_registerName(c"alloc".as_ptr()));
                    generator = send_init(
                        allocated,
                        sel_registerName(c"initWithStyle:".as_ptr()),
                        STYLE_HEAVY,
                    );
                }
                GENERATOR.with(|g| g.set((generator, true)));
                crate::logfile::program_log::log(&format!(
                    "platform: the phone's impact feedback {}",
                    if generator.is_null() {
                        "was not found"
                    } else {
                        "is ready"
                    }
                ));
            }
            if generator.is_null() {
                return;
            }
            let with_intensity = sel_registerName(c"impactOccurredWithIntensity:".as_ptr());
            if class_respondsToSelector(class, with_intensity) {
                let send: unsafe extern "C" fn(Id, Sel, f64) =
                    std::mem::transmute::<unsafe extern "C" fn(), _>(objc_msgSend);
                send(generator, with_intensity, f64::from(intensity));
            } else {
                let send: unsafe extern "C" fn(Id, Sel) =
                    std::mem::transmute::<unsafe extern "C" fn(), _>(objc_msgSend);
                send(generator, sel_registerName(c"impactOccurred".as_ptr()));
            }
        }
    }

    /// One impact as strong as `strength` (0 .. 1) says; its length is the system's.
    pub fn pulse(strength: f32, _ms: u32) {
        // SAFETY: the main queue is a constant of libdispatch; the context is the intensity's
        // bits, read back by `impact`, which takes nothing else from it.
        unsafe {
            dispatch_async_f(
                &raw const _dispatch_main_q,
                strength.clamp(0.0, 1.0).to_bits() as usize as *mut c_void,
                impact,
            );
        }
    }

    /// An impact is over by itself.
    pub fn stop() {}
}

pub use backend::{pulse, stop};
