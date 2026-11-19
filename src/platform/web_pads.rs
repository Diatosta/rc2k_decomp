#![allow(unsafe_code)]
//! The browser's gamepads for the game. SDL's own Emscripten joystick driver reads the page's
//! Gamepad API from the thread that started it, and the game runs on a worker, which has no
//! `navigator.getGamepads`: its connect event would end in an exception there. So that driver's
//! event handlers are removed at the start (`stop_sdl_driver`), and each pump of the events
//! samples the pads on the page's thread (`emscripten_sample_gamepad_data` and the calls that
//! read it run there, proxied) and copies them into SDL virtual joysticks (`ffi::sdl_input::
//! VirtualPad`): a pad with the browser's standard layout becomes a virtual gamepad of SDL's
//! layout (so the menus, the icons and the game's controller code treat it as any gamepad), any
//! other a plain joystick of its raw axes and buttons. A pad the browser lists appears at its
//! first button press (a browser rule), and goes with its disconnection. Rumble is not carried
//! over.

use crate::platform::ffi::sdl_input::VirtualPad;
use crate::program_log::log;
use std::ffi::{CStr, c_char, c_void};
use std::sync::Mutex;

/// The browser's record of a pad (`EmscriptenGamepadEvent`).
#[repr(C)]
struct PadState {
    timestamp: f64,
    axes_count: i32,
    buttons_count: i32,
    axis: [f64; 64],
    analog_button: [f64; 64],
    digital_button: [bool; 64],
    connected: bool,
    index: i32,
    id: [c_char; 64],
    mapping: [c_char; 64],
}

unsafe extern "C" {
    fn emscripten_sample_gamepad_data() -> i32;
    fn emscripten_get_num_gamepads() -> i32;
    fn emscripten_get_gamepad_status(index: i32, state: *mut PadState) -> i32;
    fn emscripten_set_gamepadconnected_callback_on_thread(
        user_data: *mut c_void,
        use_capture: bool,
        callback: Option<unsafe extern "C" fn(i32, *const PadState, *mut c_void) -> bool>,
        thread: usize,
    ) -> i32;
    fn emscripten_set_gamepaddisconnected_callback_on_thread(
        user_data: *mut c_void,
        use_capture: bool,
        callback: Option<unsafe extern "C" fn(i32, *const PadState, *mut c_void) -> bool>,
        thread: usize,
    ) -> i32;
}

/// `EMSCRIPTEN_RESULT_SUCCESS`.
const SUCCESS: i32 = 0;
/// `EM_CALLBACK_THREAD_CONTEXT_CALLING_THREAD`: the thread SDL registered its handlers from.
const CALLING_THREAD: usize = 2;
/// The pads the page lists at most (the Gamepad API's four).
const SLOTS: usize = 4;

/// SDL's buttons of a virtual gamepad, in its own order, by the browser's standard layout's button
/// numbers (south, east, west, north, back, guide, start, the two sticks, the two shoulders, the
/// d-pad's up, down, left and right).
const STANDARD_BUTTONS: [usize; 15] = [0, 1, 2, 3, 8, 16, 9, 10, 11, 4, 5, 12, 13, 14, 15];

/// A pad of the page copied into SDL: its place in the page's list and what SDL was given.
struct Copy {
    index: i32,
    pad: VirtualPad,
    /// How many of the values are axes (the rest are buttons).
    axes: usize,
    /// The last values given (the axes, then the buttons), so only changes are passed on.
    given: Vec<i32>,
}

static COPIES: Mutex<Vec<Copy>> = Mutex::new(Vec::new());

/// SDL's own browser driver no longer told of pads: its handlers are removed (it called them on
/// the thread that started it, where the Gamepad API is not there).
pub fn stop_sdl_driver() {
    // SAFETY: the calls remove the handlers SDL set (a null callback removes them) on the page's
    // window; no data is passed.
    let (connected, disconnected) = unsafe {
        (
            emscripten_set_gamepadconnected_callback_on_thread(
                std::ptr::null_mut(),
                false,
                None,
                CALLING_THREAD,
            ),
            emscripten_set_gamepaddisconnected_callback_on_thread(
                std::ptr::null_mut(),
                false,
                None,
                CALLING_THREAD,
            ),
        )
    };
    log(&format!(
        "web: SDL's own gamepad handlers removed ({connected}, {disconnected}); the pads are copied from the page"
    ));
}

