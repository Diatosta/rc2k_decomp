//! The inspection's two fades (`menu::fade_in`, `menu::fade_out`): frames of
//! `menu::fade_frame` (`screens::run_frame`) for `dword_75AB44` milliseconds, the fade
//! quad's see-through `dword_75AB50` going from 0 up to (in) or from `dword_75AA94` down
//! (out) with the time gone (`dword_75AB4C`, from `dword_75AB48`).

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, SelfTest, call_game, game_addr, game_read, game_write, rva};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

pub const START_RVA: usize = rva(0x75AB48); // dword_75AB48
pub const GONE_RVA: usize = rva(0x75AB4C); // dword_75AB4C
pub const SEE_RVA: usize = rva(0x75AB50); // dword_75AB50
pub const LENGTH_RVA: usize = rva(0x75AB44); // dword_75AB44
pub const FULL_RVA: usize = rva(0x75AA94); // dword_75AA94
const RUN_FRAME: usize = rva(0x405A71); // screens::run_frame
const FADE_FRAME: usize = rva(0x4A1E60); // menu::fade_frame
const TICK_COUNT: usize = rva(0x411024); // system::tick_count
const DIVIDE_BY_ZERO: FatalError = FatalError {
    code: 0x80701010,
    message: "",
};

/// A call by place: 1 `system::tick_count` (eax), 2 `screens::run_frame` (eax the frame).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A fade: `out` the way. Every register kept (`pusha` .. `popa`).
pub unsafe fn fade(entry: &Regs, out: bool, part: Part) -> Result<Regs, Refused> {
    let length: u32 = unsafe { game_read(LENGTH_RVA) };
    if (1..100).contains(&length) {
        return Err(BadInput("a fade shorter than 100 ms: the original divides by 0").into());
    }
    let mut g = *entry;
    unsafe {
        part(1, &mut g)?;
        game_write(START_RVA, g.eax);
        game_write(SEE_RVA, if out { game_read(FULL_RVA) } else { 0u32 });
        g.eax = game_addr(FADE_FRAME) as u32;
        part(2, &mut g)?;
        loop {
            part(1, &mut g)?;
            let gone = g.eax.wrapping_sub(game_read(START_RVA));
            g.eax = gone;
            game_write(GONE_RVA, gone);
            if gone != 0 || out {
                let length: u32 = game_read(LENGTH_RVA);
                if length == 0 {
                    return Err(DIVIDE_BY_ZERO.into());
                }
                if gone == 0 {
                    // the way out takes the time gone of 0 to the division's check
                    return Err(DIVIDE_BY_ZERO.into());
                }
                g.edx = 0;
                g.ebp = length / 100;
                g.ebx = g.ebp;
                let share = gone / g.ebp;
                g.edx = gone % g.ebp;
                let full: u32 = game_read(FULL_RVA);
                if out {
                    g.eax = share.wrapping_mul(full);
                    g.ebx = full;
                    if g.eax == 0 {
                        g.ebx = 1;
                    } else {
                        g.edx = g.eax % 100;
                        g.ebp = 100;
                        g.eax /= 100;
                        g.ebx = full.wrapping_sub(g.eax);
                    }
                    game_write(SEE_RVA, g.ebx);
                } else {
                    g.eax = share;
                    g.ebx = full.wrapping_mul(share);
                    if g.ebx != 0 {
                        g.eax = g.ebx;
                        g.edx = g.eax % 100;
                        g.ebp = 100;
                        g.eax /= 100;
                        game_write(SEE_RVA, g.eax);
                    }
                }
            }
            g.eax = game_addr(FADE_FRAME) as u32;
            part(2, &mut g)?;
            g.eax = game_read(GONE_RVA);
            if (g.eax as i32) >= game_read::<i32>(LENGTH_RVA) {
                break;
            }
        }
    }
    Ok(*entry)
}

/// In the game: the clock's Rust version, the frame through the game (platform).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => r.eax = crate::system::tick_count::tick_count(),
        _ => unsafe { call_game(r, game_addr(RUN_FRAME)) },
    }
    Ok(())
}

// ---- the test: a copy whose clock and frame calls go to stand-ins ----

static CLOCK: AtomicU32 = AtomicU32::new(0);
static STEP: AtomicU32 = AtomicU32::new(0);
/// Each frame: the see-through at the time, and eax .. ebp.
static FRAMES: Mutex<Vec<[u32; 8]>> = Mutex::new(Vec::new());

fn stand_in(place: u32, r: &mut Regs) {
    match place {
        1 => {
            let now = CLOCK.fetch_add(STEP.load(Ordering::SeqCst), Ordering::SeqCst);
            r.eax = now;
        }
        _ => FRAMES.lock().unwrap().push([
            unsafe { game_read(SEE_RVA) },
            r.eax,
            r.ebx,
            r.ecx,
            r.edx,
            r.esi,
            r.edi,
            r.ebp,
        ]),
    }
}

/// The test of a fade at `address` (`len` bytes; `calls` its call and jump offsets).
pub fn test_fade(
    mut t: SelfTest,
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
    out: bool,
) -> (usize, usize) {
    use crate::install::stand_ins::{set_handler, stand_in as place};
    let copy = match unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| match c {
            TICK_COUNT => Some(place(1)),
            RUN_FRAME => Some(place(2)),
            _ => None,
        })
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(stand_in);
    let regions: Vec<Region> = [START_RVA, GONE_RVA, SEE_RVA].map(at).to_vec();
    let mut vars = regions.clone();
    vars.extend([LENGTH_RVA, FULL_RVA].map(at));
    let saved = capture(&vars);
    // (length, full see-through, the clock's step)
    for (length, full, step) in [
        (1000u32, 0xFF, 37u32),
        (400, 0x80, 100),
        (100, 0xFF, 7),
        (0x7D0, 0x40, 333),
    ] {
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(LENGTH_RVA, length);
            game_write(FULL_RVA, full);
            CLOCK.store(0x1000_0000, Ordering::SeqCst);
            STEP.store(step, Ordering::SeqCst);
            FRAMES.lock().unwrap().clear();
        };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            std::mem::take(&mut *FRAMES.lock().unwrap()),
            capture(&regions),
            ro.preserved_except(&Regs::SENTINEL, 0),
        );
        prepare();
        let mine = unsafe {
            fade(&Regs::SENTINEL, out, &mut |p, r| {
                stand_in(p, r);
                Ok(())
            })
        };
        let mine = (
            std::mem::take(&mut *FRAMES.lock().unwrap()),
            capture(&regions),
            mine.is_ok(),
        );
        t.check(theirs == mine, || {
            format!(
                "length {length}, full {full:#x}, step {step}: frames {} / {}, first differing {:?}; state same {}",
                theirs.0.len(),
                mine.0.len(),
                theirs.0.iter().zip(&mine.0).position(|(a, b)| a != b),
                theirs.1 == mine.1
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
