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

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};

// ---- game data ----

/// `dword_623634` the low end, `dword_623638` the high end.
const ENDS_RVA: usize = rva(0x623634);
const PROGRESS_RVA: usize = rva(0x623700); // dword_623700
const SHOW_RVA: usize = rva(0x623704); // dword_623704
const BAR_RVA: usize = rva(0x4467C7); // race::loading_bar_frame
const NOTHING_RVA: usize = rva(0x4467C6); // nullsub_8
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

/// The Rust version: `done` of the system textures are done. The function
/// `dword_623704` names is one of the two the loading puts there: `race::loading_bar_frame`
/// (0x4467C7) or the empty function after it (0x4467C6); they read no register, so none of
/// the original's (ebp the span, eax / edx / ebx the progress, esi the function) is passed.
pub unsafe fn step_loading_bar(done: u32) -> Result<(), Refused> {
    unsafe {
        let show: u32 = game_read(SHOW_RVA);
        if show == 0 {
            return Err(BadInput("no function to show the loading with").into());
        }
        if let Some(progress) = progress(done) {
            game_write(PROGRESS_RVA, progress);
        }
        if show == game_addr(BAR_RVA) as u32 {
            crate::race::loading_bar_frame::loading_bar_frame()
        } else if show == game_addr(NOTHING_RVA) as u32 {
            Ok(())
        } else {
            Err(BadInput("a loading bar function that is neither of the loading's").into())
        }
    }
}
