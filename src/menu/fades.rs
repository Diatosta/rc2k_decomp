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
    // Converted: `rc2k_core::menu::fades` (`too_short`, `first_see`, `step`, `over`).
    use rc2k_core::menu::fades::{self, Stop};
    let length: u32 = unsafe { game_read(LENGTH_RVA) };
    if fades::too_short(length) {
        return Err(BadInput("a fade shorter than 100 ms: the original divides by 0").into());
    }
    let mut g = *entry;
    unsafe {
        part(1, &mut g)?;
        game_write(START_RVA, g.eax);
        game_write(SEE_RVA, fades::first_see(out, game_read(FULL_RVA)));
        g.eax = game_addr(FADE_FRAME) as u32;
        part(2, &mut g)?;
        loop {
            part(1, &mut g)?;
            let gone = g.eax.wrapping_sub(game_read(START_RVA));
            g.eax = gone;
            game_write(GONE_RVA, gone);
            // The length and the full see-through read only where the original reads them.
            let made = if gone != 0 || out {
                fades::step(gone, game_read(LENGTH_RVA), || game_read(FULL_RVA), out)
            } else {
                Ok(None)
            };
            match made {
                Ok(Some(s)) => {
                    (g.eax, g.ebx, g.edx, g.ebp) = (s.eax, s.ebx, s.edx, s.ebp);
                    if let Some(see) = s.see {
                        game_write(SEE_RVA, see);
                    }
                }
                Ok(None) => {}
                Err(Stop::DivideByZero) => return Err(DIVIDE_BY_ZERO.into()),
                Err(Stop::ShortStep) => {
                    return Err(BadInput("a fade's step of 0 ms: the original divides by 0").into());
                }
            }
            g.eax = game_addr(FADE_FRAME) as u32;
            part(2, &mut g)?;
            g.eax = game_read(GONE_RVA);
            if fades::over(g.eax, game_read(LENGTH_RVA)) {
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
        _ => crate::calls::call(RUN_FRAME, r),
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
    // `calls` holds the calls and the `jcc rel32` to the fatal-error stubs; the copy moves
    // the calls, the jumps are pointed back at their targets.
    let original = game_addr(rva(address));
    // The copy's bytes: the entry over the original (whose first bytes are the patch).
    let byte = |o: usize| match entry.get(o) {
        Some(&b) => b,
        None => unsafe { *((original + o) as *const u8) },
    };
    let (only_calls, jumps): (Vec<usize>, Vec<usize>) =
        calls.iter().partition(|&&o| byte(o) == 0xE8);
    let copy = match unsafe {
        RedirectedCopy::new(original, len, entry, &only_calls, &|c| match c {
            TICK_COUNT => Some(place(1)),
            RUN_FRAME => Some(place(2)),
            _ => None,
        })
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    for o in jumps {
        let rel = unsafe { ((original + o + 2) as *const i32).read_unaligned() };
        let target = (original + o + 6).wrapping_add(rel as usize);
        unsafe { copy.retarget(o + 2, target) };
    }
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
