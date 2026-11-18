//! The touch screen (phones and tablets): SDL's finger events into the controls of
//! `rc2k_core::input::touch`, the virtual gamepad the game reads in a race and the overlay drawn
//! over the frame. (The menus' fingers: `touch_menu`.)
//!
//! The virtual gamepad is attached at the start, so the game lists it with the other
//! controllers: its default profile (the left stick steers, the right trigger accelerates, the
//! left one brakes, the south button is a gear up, east a gear down, west the handbrake, north the
//! camera, start the pause) is what the touch layout presses. A real gamepad plugged in hides the
//! overlay.

use crate::platform::ffi::{sdl, sdl_input};
use rc2k_core::input::menu_touch::Phase;
use rc2k_core::input::touch::{Button, Controls, Layout, Output};
use sdl3_sys::everything::{
    SDL_GAMEPAD_AXIS_LEFT_TRIGGER, SDL_GAMEPAD_AXIS_LEFTX, SDL_GAMEPAD_AXIS_RIGHT_TRIGGER,
    SDL_GAMEPAD_BUTTON_EAST, SDL_GAMEPAD_BUTTON_NORTH, SDL_GAMEPAD_BUTTON_SOUTH,
    SDL_GAMEPAD_BUTTON_START, SDL_GAMEPAD_BUTTON_WEST, SDL_JoystickID,
};
use std::sync::Mutex;

/// The virtual gamepad's USB ids (pid.codes' vendor, an id of this program's own).
const VENDOR: u16 = 0x1209;
const PRODUCT: u16 = 0x7C02;

struct State {
    pad: Option<(SDL_JoystickID, sdl_input::Joystick)>,
    controls: Option<Controls>,
    /// What the pad last got.
    sent: Output,
    /// Whether the controls (not the gestures) are the live mode.
    driving: bool,
    /// Real gamepads present.
    real_pads: u32,
}

static STATE: Mutex<State> = Mutex::new(State {
    pad: None,
    controls: None,
    sent: Output {
        steer: 0.0,
        held: [false; rc2k_core::input::touch::BUTTONS],
    },
    driving: false,
    real_pads: 0,
});

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The name the virtual gamepad is listed with.
const PAD_NAME: &str = "Touch screen";

/// Where the touch screen's pad is among the controllers listed.
pub fn index_in(list: &[rc2k_core::input::device_id::Listed]) -> Option<usize> {
    list.iter().position(|l| l.name == PAD_NAME)
}

/// The virtual gamepad plugged in, before the game lists its controllers.
pub fn attach_at_start() {
    let mut st = state();
    if st.pad.is_some() {
        return;
    }
    let attached = sdl_input::attach_virtual_gamepad(VENDOR, PRODUCT, c"Touch screen")
        .and_then(|id| sdl_input::Joystick::open(id).map(|joystick| (id, joystick)));
    crate::logfile::program_log::log(&format!(
        "touch: the virtual gamepad {}",
        if attached.is_some() {
            "is attached"
        } else {
            "could not be attached"
        }
    ));
    st.pad = attached;
}

/// SDL has found a joystick: a real one (not the virtual gamepad) hides the overlay.
pub fn pad_added(id: SDL_JoystickID) {
    let mut st = state();
    if st.pad.is_some_and(|(ours, _)| ours == id) {
        return;
    }
    if sdl_input::is_gamepad(id) {
        st.real_pads += 1;
    }
}

/// A joystick has gone.
pub fn pad_removed(id: SDL_JoystickID) {
    let mut st = state();
    if st.pad.is_some_and(|(ours, _)| ours == id) {
        return;
    }
    st.real_pads = st.real_pads.saturating_sub(1);
}

/// A finger's change in a race (`x`, `y`: parts of the window): it drives the controls.
pub fn finger(window: sdl::Window, id: u64, phase: Phase, x: f32, y: f32, now: u64) {
    let Some(screen) = crate::platform::touch_menu::screen_of(window) else {
        return;
    };
    let (x, y) = (x * screen.width, y * screen.height);
    let mut st = state();
    change_mode(&mut st, true);
    let controls = st.controls.get_or_insert_with(|| Controls::new(&screen));
    controls.resize(&screen);
    controls.finger(id, phase, x, y, now);
}

fn change_mode(st: &mut State, driving: bool) {
    if st.driving != driving {
        st.driving = driving;
        if let Some(controls) = st.controls.as_mut() {
            controls.release_all();
        }
    }
}

