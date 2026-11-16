//! What each of the current player's ten controls is bound to, on the controller pages: "no
//! binding" (string 0x3BD), or the direction's name, for a device of kind 3 on the first two rows
//! the axis, and for a button the button's number. The row being learnt is left out. Refuses no
//! current player and a direction past the two 9-entry name tables.

use crate::errors::{BadInput, Refused};
use crate::screens::game_screen::ControlsScreen;
use crate::text::draw_text::Canvas;

/// The directions' names (`ControlsScreen.stick_names[0]`).
fn direction_names(
    controls: &crate::screens::controls_store::ControlsScreen,
) -> rc2k_core::screens::controls::DirectionNames {
    rc2k_core::screens::controls::DirectionNames {
        pad: controls.pad(),
        stick: controls.stick(),
    }
}

/// Row n's text (without its NUL): "no binding" (string 0x3BD), a button's name (string 0x3C0)
/// and number, a hat direction's name, or an axis as its short name, "(+x)" with its side
/// (none for a centred axis, the steering).
pub fn binding_text(
    o: crate::owners::Owners<'static>,
    controls: &crate::screens::controls_store::ControlsScreen,
    player: crate::menu::standings::PlayerCell,
    n: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<Vec<u8>, Refused> {
    use rc2k_core::input::profile::{HatDirection, Row, Shown, shown};
    let current = o.menus.screens.event.current_player.get();
    let controller = player.get().controller;
    let row = Row::of_control(n + 1).ok_or(BadInput("a binding row past the controls"))?;
    let (profile_shown, kind) = if controller == 0 {
        (Shown::Nothing, rc2k_core::input::profile::Kind::Joystick)
    } else {
        crate::input::profiles::with_profile(o.input, current, controller, |p| {
            let kind = if p.wheel {
                rc2k_core::input::profile::Kind::Wheel
            } else {
                rc2k_core::input::profile::Kind::Joystick
            };
            (shown(p, row), kind)
        })
        .unwrap_or((Shown::Nothing, rc2k_core::input::profile::Kind::Joystick))
    };
    let pad =
        crate::input::profiles::controller_kind(o.input, controller).map_or(kind, |(kind, _)| kind);
    let mut canvas = crate::text::draw_text::GameCanvas::new(gfx);
    let mut s = ControlsScreen::new(&mut canvas, o);
    use rc2k_core::screens::screen::Screen;
    Ok(match profile_shown {
        Shown::Nothing => s.string_bytes(0x3BD),
        Shown::Button(number) => {
            let mut out = s.string_bytes(0x3C0);
            out.push(b' ');
            out.extend(rc2k_core::strings::format::decimal(
                number,
                rc2k_core::strings::format::Pad::None,
            ));
            out
        }
        Shown::Hat(direction) => {
            let names = direction_names(controls).pad;
            let code = match direction {
                HatDirection::Up => 1,
                HatDirection::Down => 2,
                HatDirection::Left => 4,
                HatDirection::Right => 8,
            };
            s.string_bytes(names[code] as u32)
        }
        Shown::Axis(axis, side) => {
            let mut out = b"(".to_vec();
            match side {
                Some(true) => out.push(b'+'),
                Some(false) => out.push(b'-'),
                None => {}
            }
            out.extend(rc2k_core::input::profile::axis_label(pad, axis).bytes());
            out.push(b')');
            out
        }
    })
}

/// Returns whether the last text drawn was too wide for its box.
pub fn draw_control_bindings(
    o: crate::owners::Owners<'static>,
    canvas: &mut dyn Canvas,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<bool, Refused> {
    let menus = o.menus;
    let controls = &menus.screens.controls;
    let Some(player) = crate::menu::player::player(menus, menus.screens.event.current_player.get())
    else {
        return Err(BadInput("no current player").into());
    };
    let learning: u32 = controls.learning.get();
    let row: u32 = controls.learned.get();
    let mut texts: [Vec<u8>; 10] = Default::default();
    for (n, text) in texts.iter_mut().enumerate() {
        *text = binding_text(o, controls, player, n as u32, gfx)?;
    }
    // The logic is in `rc2k_core::screens::controls::control_bindings`.
    let learnt = (learning == 1 && (1..=10).contains(&row)).then(|| row as usize - 1);
    rc2k_core::screens::controls::control_bindings(
        &mut ControlsScreen::new(canvas, o),
        &texts,
        learnt,
    )
}
