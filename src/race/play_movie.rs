//! `race::play_movie` (0x460776): "shaunplayrendermovie - in" logged, the clock restarted
//! (`scene::restart_clock`, `g_sceneClock` into `dword_613840`), the replay played
//! (`race::play_movie_loop`), "shaunplayrendermovie - out" logged.
//!
//! In:  nothing
//! Out: the registers as the play leaves them; no flags read
//!
//! Caller: `race::play_replay_range`. Its calls through `part`: 1 `logfile::log_write_l1`,
//! 2 `scene::restart_clock`, 3 `play_movie_loop`, 4 `logfile::log_write_l1_nl`; in the
//! game their Rust versions. Ungated (it plays a replay). Shadow mode runs the original
//! alone. Self-test: a copy whose calls go to stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::viewer::{Part, get, put, regs_entry};

const SCENE_CLOCK: usize = 0x613830; // g_sceneClock
const CLOCK_BASE: usize = 0x613840; // dword_613840
const IN_TEXT: usize = 0x4DDF10; // "shaunplayrendermovie - in"
const OUT_TEXT: usize = 0x4DDF2A; // "shaunplayrendermovie - out"

/// The Rust version: the registers it leaves.
pub fn play_movie(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let mut s = Regs {
        eax: a(IN_TEXT),
        ..r
    };
    part(1, &mut s)?;
    part(2, &mut r)?;
    r.eax = get(SCENE_CLOCK);
    put(CLOCK_BASE, r.eax);
    part(3, &mut r)?;
    let mut s = Regs {
        eax: a(OUT_TEXT),
        ..r
    };
    part(4, &mut s)?;
    Ok(r)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 | 4 => unsafe { crate::logfile::log_write::log_write(r.eax as *const u8, place == 4, 1) },
        2 => crate::drive::race_parts::restart_clock(r),
        _ => {
            *r = crate::race::play_movie_loop::play_movie_loop(
                r,
                &mut crate::race::play_movie_loop::game_part,
            )?
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| play_movie(r, &mut game_part))
}
