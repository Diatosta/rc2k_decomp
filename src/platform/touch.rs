//! The touch screen (phones and tablets): SDL's finger events into the controls of
//! `rc2k_core::input::touch`, the virtual gamepad the game reads in a race and the overlay drawn
//! over the frame. (The menus' fingers: `touch_menu`.)
//!
//! The virtual gamepad is attached at the start, so the game lists it with the other
//! controllers: its default profile (the left stick steers, the right trigger accelerates, the
//! left one brakes, the south button is a gear up, west a gear down, east the handbrake, north the
//! camera, back the lights, start the pause) is what the touch layout presses. A real gamepad plugged in hides the
//! overlay.

use crate::platform::ffi::{sdl, sdl_input};
use rc2k_core::input::menu_touch::Phase;
use rc2k_core::input::touch::PAD_NAME;
use rc2k_core::input::touch::{
    Answer, AnswerTaps, Asked, Button, Controls, Layout, Output, Source,
};
use sdl3_sys::everything::{SDL_GAMEPAD_AXIS_LEFTX, SDL_JoystickID};
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
    /// What player 1 drives with (`rc2k_core::input::touch::Source`).
    source: Source,
    /// The name of the real gamepad last used, to find it among the controllers listed.
    used_name: Option<String>,
    /// Set when `source` changed, until `take_source_change`.
    source_changed: bool,
    /// The finger on an answer button of the pause or the question.
    answers: AnswerTaps,
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
    source: Source::Auto,
    used_name: None,
    source_changed: false,
    answers: AnswerTaps::new(),
});

fn state() -> std::sync::MutexGuard<'static, State> {
    STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Where the touch screen's pad is among the controllers listed.
fn index_in(list: &[rc2k_core::input::device_id::Listed]) -> Option<usize> {
    list.iter().position(|l| l.name == PAD_NAME)
}

/// The controller number player 1 drives with (see `rc2k_core::input::touch::
/// player_one_controller`), given the one the player has and the controllers listed.
pub fn player_one_controller(list: &[rc2k_core::input::device_id::Listed], current: u32) -> u32 {
    let st = state();
    let touch = index_in(list).map(|at| at as u32 + 1);
    // The real controller: the one last used when it is listed, else the first that is not
    // the touch screen's.
    let pad = st
        .used_name
        .as_ref()
        .and_then(|name| {
            list.iter()
                .position(|l| &l.name == name && l.name != PAD_NAME)
        })
        .or_else(|| list.iter().position(|l| l.name != PAD_NAME))
        .map(|at| at as u32 + 1);
    rc2k_core::input::touch::player_one_controller(st.source, current, touch, pad)
}

/// Whether the player's controller number is the touch screen's pad (a name of it).
pub fn is_touch_pad(list: &[rc2k_core::input::device_id::Listed], number: u32) -> bool {
    number
        .checked_sub(1)
        .is_some_and(|at| index_in(list) == Some(at as usize))
}

/// Player 1's source set; the change noted for `take_source_change`.
fn set_source(st: &mut State, source: Source) {
    if st.source != source {
        crate::program_log::log(&format!(
            "touch: player 1 drives with {}",
            match source {
                Source::Auto => "the touch screen (nothing used yet)",
                Source::Touch => "the touch screen",
                Source::Pad => "a controller",
            }
        ));
        st.source = source;
        st.source_changed = true;
        // Whatever the other side held is let go.
        if let Some(controls) = st.controls.as_mut() {
            controls.release_all();
        }
    }
}

/// Whether player 1's source changed since the last call.
pub fn take_source_change() -> bool {
    std::mem::take(&mut state().source_changed)
}

/// A real gamepad's button or stick was used (`id` 0: a stand-in's event, which has none): it
/// drives player 1 from now on. The virtual gamepad's own events are not that.
pub(super) fn pad_used(id: SDL_JoystickID) {
    let mut st = state();
    if st.pad.as_ref().is_some_and(|(ours, _)| *ours == id) {
        return;
    }
    if id.0 != 0 {
        st.used_name = Some(sdl_input::joystick_name_for_id(id, ""));
    }
    set_source(&mut st, Source::Pad);
}

