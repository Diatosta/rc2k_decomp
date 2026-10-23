//! `race::loadlevel_init` (0x44426F): the stage's loading proper ("LOADLEVEL_INIT"), its
//! 27 steps each logged, indented and marked on the loading bar. See `race/loadlevel.rs`.
//!
//! In:  nothing
//! Out: the registers as the last call (`scene::find_object_of_kind`) leaves them
//!
//! One caller, `race::load_stage` (0x4440B1). Its calls are the Rust versions but two,
//! which go through the game (`call_game`, section 15): init mark (`sub_456EA7`) and the
//! loading bar's callback (`sub_4467C7`, a `call esi` on `dword_623704`). Its caller reads no flags after it (`pushf` follows). Shadow mode
//! runs the original alone (it logs and loads the stage). Self-test: a copy whose 57
//! calls go to stand-ins that note them (the log and the loading bar too, the callback's
//! address patched), its 59 references to `g_logIndent`, `g_logEnabled`, `g_inRace` and
//! the callback the test's, against the Rust version on the same: editor or not, the
//! stages that flip the direction, the refresh flags, with and without `dword_64BB18`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::loadlevel::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { loadlevel_init(r, LoadVars::game(), &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
