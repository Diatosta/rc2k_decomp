//! The two stream commands (IDA, 2026-10-14): command 9, `audio::stream_command`
//! (0x40C194, `g_soundStream`), and command 0xB, `audio::music_command` (0x40C4DD,
//! `g_musicStream`); and `audio::stop_streams` (0x40C5E8). Actions (edx, unsigned): 0 stop;
//! 1 the thread's turn (feed while playing; stopped after it: stop, and for the music the
//! next tune); 3 the volume of what plays; any other plays (stop first if playing; nothing
//! for a volume <= 0, for `g_noMusic` (music), and a stop for no path or tune).
//!
//! Music tunes: the file is `g_musicPath` with the tune's two last digits at +0xE
//! (`strings::format_u32`, flags 0x12); tune 0xA is 10 + `g_tuneTurn` for the first turn
//! (counted on, 0 .. 5) whose file exists. When a tune stops, `g_nextTune` is played, and it
//! is never cleared there: the second tune repeats.

use crate::audio::stream::*;
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr};

/// The calls by the game's addresses, each with the registers it reads: 0 `start_stream`
/// (eax the path, esi; the carry), 1 `feed_stream`, 2 `stop_stream`, 3 `set_stream_volume`
/// (esi), 4 `files::file_exists` (eax; the carry: not there).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;
pub const START: u32 = 0;
pub const FEED: u32 = 1;
pub const STOP: u32 = 2;
pub const VOLUME: u32 = 3;
pub const EXISTS: u32 = 4;

/// The Rust versions, as the game has them.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        let s = stream(r.esi);
        match place {
            START => {
                let started = start_stream(s, r.eax as *const u8, &StartVars::game())?;
                r.set_carry(!started);
            }
            FEED => {
                let fed = feed_stream(
                    s,
                    game_addr(crate::audio::fill_buffer::LOCKED_PARTS_RVA) as u32,
                )?;
                r.set_carry(!fed);
            }
            STOP => stop_stream(s)?,
            VOLUME => set_stream_volume(s)?,
            _ => {
                let there = crate::files::file_exists::file_exists(r.eax as *const u8)?;
                r.set_carry(!there);
            }
        }
    }
    Ok(())
}

/// One call: carry out.
fn call(part: &mut Part, place: u32, eax: u32, player: u32) -> Result<bool, Refused> {
    let mut r = Regs {
        eax,
        esi: player,
        ..Regs::default()
    };
    part(place, &mut r)?;
    Ok(r.carry())
}

fn playing(player: u32) -> bool {
    unsafe { stream(player).playing == 1 }
}

/// `audio::stream_command` on the player at `player`.
pub fn stream_command(
    path: u32,
    volume: i32,
    action: u32,
    player: u32,
    mut part: Part,
) -> Result<(), Refused> {
    let part = &mut part;
    let stop = |part: &mut Part| call(part, STOP, 0, player).map(|_| ());
    match action {
        0 => return stop(part),
        1 => {
            if !playing(player) {
                return Ok(());
            }
        }
        3 => {
            if playing(player) {
                unsafe { stream(player).volume = volume as u32 };
                call(part, VOLUME, 0, player)?;
            }
            return Ok(());
        }
        _ => {
            if playing(player) {
                stop(part)?;
            }
            if volume <= 0 {
                return Ok(());
            }
            if path == 0 {
                return stop(part);
            }
            unsafe { stream(player).volume = volume as u32 };
            if call(part, START, path, player)? {
                return stop(part);
            }
        }
    }
    call(part, FEED, 0, player)?;
    if !playing(player) {
        stop(part)?;
    }
    Ok(())
}

/// Where `music_command` keeps its things.
#[derive(Clone, Copy)]
pub struct MusicVars {
    pub player: u32,
    pub next_tune: u32,
    pub no_music: u32,
    pub path: u32,
    pub turn: u32,
}

impl MusicVars {
    pub fn game() -> Self {
        let a = |r| game_addr(r) as u32;
        Self {
            player: a(MUSIC_STREAM_RVA),
            next_tune: a(NEXT_TUNE_RVA),
            no_music: a(NO_MUSIC_RVA),
            path: a(MUSIC_PATH_RVA),
            turn: a(TUNE_TURN_RVA),
        }
    }
}

/// How often a tune that stops at once is played again before the Rust version gives up
/// (the original plays it for ever: section 16).
const RETRIES: u32 = 0x100;