/// The virtual gamepad plugged in, before the game lists its controllers.
pub(super) fn attach_at_start() {
    let mut st = state();
    if st.pad.is_some() {
        return;
    }
    let attached = sdl_input::attach_virtual_gamepad(VENDOR, PRODUCT, c"Touch screen")
        .and_then(|id| sdl_input::Joystick::open(id).map(|joystick| (id, joystick)));
    crate::program_log::log(&format!(
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
pub(super) fn pad_added(id: SDL_JoystickID) {
    let mut st = state();
    if st.pad.as_ref().is_some_and(|(ours, _)| *ours == id) {
        return;
    }
    if sdl_input::is_gamepad(id) {
        st.real_pads += 1;
    }
}

/// A joystick has gone.
pub(super) fn pad_removed(id: SDL_JoystickID) {
    let mut st = state();
    if st.pad.as_ref().is_some_and(|(ours, _)| *ours == id) {
        return;
    }
    st.real_pads = st.real_pads.saturating_sub(1);
    if st.real_pads == 0 && st.source == Source::Pad {
        set_source(&mut st, Source::Touch);
    }
}

/// A finger's change in a race (`x`, `y`: parts of the window): it drives the controls.
pub fn finger(window: sdl::Window, id: u64, phase: Phase, x: f32, y: f32, now: u64) {
    let Some(screen) = crate::platform::safe_area::screen_of(window) else {
        return;
    };
    let (x, y) = (x * screen.width, y * screen.height);
    let mut st = state();
    set_source(&mut st, Source::Touch);
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
    let out = match (&st.controls, driving && st.source != Source::Pad) {
        (Some(controls), true) => controls.output(now),
        _ => Output::default(),
    };
    if out == st.sent {
        return;
    }
    let Some((_, pad)) = st.pad.as_ref() else {
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
        match crate::platform::input::touch_pad_button(button) {
            Ok(pressed) => {
                pad.set_virtual_button(pressed.0, now_held);
            }
            Err(axis) => {
                pad.set_virtual_axis(axis.0, level(now_held));
            }
        }
    }
    st.sent = out;
}

/// The race's controls over the frame.
pub fn draw(renderer: sdl::Renderer, window: sdl::Window, now: u64) {
    let st = state();
    if st.source == Source::Pad || st.pad.is_none() {
        return;
    }
    let Some(screen) = crate::platform::safe_area::screen_of(window) else {
        return;
    };
    let layout = Layout::new(&screen);
    let out = st
        .controls
        .as_ref()
        .map_or_else(Output::default, |c| c.output(now));
    for (button, r) in layout.places() {
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

/// A finger's change while the race waits for an answer: the answer it gives, if it is one.
pub fn answer_finger(
    window: sdl::Window,
    asked: Asked,
    id: u64,
    phase: Phase,
    x: f32,
    y: f32,
) -> Option<Answer> {
    let screen = crate::platform::safe_area::screen_of(window)?;
    state().answers.finger(
        &screen,
        asked,
        id,
        phase,
        x * screen.width,
        y * screen.height,
    )
}

/// A button's word: the game's letters as pixel runs, and the plain text for a renderer that
/// has no such picture.
pub struct Label {
    pub runs: Option<(Vec<crate::gfx::notice_text::Run>, i32, i32)>,
    pub plain: String,
}

/// The answer buttons over the frame, in the controls' style (white, see-through), the words
/// given by the caller; the one under a finger lit.
pub fn draw_answers(renderer: sdl::Renderer, window: sdl::Window, asked: Asked, labels: &[Label]) {
    let Some(screen) = crate::platform::safe_area::screen_of(window) else {
        return;
    };
    let pressed = state().answers.pressed();
    for ((answer, r), label) in rc2k_core::input::touch::answer_rects(&screen, asked)
        .into_iter()
        .zip(labels)
    {
        let held = pressed == Some(answer);
        renderer.fill_blended([0, 0, 0, 110], [r.x, r.y, r.w, r.h]);
        renderer.fill_blended(
            [255, 255, 255, if held { 130 } else { 55 }],
            [r.x, r.y, r.w, r.h],
        );
        match &label.runs {
            Some((runs, pw, ph)) => {
                // The game's letters, each pixel as a rectangle of whole screen pixels.
                let scale = (r.w * 0.85 / *pw as f32).min(r.h * 0.5 / *ph as f32);
                let (ox, oy) = (
                    r.x + (r.w - *pw as f32 * scale) / 2.0,
                    r.y + (r.h - *ph as f32 * scale) / 2.0,
                );
                for run in runs {
                    let (x0, x1) = (
                        (run.x as f32 * scale).round(),
                        ((run.x + run.len) as f32 * scale).round(),
                    );
                    let (y0, y1) = (
                        (run.y as f32 * scale).round(),
                        ((run.y + 1) as f32 * scale).round(),
                    );
                    let [red, green, blue] = run.rgb;
                    renderer.fill_blended(
                        [red, green, blue, 255],
                        [ox + x0, oy + y0, x1 - x0, y1 - y0],
                    );
                }
            }
            None => {
                let chars = label.plain.chars().count().max(1) as f32;
                let scale = (r.h * 0.3 / 8.0).min(r.w * 0.85 / (8.0 * chars));
                let (tw, th) = (8.0 * scale * chars, 8.0 * scale);
                renderer.text_blended(
                    [255, 255, 255, 250],
                    r.x + (r.w - tw) / 2.0,
                    r.y + (r.h - th) / 2.0,
                    scale,
                    &label.plain,
                );
            }
        }
    }
}
