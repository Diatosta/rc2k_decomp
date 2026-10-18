//! The car's sound cues (IDA 2026-10-18): `physics::off_line_warnings` (0x44148F) counts
//! the car off its line and turned from it and, now and then, cues a sound by
//! `physics::cue_listed_car_sound` (0x45D1BC) -> `physics::cue_car_sound` (0x40ECE7) ->
//! `physics::queue_sound_cue` (0x40EE5C), which may stop the playing one by
//! `sound::play_sound`. This file: their calls by place (0 `play_sound`, 1
//! `queue_sound_cue`, 2 `cue_car_sound`, 3 `cue_listed_car_sound`): in the game the Rust
//! versions, in the tests stand-ins that note the calls and their registers (and change
//! none), and copies of the originals whose calls go to them.
//!
//! Shadow mode runs these originals alone (a sound command must not be sent twice).

use crate::errors::Refused;
use crate::framework::{Regs, call_game, game_addr, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use std::sync::Mutex;

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

pub const PLAY: u32 = 0;
pub const QUEUE: u32 = 1;
pub const CUE: u32 = 2;
pub const LISTED: u32 = 3;
const PLACES: [usize; 4] = [0x409B36, 0x40EE5C, 0x40ECE7, 0x45D1BC];

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        PLAY => crate::sound::play_sound::play_sound_in_game(r),
        QUEUE => crate::physics::queue_sound_cue::queue_sound_cue_in(r, &mut game_part)?,
        CUE => crate::physics::cue_car_sound::cue_car_sound_in(r, &mut game_part)?,
        _ => crate::physics::cue_listed_car_sound::cue_listed_car_sound_in(r, &mut game_part)?,
    }
    Ok(())
}

static CALLED: Mutex<Vec<(u32, [u32; 7])>> = Mutex::new(Vec::new());

/// The stand-ins: each call noted, the registers left as they came.
pub fn handler(place: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]));
}

pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    handler(place, r);
    Ok(())
}

pub fn take_called() -> Vec<(u32, [u32; 7])> {
    std::mem::take(&mut *CALLED.lock().unwrap())
}

/// A copy of one whose calls (at `calls`) go to the stand-ins.
pub fn copy_of(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
) -> Result<RedirectedCopy, &'static str> {
    set_handler(handler);
    unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            PLACES
                .iter()
                .position(|&k| rva(k) == c)
                .map(|n| stand_in(n as u32))
        })
    }
}

/// The hooks' dispatch: the original alone in shadow mode, else `rust` with the game's
/// calls; the registers as it leaves them.
pub fn dispatch_cue(
    hook: &crate::framework::RawHook,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: fn(&mut Regs, Part) -> Result<(), Refused>,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut g = *r;
    match rust(&mut g, &mut game_part) {
        Ok(()) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

/// One test case: `fill` then the copy, `fill` then `rust` with the stand-ins; the calls,
/// the registers and the memory compared.
pub fn both(
    t: &mut crate::framework::SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    entry: Regs,
    fill: &dyn Fn(),
    all: &[crate::service::harness::Region],
    rust: fn(&mut Regs, Part) -> Result<(), Refused>,
) {
    use crate::service::harness::capture;
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    fill();
    take_called();
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), capture(all));
    fill();
    let mut g = entry;
    let ok = rust(&mut g, &mut test_part).is_ok();
    let mine = (ok.then(|| regs(&g)), take_called(), capture(all));
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}