/// `audio::music_command`: `tune` with `next` to follow.
pub fn music_command(
    tune: u32,
    next: u32,
    volume: i32,
    action: u32,
    v: &MusicVars,
    mut part: Part,
) -> Result<(), Refused> {
    let part = &mut part;
    use crate::audio::voices::{read, write};
    let player = v.player;
    let stop = |part: &mut Part| call(part, STOP, 0, player).map(|_| ());
    let mut tune = tune;
    unsafe {
        let stop_and_forget = |part: &mut Part| -> Result<(), Refused> {
            stop(part)?;
            write(v.next_tune, 0);
            Ok(())
        };
        let mut feed_first = false;
        match action {
            0 => return stop_and_forget(part),
            1 => {
                if !playing(player) {
                    return Ok(());
                }
                feed_first = true;
            }
            3 => {
                if playing(player) {
                    stream(player).volume = volume as u32;
                    call(part, VOLUME, 0, player)?;
                }
                return Ok(());
            }
            _ => {
                if playing(player) {
                    stop(part)?;
                }
                if volume <= 0 || read(v.no_music) != 0 {
                    return Ok(());
                }
                if tune == 0 {
                    return stop_and_forget(part);
                }
                write(v.next_tune, next);
                stream(player).volume = volume as u32;
            }
        }
        let digits = |n: u32| {
            let d = crate::strings::format_u32::format_u32(n, 0x12);
            std::ptr::copy_nonoverlapping(d.as_ptr(), (v.path + 0xE) as *mut u8, d.len());
        };
        let mut plays = 0;
        loop {
            if !feed_first {
                plays += 1;
                if plays > RETRIES {
                    return Err(BadInput("a tune that stops at once, played again for ever").into());
                }
                if tune == 0xA {
                    let first = read(v.turn);
                    let mut looks = 0;
                    loop {
                        let n = 0xA + read(v.turn);
                        let mut turn = read(v.turn).wrapping_add(1);
                        if turn >= 6 {
                            turn = 0;
                        }
                        write(v.turn, turn);
                        digits(n);
                        if !call(part, EXISTS, v.path, player)? {
                            break;
                        }
                        looks += 1;
                        if first == read(v.turn) {
                            write(v.next_tune, 0);
                            return Ok(());
                        }
                        if looks > 6 {
                            return Err(BadInput(
                                "no tune file, with a tune turn past 5: the original looks for ever",
                            )
                            .into());
                        }
                    }
                } else {
                    digits(tune);
                }
                if call(part, START, v.path, player)? {
                    return stop_and_forget(part);
                }
            }
            feed_first = false;
            call(part, FEED, 0, player)?;
            if playing(player) {
                return Ok(());
            }
            stop(part)?;
            tune = read(v.next_tune);
            if tune == 0 {
                return stop_and_forget(part);
            }
        }
    }
}

// ---- for the self-tests: stand-ins for the five calls ----

use std::sync::Mutex;

/// What the stand-ins do: `start` fails with `start_fails`; the first `feed_stops` feeds
/// end the tune; a file exists for each tune whose bit (its two digits) is in `present`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Script {
    pub start_fails: bool,
    pub feed_stops: u32,
    pub present: u64,
    pub feeds: u32,
}

pub static SCRIPT: Mutex<Script> = Mutex::new(Script {
    start_fails: false,
    feed_stops: 0,
    present: 0,
    feeds: 0,
});
/// Each call: the place, eax where it is read (start, exists), esi.
pub static NOTED: Mutex<Vec<[u32; 3]>> = Mutex::new(Vec::new());

pub fn handler(place: u32, r: &mut Regs) {
    let eax = if place == START || place == EXISTS {
        r.eax
    } else {
        0
    };
    NOTED.lock().unwrap().push([place, eax, r.esi]);
    let mut s = SCRIPT.lock().unwrap();
    unsafe {
        let p = stream(r.esi);
        match place {
            START => {
                r.set_carry(s.start_fails);
                if !s.start_fails {
                    p.playing = 1;
                }
            }
            FEED => {
                s.feeds += 1;
                if s.feeds <= s.feed_stops {
                    p.playing = 0;
                }
                r.set_carry(false);
            }
            STOP => p.playing = 0,
            VOLUME => {}
            _ => {
                let d = std::slice::from_raw_parts((r.eax + 0xE) as *const u8, 2);
                let n = (d[0].wrapping_sub(b'0') as u32) * 10 + d[1].wrapping_sub(b'0') as u32;
                r.set_carry(n >= 64 || s.present & (1 << n) == 0);
            }
        }
    }
}

/// Begins a run of the stand-ins.
pub fn begin(script: Script) {
    *SCRIPT.lock().unwrap() = script;
    NOTED.lock().unwrap().clear();
}

pub fn noted() -> Vec<[u32; 3]> {
    std::mem::take(&mut *NOTED.lock().unwrap())
}
