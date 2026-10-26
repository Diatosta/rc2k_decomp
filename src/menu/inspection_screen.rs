//! `menu::inspection_screen` (0x4A1BBF): the inspection's screen shown: `dword_623FB0` /
//! `dword_623FB4` 0 and `dword_624080` 0x100, the menu scene prepared
//! (`scene::prepare_menu_scene`), the clip's screen edges 0 and 0x320 (`dword_6241DC` /
//! `dword_6241E0` and, as floats, `g_clipScreenLeft` / `Right`), the background loaded
//! (`service::load_background`, 0x470F55; its carry not read), copied to the surface and
//! the frame shown.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`. Its four calls through an `Env` (the
//! copy and the showing are the platform's; the others are reached through their
//! addresses, Rust where hooked). Shadow mode runs the original alone (it shows a frame).
//! Self-test: a copy whose calls go to `menu::frame_test`'s stand-ins, against the Rust
//! version on the same: the calls and the variables compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_write, rva};
use crate::menu::plain_frame::{COPY_PICTURE, SHOW_FRAME};
use crate::menu::screen_loop::{Env, Real};

const PREPARE: usize = rva(0x46F7FD); // scene::prepare_menu_scene
const LOAD_BACKGROUND: usize = rva(0x470F55); // service::load_background
const ZEROED_RVA: [usize; 2] = [rva(0x623FB0), rva(0x623FB4)]; // dword_623FB0, dword_623FB4
const SCALE_RVA: usize = rva(0x624080); // dword_624080
const EDGES_RVA: [usize; 2] = [rva(0x6241DC), rva(0x6241E0)]; // dword_6241DC, dword_6241E0
const CLIP_RVA: [usize; 2] = [rva(0x6241EC), rva(0x6241F0)]; // g_clipScreenLeft, g_clipScreenRight

/// The Rust version (see the header).
pub unsafe fn inspection_screen(entry: &Regs, env: &mut dyn Env) {
    let mut g = *entry;
    unsafe {
        game_write(ZEROED_RVA[0], 0u32);
        game_write(ZEROED_RVA[1], 0u32);
        game_write(SCALE_RVA, 0x100u32);
        env.call(PREPARE, &mut g);
        game_write(EDGES_RVA[0], 0u32);
        game_write(EDGES_RVA[1], 0x320u32);
        game_write(CLIP_RVA[0], 0.0f32.to_bits());
        game_write(CLIP_RVA[1], 800.0f32.to_bits());
        for function in [LOAD_BACKGROUND, COPY_PICTURE, SHOW_FRAME] {
            env.call(function, &mut g);
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { inspection_screen(r, &mut Real) };
    Ok(())
}
