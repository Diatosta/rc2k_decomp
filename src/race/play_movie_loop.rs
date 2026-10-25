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
//! Caller: `race::play_movie`.

use crate::errors::Refused;
use crate::input::store::DEVICES;
use crate::physics::flow::a;
use crate::race::viewer::{BLOCK, PLAY_END, get, put};

const STATE: usize = 0x5C5CE0; // dword_5C5CE0
const SCENE_CLOCK: usize = 0x613830; // g_sceneClock
const CLOCK_BASE: usize = 0x613840; // dword_613840
const TEXTS: [usize; 4] = [0x4DDF45, 0x4DDF50, 0x4DDF61, 0x4DDF72]; // "dpg replay" ..

fn log_text(text: usize, newline: bool) {
    unsafe { crate::logfile::log_write::log_write(a(text) as *const u8, newline, 1) }
}

fn log_number(n: u32) {
    crate::logfile::log_int::log_int(n as i32, true, 1)
}

/// The Rust version. Reads no register, and its one caller (`race::play_movie`, whose
/// caller reads none either) reads none it leaves (`lq -s 0x4607A5`: nothing live).
pub fn play_movie_loop() -> Result<(), Refused> {
    use crate::drive::race_parts;
    unsafe { crate::race::stage_prefs::init_skidmarks() };
    log_text(TEXTS[0], false);
    log_text(TEXTS[1], false);
    log_number(get(BLOCK));
    log_text(TEXTS[2], false);
    log_number(get(PLAY_END));
    put(STATE, 2);
    race_parts::restart_clock();
    put(CLOCK_BASE, get(SCENE_CLOCK));
    loop {
        race_parts::input_update();
        if DEVICES.keys_pressed.get(1) != 0 {
            break;
        }
        unsafe { crate::drive::race_step::race_step() }?;
        race_parts::race_frame()?;
        if get(BLOCK) == get(PLAY_END) {
            break;
        }
    }
    log_text(TEXTS[3], false);
    Ok(())
}
