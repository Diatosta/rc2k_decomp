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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::viewer::{
    Part, Saved, answer, dispatch_parts, dword, get, put, regs_entry, run_both, test_part,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::play_movie", rva(0x460776), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| play_movie(r, &mut game_part));
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| play_movie(r, &mut game_part))
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; lea eax, aShaunplayrende
    let copy = match crate::race::viewer::copy(
        0x460776,
        0x2F,
        &[0x60, 0x8D, 0x05, 0x10, 0xDF, 0x4D, 0x00],
        &[0x7, 0xD, 0x1C, 0x28],
        &[(0x401427, 1), (0x450381, 2), (0x4607A5, 3), (0x40142B, 4)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions = [dword(CLOCK_BASE), dword(SCENE_CLOCK)];
    let saved = Saved::new(&regions);
    run_both(
        &mut t,
        "a play",
        copy.addr(),
        Regs::SENTINEL,
        &regions,
        0,
        &|| {
            put(SCENE_CLOCK, 0x4321);
            put(CLOCK_BASE, 0);
            answer(Box::new(|place, r| {
                if place == 3 {
                    r.eax = 0x55;
                    r.ebx = 0x66;
                }
            }));
        },
        &|r| play_movie(r, &mut test_part),
    );
    crate::race::viewer::quiet();
    drop(saved);
    t.finish()
}
