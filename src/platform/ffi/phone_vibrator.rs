//! The phone's own vibration motor: on Android SDL's "VIBRATOR_SERVICE" haptic device (the
//! manifest asks for the VIBRATE permission); on iOS Core Haptics' continuous events (UIKit's impact feedback
//! where the phone has no Core Haptics), called through the Objective-C runtime on the main
//! thread (SDL has no haptic device for the phone there). When the motor is not found every call does nothing.

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
    use std::ffi::{CStr, c_char, c_void};

    type Id = *mut c_void;
    type Sel = *mut c_void;

    #[link(name = "objc")]
    unsafe extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn class_respondsToSelector(class: Id, sel: Sel) -> bool;
        fn objc_msgSend();
    }

    // Core Haptics' NSString constants (iOS 13 and later, the build's deployment target).
    #[link(name = "CoreHaptics", kind = "framework")]
    unsafe extern "C" {
        static CHHapticEventTypeHapticContinuous: Id;
        static CHHapticEventParameterIDHapticIntensity: Id;
        static CHHapticEventParameterIDHapticSharpness: Id;
    }

    unsafe extern "C" {
        static _dispatch_main_q: c_void;
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
    }

    // A pulse's strength and length travel in one pointer-sized word.
    const _: () = assert!(usize::BITS >= 64);

    /// `UIImpactFeedbackStyleHeavy`: the strongest; the intensity softens it.
    const STYLE_HEAVY: isize = 2;

    /// How sharp a pulse feels (0 .. 1): low, so it is a rumble and not a click.
    const SHARPNESS: f32 = 0.2;

    /// `objc_msgSend` as a function of the type given (the signature of the message sent).
    macro_rules! send {
        ($t:ty) => {
            std::mem::transmute::<unsafe extern "C" fn(), $t>(objc_msgSend)
        };
    }

    /// What the main thread has made for the phone's motor.
    #[derive(Clone, Copy)]
    struct State {
        /// The haptic engine (null: none running).
        engine: Id,
        /// The impact generator (null: none, or not looked for).
        generator: Id,
        /// Whether the generator has been looked for.
        generator_looked: bool,
        /// Whether the hardware has been found not to do Core Haptics (no engine is tried again).
        unsupported: bool,
    }

    thread_local! {
        /// Touched only by the main thread, where `pulse_main` runs.
        static STATE: std::cell::Cell<State> = const {
            std::cell::Cell::new(State {
                engine: std::ptr::null_mut(),
                generator: std::ptr::null_mut(),
                generator_looked: false,
                unsupported: false,
            })
        };
    }

    fn sel(name: &CStr) -> Sel {
        // SAFETY: registering a selector name is always valid.
        unsafe { sel_registerName(name.as_ptr()) }
    }

    /// A started haptic engine, or null (and `unsupported` set when the hardware has none).
    ///
    /// # Safety
    /// Main thread only.
    unsafe fn make_engine(state: &mut State) -> Id {
        // SAFETY: Objective-C messages with the signatures of the methods named; classes and
        // objects are checked for null before they are sent to, and `err` is an out parameter
        // the methods fill (it is not read).
        unsafe {
            let class = objc_getClass(c"CHHapticEngine".as_ptr());
            if class.is_null() {
                state.unsupported = true;
                return std::ptr::null_mut();
            }
            let caps =
                send!(unsafe extern "C" fn(Id, Sel) -> Id)(class, sel(c"capabilitiesForHardware"));
            if caps.is_null()
                || !send!(unsafe extern "C" fn(Id, Sel) -> bool)(caps, sel(c"supportsHaptics"))
            {
                state.unsupported = true;
                return std::ptr::null_mut();
            }
            let mut err: Id = std::ptr::null_mut();
            let allocated = send!(unsafe extern "C" fn(Id, Sel) -> Id)(class, sel(c"alloc"));
            let engine = send!(unsafe extern "C" fn(Id, Sel, *mut Id) -> Id)(
                allocated,
                sel(c"initAndReturnError:"),
                &raw mut err,
            );
            if engine.is_null() {
                return std::ptr::null_mut();
            }
            // Haptics only: the engine leaves the game's audio session alone.
            send!(unsafe extern "C" fn(Id, Sel, bool))(engine, sel(c"setPlaysHapticsOnly:"), true);
            if !send!(unsafe extern "C" fn(Id, Sel, *mut Id) -> bool)(
                engine,
                sel(c"startAndReturnError:"),
                &raw mut err,
            ) {
                send!(unsafe extern "C" fn(Id, Sel))(engine, sel(c"release"));
                return std::ptr::null_mut();
            }
            engine
        }
    }

    /// One continuous haptic event of `strength` lasting `ms`, played on `engine` at once.
    /// False when any step failed.
    ///
    /// # Safety
    /// Main thread only; `engine` a live `CHHapticEngine`.
    unsafe fn play(engine: Id, strength: f32, ms: u32) -> bool {
        // SAFETY: Objective-C messages with the signatures of the methods named. The objects
        // made with alloc / init are released here once the pattern holds them; the array and
        // the player are autoreleased (the main run loop's pool frees them). `err` is an out
        // parameter that is not read. The Core Haptics constants are read only after the engine
        // class was found.
        unsafe {
            let mut err: Id = std::ptr::null_mut();
            let alloc = send!(unsafe extern "C" fn(Id, Sel) -> Id);
            let release = send!(unsafe extern "C" fn(Id, Sel));
            let param_class = objc_getClass(c"CHHapticEventParameter".as_ptr());
            let event_class = objc_getClass(c"CHHapticEvent".as_ptr());
            let pattern_class = objc_getClass(c"CHHapticPattern".as_ptr());
            let array_class = objc_getClass(c"NSArray".as_ptr());
            if param_class.is_null()
                || event_class.is_null()
                || pattern_class.is_null()
                || array_class.is_null()
            {
                return false;
            }
            let param_init = send!(unsafe extern "C" fn(Id, Sel, Id, f32) -> Id);
            let intensity = param_init(
                alloc(param_class, sel(c"alloc")),
                sel(c"initWithParameterID:value:"),
                CHHapticEventParameterIDHapticIntensity,
                strength.clamp(0.0, 1.0),
            );
            let sharpness = param_init(
                alloc(param_class, sel(c"alloc")),
                sel(c"initWithParameterID:value:"),
                CHHapticEventParameterIDHapticSharpness,
                SHARPNESS,
            );
            let array_of = send!(unsafe extern "C" fn(Id, Sel, *const Id, usize) -> Id);
            let array_selector = sel(c"arrayWithObjects:count:");
            let parameters = [intensity, sharpness];
            let event = send!(unsafe extern "C" fn(Id, Sel, Id, Id, f64, f64) -> Id)(
                alloc(event_class, sel(c"alloc")),
                sel(c"initWithEventType:parameters:relativeTime:duration:"),
                CHHapticEventTypeHapticContinuous,
                if intensity.is_null() || sharpness.is_null() {
                    std::ptr::null_mut()
                } else {
                    array_of(array_class, array_selector, parameters.as_ptr(), 2)
                },
                0.0,
                f64::from(ms) / 1000.0,
            );
            if !intensity.is_null() {
                release(intensity, sel(c"release"));
            }
            if !sharpness.is_null() {
                release(sharpness, sel(c"release"));
            }
            if event.is_null() {
                return false;
            }
            let events = [event];
            let pattern = send!(unsafe extern "C" fn(Id, Sel, Id, Id, *mut Id) -> Id)(
                alloc(pattern_class, sel(c"alloc")),
                sel(c"initWithEvents:parameters:error:"),
                array_of(array_class, array_selector, events.as_ptr(), 1),
                std::ptr::null_mut(),
                &raw mut err,
            );
            release(event, sel(c"release"));
            if pattern.is_null() {
                return false;
            }
            let player = send!(unsafe extern "C" fn(Id, Sel, Id, *mut Id) -> Id)(
                engine,
                sel(c"createPlayerWithPattern:error:"),
                pattern,
                &raw mut err,
            );
            release(pattern, sel(c"release"));
            if player.is_null() {
                return false;
            }
            // `CHHapticTimeImmediate` is 0.
            send!(unsafe extern "C" fn(Id, Sel, f64, *mut Id) -> bool)(
                player,
                sel(c"startAtTime:error:"),
                0.0,
                &raw mut err,
            )
        }
    }

    /// The pulse by Core Haptics: the engine made when first needed and again when a pulse
    /// fails (the system stops an engine now and then). False when it cannot be done.
    ///
    /// # Safety
    /// Main thread only.
    unsafe fn core_pulse(state: &mut State, strength: f32, ms: u32) -> bool {
        if state.unsupported {
            return false;
        }
        // SAFETY: `make_engine` and `play` are called from the main thread with an engine
        // `make_engine` returned; a failed engine is released once and not used again.
        unsafe {
            for _ in 0..2 {
                if state.engine.is_null() {
                    state.engine = make_engine(state);
                    if state.engine.is_null() {
                        return false;
                    }
                    crate::logfile::program_log::log("platform: the phone's haptic engine started");
                }
                if play(state.engine, strength, ms) {
                    return true;
                }
                send!(unsafe extern "C" fn(Id, Sel))(state.engine, sel(c"release"));
                state.engine = std::ptr::null_mut();
            }
        }
        false
    }

    /// The fallback: one impact of the intensity, its length the system's.
    ///
    /// # Safety
    /// Main thread only.
    unsafe fn impact(state: &mut State, intensity: f32) {
        // SAFETY: the Objective-C runtime's calls with the types of the methods named: the
        // class and selectors are looked up by name, the generator's method is checked with
        // `class_respondsToSelector` before it is sent, and `objc_msgSend` is cast to the
        // signature of each message (the arguments are passed in registers as typed here).
        unsafe {
            let class = objc_getClass(c"UIImpactFeedbackGenerator".as_ptr());
            if !state.generator_looked {
                if !class.is_null() {
                    let allocated =
                        send!(unsafe extern "C" fn(Id, Sel) -> Id)(class, sel(c"alloc"));
                    state.generator = send!(unsafe extern "C" fn(Id, Sel, isize) -> Id)(
                        allocated,
                        sel(c"initWithStyle:"),
                        STYLE_HEAVY,
                    );
                }
                state.generator_looked = true;
                crate::logfile::program_log::log(&format!(
                    "platform: the phone's impact feedback {}",
                    if state.generator.is_null() {
                        "was not found"
                    } else {
                        "is ready"
                    }
                ));
            }
            if state.generator.is_null() {
                return;
            }
            let with_intensity = sel(c"impactOccurredWithIntensity:");
            if class_respondsToSelector(class, with_intensity) {
                send!(unsafe extern "C" fn(Id, Sel, f64))(
                    state.generator,
                    with_intensity,
                    f64::from(intensity),
                );
            } else {
                send!(unsafe extern "C" fn(Id, Sel))(state.generator, sel(c"impactOccurred"));
            }
        }
    }

    /// Runs on the main thread (where UIKit's and Core Haptics' objects are used): one pulse
    /// whose strength (high half) and length in ms (low half) `context` holds.
    extern "C" fn pulse_main(context: *mut c_void) {
        let word = context as usize as u64;
        let strength = f32::from_bits((word >> 32) as u32);
        let ms = word as u32;
        let mut state = STATE.with(std::cell::Cell::get);
        // SAFETY: this runs on the main queue, the thread `core_pulse` and `impact` need.
        unsafe {
            if !core_pulse(&mut state, strength, ms) {
                impact(&mut state, strength);
            }
        }
        STATE.with(|s| s.set(state));
    }

    /// One pulse as strong as `strength` (0 .. 1) says, lasting `ms`: a continuous haptic event
    /// where the phone has Core Haptics, an impact of the strength (its length the system's)
    /// where it has not. Played on the main thread; this call does not wait for it.
    pub fn pulse(strength: f32, ms: u32) {
        let word = (u64::from(strength.clamp(0.0, 1.0).to_bits()) << 32) | u64::from(ms);
        // SAFETY: the main queue is a constant of libdispatch; the context is the pulse's
        // bits, read back by `pulse_main`, which takes nothing else from it.
        unsafe {
            dispatch_async_f(
                &raw const _dispatch_main_q,
                word as usize as *mut c_void,
                pulse_main,
            );
        }
    }

    /// A pulse is over by itself, at most a fraction of a second after the rumble stops.
    pub fn stop() {}
}

pub use backend::{pulse, stop};
