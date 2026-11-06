//! `screens::draw_control_bindings` (0x46A829): what each of the current player's ten
//! controls is bound to, on the controller pages.
//!
//! In:  nothing
//! Out: every register preserved; carry of the last row's `draw_text` (clear when that row is
//! the one being learnt)
//!
//! The current player's (`dword_75598C`) bindings at +0x50 .. +0x74 (a dword each: dl a button, dh
//! a direction, 0 nothing), at x 0x1B8, y 0xD5 and on every 0x1B, in the font at 0x51E8C4, built in
//! `byte_A8BC54`: string 0x3BD for nothing; else dh's name (string `word_755200[dh]` for a device
//! kind, +0x34, below 2, else `word_755212[dh]`), for kind 3 on the first two rows the axis
//! (`strings::append_axis_name` of +0x3B, +0x3A), then for a button ('+' after a direction) string
//! 0x3C0, a space and the button's number. The row being learnt (`dword_7551F4` 1, `dword_7551FC`
//! the row from 1) is left out. In the draw lists of the four controller pages (0x758D9A ..).
//! Refuses (before anything) no current player and a direction past the two 9-entry tables (dh
//! above 8: the original reads past them). The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::menus;
use crate::screens::game_screen::ControlsScreen;
use crate::screens::owner::screens;
use crate::text::draw_text::Canvas;

// byte_A8BC54

/// The directions' names (`word_755200`, `word_755212`).
fn direction_names(
    controls: &crate::screens::controls_store::ControlsScreen,
) -> rc2k_core::screens::controls::DirectionNames {
    rc2k_core::screens::controls::DirectionNames {
        pad: controls.pad(),
        stick: controls.stick(),
    }
}

/// Row n's text (without its NUL). Converted: `rc2k_core::screens::controls::binding_text`.
pub fn binding_text(
    controls: &crate::screens::controls_store::ControlsScreen,
    player: crate::menu::standings::PlayerCell,
    n: u32,
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
    let mut canvas = crate::text::draw_text::GameCanvas;
    let mut s = ControlsScreen::new(&mut canvas);
    Ok(rc2k_core::screens::controls::binding_text(
        &mut s,
        &b,
        n as usize,
        &direction_names(controls),
    )?)
}

/// Returns the carry.
pub fn draw_control_bindings(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let controls = &screens().controls;
    let Some(player) = crate::menu::player::player(menus().screens.event.current_player.get())
    else {
        return Err(BadInput("no current player").into());
    };
    let learning: u32 = controls.learning.get();
    let row: u32 = controls.learned.get();
    let mut texts: [Vec<u8>; 10] = Default::default();
    for (n, text) in texts.iter_mut().enumerate() {
        *text = binding_text(controls, player, n as u32)?;
    }
    // Converted: `rc2k_core::screens::controls::control_bindings`.
    let learnt = (learning == 1 && (1..=10).contains(&row)).then(|| row as usize - 1);
    rc2k_core::screens::controls::control_bindings(&mut ControlsScreen::new(canvas), &texts, learnt)
}