/// The text of a record's C string field.
fn text(field: &[c_char; 64]) -> String {
    // SAFETY: the page's text is written NUL-ended within the 64 bytes (the record starts
    // zeroed and the copy keeps a NUL last).
    unsafe { CStr::from_ptr(field.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// `Vendor: 045e Product: 028e` of Chrome's pad names (Firefox's `046d-c216-Name` too). The page
/// gives the name cut to 64 bytes: an id cut short is 0.
fn usb_ids(id: &str) -> (u16, u16) {
    let after = |key: &str| {
        id.find(key)
            .and_then(|at| id.get(at + key.len()..at + key.len() + 4))
            .and_then(|hex| u16::from_str_radix(hex, 16).ok())
    };
    if after("Vendor: ").is_some() || after("Product: ").is_some() {
        return (
            after("Vendor: ").unwrap_or(0),
            after("Product: ").unwrap_or(0),
        );
    }
    let mut parts = id.split('-');
    match (parts.next(), parts.next()) {
        (Some(vendor), Some(product)) => (
            u16::from_str_radix(vendor, 16).unwrap_or(0),
            u16::from_str_radix(product, 16).unwrap_or(0),
        ),
        _ => (0, 0),
    }
}

/// The pad's name as the player knows it: the browser's text without its parenthesised ids.
fn name_of(id: &str) -> String {
    let name = id.split(" (").next().unwrap_or(id).trim();
    if name.is_empty() {
        "Browser gamepad".to_string()
    } else {
        name.to_string()
    }
}

/// An axis value (-1 to 1) as SDL's 16 bits.
fn axis16(value: f64) -> i16 {
    (value.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// A trigger's value (0 to 1) as SDL's axis, whose rest is its minimum.
fn trigger16(value: f64) -> i16 {
    (value.clamp(0.0, 1.0) * 65535.0 - 32768.0).round() as i16
}

/// The values a pad gives SDL: its axes, then its buttons (0 or 1).
fn values(state: &PadState, standard: bool) -> Vec<i32> {
    let axis = |n: usize| state.axis.get(n).copied().unwrap_or(0.0);
    let button = |n: usize| state.digital_button.get(n).copied().unwrap_or(false);
    if standard {
        let mut out = vec![
            i32::from(axis16(axis(0))),
            i32::from(axis16(axis(1))),
            i32::from(axis16(axis(2))),
            i32::from(axis16(axis(3))),
            i32::from(trigger16(state.analog_button[6])),
            i32::from(trigger16(state.analog_button[7])),
        ];
        out.extend(STANDARD_BUTTONS.iter().map(|&n| i32::from(button(n))));
        out
    } else {
        let axes = (state.axes_count.clamp(0, 16)) as usize;
        let buttons = (state.buttons_count.clamp(0, 32)) as usize;
        let mut out: Vec<i32> = (0..axes).map(|n| i32::from(axis16(axis(n)))).collect();
        out.extend((0..buttons).map(|n| i32::from(button(n))));
        out
    }
}

/// The page's pads looked at and copied: new ones attached, gone ones detached, the values passed
/// on. Cheap enough for every pump of the events; a browser without the Gamepad API does nothing.
pub fn poll() {
    // SAFETY: the page's sampling call, proxied to the page's thread; it needs no arguments.
    if unsafe { emscripten_sample_gamepad_data() } != SUCCESS {
        return;
    }
    // SAFETY: reads the sample just taken.
    let listed = unsafe { emscripten_get_num_gamepads() }.clamp(0, SLOTS as i32);
    let mut copies = COPIES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut here = Vec::new();
    for index in 0..listed {
        // SAFETY: an all-zero record is valid (numbers, flags and NUL-ended text).
        let mut state: PadState = unsafe { std::mem::zeroed() };
        // SAFETY: `state` is a whole record the call fills for the pad at `index`.
        let got = unsafe { emscripten_get_gamepad_status(index, &raw mut state) };
        if got != SUCCESS || !state.connected {
            continue;
        }
        here.push(index);
        let standard = text(&state.mapping) == "standard" && state.buttons_count >= 16;
        let wanted = values(&state, standard);
        if let Some(copy) = copies.iter_mut().find(|c| c.index == index) {
            pass_on(copy, &wanted);
            continue;
        }
        let id = text(&state.id);
        let (vendor, product) = usb_ids(&id);
        let name = std::ffi::CString::new(name_of(&id)).unwrap_or_default();
        let pad = if standard {
            VirtualPad::attach_gamepad(&name, vendor, product)
        } else {
            VirtualPad::attach_stick(
                &name,
                vendor,
                product,
                state.axes_count.clamp(0, 16) as u16,
                state.buttons_count.clamp(0, 32) as u16,
            )
        };
        match pad {
            Some(pad) => {
                log(&format!(
                    "web: the browser's gamepad {index} ({id}, {} mapping) copied into SDL as {}",
                    text(&state.mapping),
                    if standard { "a gamepad" } else { "a joystick" }
                ));
                let mut copy = Copy {
                    index,
                    pad,
                    axes: if standard {
                        6
                    } else {
                        state.axes_count.clamp(0, 16) as usize
                    },
                    given: Vec::new(),
                };
                pass_on(&mut copy, &wanted);
                copies.push(copy);
            }
            None => log(&format!(
                "web: SDL takes no virtual joystick for the browser's gamepad {index} ({id})"
            )),
        }
    }
    let mut kept = Vec::with_capacity(copies.len());
    for copy in copies.drain(..) {
        if here.contains(&copy.index) {
            kept.push(copy);
        } else {
            log(&format!(
                "web: the browser's gamepad {} is gone",
                copy.index
            ));
            copy.pad.detach();
        }
    }
    *copies = kept;
}

/// The values that changed handed to SDL's virtual joystick.
fn pass_on(copy: &mut Copy, wanted: &[i32]) {
    for (n, &value) in wanted.iter().enumerate() {
        if copy.given.get(n) == Some(&value) {
            continue;
        }
        if n < copy.axes {
            copy.pad.set_axis(n as i32, value as i16);
        } else {
            copy.pad.set_button((n - copy.axes) as i32, value != 0);
        }
    }
    copy.given = wanted.to_vec();
}
