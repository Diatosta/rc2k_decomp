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
use crate::framework::{game_read, rva};
use crate::menu::player::player_at;
use crate::screens::game_screen::ControlsScreen;
use crate::service::state::PLAYER_INDEX_RVA;
use crate::text::draw_text::Canvas;

// byte_A8BC54
const LEARNING_RVA: usize = rva(0x7551F4); // dword_7551F4
const ROW_RVA: usize = rva(0x7551FC); // dword_7551FC
const PAD_NAMES_RVA: usize = rva(0x755200); // word_755200
const STICK_NAMES_RVA: usize = rva(0x755212); // word_755212

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The directions' names (`word_755200`, `word_755212`).
unsafe fn direction_names() -> rc2k_core::screens::controls::DirectionNames {
    unsafe {
        rc2k_core::screens::controls::DirectionNames {
            pad: game_read(PAD_NAMES_RVA),
            stick: game_read(STICK_NAMES_RVA),
        }
    }
}

/// Row n's text (without its NUL). Converted: `rc2k_core::screens::controls::binding_text`.
pub unsafe fn binding_text(player: u32, n: u32) -> Result<Vec<u8>, Refused> {
    unsafe {
        let b = rc2k_core::screens::controls::Binding {
            binding: get(player + 0x50 + 4 * n),
            kind: get(player + 0x34),
            axis: if n < 2 {
                *((player + 0x3B - n) as *const u8)
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
            &direction_names(),
        )?)
    }
}

/// Returns the carry.
pub unsafe fn draw_control_bindings(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    unsafe {
        let player = player_at(game_read(PLAYER_INDEX_RVA));
        if player == 0 {
            return Err(BadInput("no current player").into());
        }
        let learning: u32 = game_read(LEARNING_RVA);
        let row: u32 = game_read(ROW_RVA);
        let mut texts: [Vec<u8>; 10] = Default::default();
        for (n, text) in texts.iter_mut().enumerate() {
            *text = binding_text(player, n as u32)?;
        }
        // Converted: `rc2k_core::screens::controls::control_bindings`.
        let learnt = (learning == 1 && (1..=10).contains(&row)).then(|| row as usize - 1);
        rc2k_core::screens::controls::control_bindings(
            &mut ControlsScreen::new(canvas),
            &texts,
            learnt,
        )
    }
}