/// The controls' state given to the virtual gamepad; once a turn of the pump.
pub fn update(driving: bool, now: u64) {
    let mut st = state();
    change_mode(&mut st, driving);
    let out = match (&st.controls, driving) {
        (Some(controls), true) => controls.output(now),
        _ => Output::default(),
    };
    if out == st.sent {
        return;
    }
    let Some((_, pad)) = st.pad else {
        return;
    };
    // A virtual gamepad's trigger rests at the axis's minimum (0 is half pressed).
    let level = |held: bool| if held { i16::MAX } else { i16::MIN };
    if out.steer != st.sent.steer {
        pad.set_virtual_axis(
            SDL_GAMEPAD_AXIS_LEFTX.0,
            (out.steer * f32::from(i16::MAX)) as i16,
        );
    }
    for (button, was, now_held) in Button::ALL
        .into_iter()
        .map(|b| (b, st.sent.is_held(b), out.is_held(b)))
    {
        if was == now_held {
            continue;
        }
        match button {
            Button::Throttle => {
                pad.set_virtual_axis(SDL_GAMEPAD_AXIS_RIGHT_TRIGGER.0, level(now_held));
            }
            Button::Brake => {
                pad.set_virtual_axis(SDL_GAMEPAD_AXIS_LEFT_TRIGGER.0, level(now_held));
            }
            Button::Handbrake => {
                pad.set_virtual_button(SDL_GAMEPAD_BUTTON_WEST.0, now_held);
            }
            Button::GearUp => {
                pad.set_virtual_button(SDL_GAMEPAD_BUTTON_SOUTH.0, now_held);
            }
            Button::GearDown => {
                pad.set_virtual_button(SDL_GAMEPAD_BUTTON_EAST.0, now_held);
            }
            Button::Camera => {
                pad.set_virtual_button(SDL_GAMEPAD_BUTTON_NORTH.0, now_held);
            }
            Button::Pause => {
                pad.set_virtual_button(SDL_GAMEPAD_BUTTON_START.0, now_held);
            }
        }
    }
    st.sent = out;
}

/// The race's controls over the frame.
pub fn draw(renderer: sdl::Renderer, window: sdl::Window, now: u64) {
    let st = state();
    if st.real_pads > 0 || st.pad.is_none() {
        return;
    }
    let Some(screen) = crate::platform::touch_menu::screen_of(window) else {
        return;
    };
    let layout = Layout::new(&screen);
    let out = st
        .controls
        .as_ref()
        .map_or_else(Output::default, |c| c.output(now));
    for button in Button::ALL {
        let r = layout.rect(button);
        let held = out.is_held(button);
        let alpha = if held { 120 } else { 45 };
        renderer.fill_blended([255, 255, 255, alpha], [r.x, r.y, r.w, r.h]);
        let label = button.label();
        let scale = (r.h * 0.26 / 8.0).min(r.w * 0.8 / (8.0 * label.len() as f32));
        let (tw, th) = (8.0 * scale * label.len() as f32, 8.0 * scale);
        renderer.text_blended(
            [255, 255, 255, if held { 240 } else { 170 }],
            r.x + (r.w - tw) / 2.0,
            r.y + (r.h - th) / 2.0,
            scale,
            label,
        );
    }
    // The wheel: a track with the finger's mark, or a hint of where to put the finger.
    let wheel = layout.wheel;
    match st.controls.as_ref().and_then(Controls::wheel_position) {
        Some((anchor, y, steer)) => {
            let thick = layout.reach * 0.12;
            renderer.fill_blended(
                [255, 255, 255, 60],
                [
                    anchor - layout.reach,
                    y - thick / 2.0,
                    2.0 * layout.reach,
                    thick,
                ],
            );
            let knob = layout.reach * 0.45;
            let kx = anchor + steer * layout.reach;
            renderer.fill_blended(
                [255, 255, 255, 150],
                [kx - knob / 2.0, y - knob / 2.0, knob, knob],
            );
        }
        None => {
            let scale = wheel.h * 0.12 / 8.0;
            renderer.text_blended(
                [255, 255, 255, 70],
                wheel.x + wheel.w * 0.18,
                wheel.y + wheel.h * 0.55,
                scale,
                "<  STEER  >",
            );
        }
    }
}
