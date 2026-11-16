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

/// Row n's text (without its NUL). The logic is in `rc2k_core::screens::controls::binding_text`.
pub fn binding_text(
    o: crate::owners::Owners<'static>,
    controls: &crate::screens::controls_store::ControlsScreen,
    player: crate::menu::standings::PlayerCell,
    n: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<Vec<u8>, Refused> {
    let b = rc2k_core::screens::controls::Binding {
        binding: player.get().binding(n as usize),
        kind: player.get().sensitivity_level,
        axis: if n < 2 {
            player.get().axis_codes[3 - n as usize]
        } else {
            0
        },
    };
    let mut canvas = crate::text::draw_text::GameCanvas::new(gfx);
    let mut s = ControlsScreen::new(&mut canvas, o);
    Ok(rc2k_core::screens::controls::binding_text(
        &mut s,
        &b,
        n as usize,
        &direction_names(controls),
    )?)
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
