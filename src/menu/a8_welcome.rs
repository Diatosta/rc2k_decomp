//! `menu::a8_welcome` (0x47F3C7): the A8 championship's welcome: the picture
//! `a8_welc.png` with the title bar (`gfx::load_picture_with_overlay`; not read: fatal
//! error 0x80712345, no message), converted; frames of it (`screens::run_frame` with
//! `menu::picture_frame`) until a key (`input::take_key`). Escape (1) asks about leaving
//! (`menu::confirm_leaving`): a no starts over. Then one more key taken (dropped).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `menu::run_a8_championship` (whose `g_eventMode` 0x13 picks it). The calls
//! through `event_flow::Call` (in the game `game_call`). The loop starts at +1: a private
//! copy (0x48 bytes, its calls and the jump to the fatal stub relocated) and a jump
//! written by hand. Ungated (it waits). Shadow mode runs the original alone. Self-test:
//! `event_flow::test_flow`, keys and answers by script.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::event_flow::*;
use crate::menu::flow_game::Game;
use rc2k_core::menu::welcome::{self};

pub unsafe fn a8_welcome(entry: &Regs, call: Call) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::welcome::a8_welcome`.
    welcome::a8_welcome(&mut Game::new(entry, call))?;
    Ok(*entry)
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { a8_welcome(r, &mut game_call) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
