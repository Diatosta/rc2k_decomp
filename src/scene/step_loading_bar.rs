//! `scene::step_loading_bar` (0x446786): moves the loading's progress on and shows it.
//!
//! In:  eax = how many of the system textures are done
//! Out: every register preserved
//!
//! Unless there are no system textures (`dword_5DC974`): the progress `dword_623700`
//! = the low end `dword_623634` plus eax times (the high end `dword_623638` less the
//! low) over the textures' number, plus 1. Then the function `dword_623704` names is
//! called (the front end's set-up puts `sub_4467C7` there, which draws).
//! One caller, `scene::reload_system_textures`.
//! Refuses no function to call.
//! Shadow mode runs the original alone and checks the progress it left.
//! Self-test: the original itself, the function it calls being a variable that the
//! test points at a stand-in of its own; 30 cases.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{Regs, game_read, game_write, rva};

// ---- game data ----

/// `dword_623634` the low end, `dword_623638` the high end.
const ENDS_RVA: usize = rva(0x623634);
const PROGRESS_RVA: usize = rva(0x623700); // dword_623700
const SHOW_RVA: usize = rva(0x623704); // dword_623704
const SYSTEM_TEXTURES_RVA: usize = rva(0x5DC974); // dword_5DC974

// ---- Rust version ----

fn ends() -> rc2k_core::scene::loading::BarEnds {
    let [low, high]: [u32; 2] = unsafe { game_read(ENDS_RVA) };
    rc2k_core::scene::loading::BarEnds { low, high }
}

/// The progress, or None where it is left as it is. Converted:
/// `rc2k_core::scene::loading::BarEnds::progress`.
unsafe fn progress(done: u32) -> Option<u32> {
    let textures: u32 = unsafe { game_read(SYSTEM_TEXTURES_RVA) };
    ends().progress(done, textures)
}

/// `show`: what stands for the function `dword_623704` names, entered with the
/// registers given.
pub unsafe fn step_loading_bar(r: &Regs, show: &mut dyn FnMut(&mut Regs)) -> Result<(), BadInput> {
    unsafe {
        if game_read::<u32>(SHOW_RVA) == 0 {
            return Err(BadInput("no function to show the loading with"));
        }
        let mut inner = *r;
        inner.ebp = ends().span();
        if let Some(progress) = progress(r.eax) {
            game_write(PROGRESS_RVA, progress);
            inner.eax = progress.wrapping_sub(1);
            inner.edx = 0;
            inner.ebx = game_read(SYSTEM_TEXTURES_RVA);
        }
        inner.esi = game_read(SHOW_RVA);
        show(&mut inner);
        Ok(())
    }
}
