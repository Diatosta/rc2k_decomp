//! `race::play_movie_loop` (0x4607A5): the viewer's replay played as a race: the skid
//! marks set up (`race::init_skidmarks`), "dpg replay", the block played from
//! (`dword_5C2530`) and the one it stops at (`dword_5C2538`) logged; the race's state 2
//! (`dword_5C5CE0`), the clock restarted (`scene::restart_clock`, `g_sceneClock` into
//! `dword_613840`); then each frame the input (`input::update`), a key pressed
//! (`g_keysPressed + 1`) ending it, the race's step (`drive::race_step`) and frame
//! (`drive::race_frame`), until the block played is the one to stop at; "dpg replay
//! exit".
//!
//! In:  nothing
//! Out: the registers as the steps leave them (eax `dword_5C2530` after a whole play); no
//!      flags read
//!
//! Caller: `race::play_movie`. Its calls through `part` (places in `CALLEES`; the logs 8
//! and 9); in the game their Rust versions, the frame through its hook (which gates each
//! frame, as for `drive::race_loop`). Ungated (it plays a replay). Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins (the frame moving the
//! block on, a key after some frames), against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::viewer::{BLOCK, PLAY_END, Part, gb, get, put, regs_entry};

const STATE: usize = 0x5C5CE0; // dword_5C5CE0
const SCENE_CLOCK: usize = 0x613830; // g_sceneClock
const CLOCK_BASE: usize = 0x613840; // dword_613840
const KEYS_PRESSED: usize = 0x4F6CDD; // g_keysPressed + 1
const TEXTS: [usize; 4] = [0x4DDF45, 0x4DDF50, 0x4DDF61, 0x4DDF72]; // "dpg replay" ..

/// The Rust version: the registers it leaves.
pub fn play_movie_loop(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(1, &mut r)?;
    let log = |part: Part, r: &Regs, place: u32, eax: u32| -> Result<(), Refused> {
        let mut s = Regs { eax, ..*r };
        part(place, &mut s)
    };
    log(part, &r, 2, a(TEXTS[0]))?;
    log(part, &r, 2, a(TEXTS[1]))?;
    log(part, &r, 3, get(BLOCK))?;
    log(part, &r, 2, a(TEXTS[2]))?;
    log(part, &r, 3, get(PLAY_END))?;
    put(STATE, 2);
    part(4, &mut r)?;
    r.eax = get(SCENE_CLOCK);
    put(CLOCK_BASE, r.eax);
    loop {
        part(5, &mut r)?;
        if gb(KEYS_PRESSED) != 0 {
            break;
        }
        part(6, &mut r)?;
        part(7, &mut r)?;
        r.eax = get(BLOCK);
        if r.eax == get(PLAY_END) {
            break;
        }
    }
    log(part, &r, 2, a(TEXTS[3]))?;
    Ok(r)
}

/// In the game: the Rust versions, the frame through its hook.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::drive::race_parts;
    unsafe {
        match place {
            1 => *r = crate::race::stage_prefs::init_skidmarks(r),
            2 | 8 => crate::logfile::log_write::log_write(r.eax as *const u8, place == 8, 1),
            3 => crate::logfile::log_int::log_int(r.eax as i32, true, 1),
            4 => race_parts::restart_clock(r),
            5 => race_parts::input_update(r),
            6 => crate::drive::race_step::race_step(r)?.apply(r),
            _ => race_parts::race_frame(r),
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| play_movie_loop(r, &mut game_part))
}
