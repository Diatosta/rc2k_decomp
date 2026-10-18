//! The car reset's steps under `sub_43B635` (stream B, 2026-10-17): `physics::false_start`
//! (0x43B9B4), `physics::reset_when_stuck` (0x43EFBA), `physics::reset_car` (0x43EFEF),
//! `physics::reset_car_on_line` (0x43FB00). Their calls go through a `Part` by place, as
//! stream A's `race/car_starts.rs` (its places 0 .. 12), with more of ours:
//! 13 `physics::reset_point_blocked`, 14 the log line (`log_write_l1_nl`, 0x40142B),
//! 15 `physics::reset_car`, 16 `physics::reset_car_on_line`, 17
//! `race::nearest_checkpoint_in`, 18 `physics::find_ground`, 19
//! `physics::count_ground_probe` (only the camera's ground checks call it). In the game the Rust
//! versions; in the self-tests copies whose calls go to the stand-ins (`handler`: stream
//! A's `note`, and for place 13 a carry by the case's pattern), and the Rust versions
//! with the same.
//!
//! Shadow mode runs these originals alone (A's `reset_car_motion` does: its calls into
//! the game).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, rva, shadow};
use crate::race::car_starts::{PLACES, Part, note, take_called};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub const BLOCKED: u32 = 13;
pub const LOG: u32 = 14;
pub const RESET_CAR: u32 = 15;
pub const ON_LINE: u32 = 16;
pub const NEAREST_IN: u32 = 17;
pub const FIND_GROUND: u32 = 18;
pub const COUNT_PROBE: u32 = 19;

/// Our places' functions (IDA addresses).
const EXTRA: [(usize, u32); 7] = [
    (0x438E22, COUNT_PROBE),
    (0x4413F8, NEAREST_IN),
    (0x441E29, FIND_GROUND),
    (0x43F0DF, BLOCKED),
    (0x40142B, LOG),
    (0x43EFEF, RESET_CAR),
    (0x43FB00, ON_LINE),
];

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            BLOCKED => crate::physics::reset_point_blocked::reset_point_blocked(r).apply(r),
            LOG => {
                // `log_write_l1_nl`: level 1, a line break.
                r.edx = 1;
                crate::logfile::log_write::log_write_regs(r);
            }
            RESET_CAR => *r = crate::physics::reset_car::reset_car(r, &mut game_part)?,
            ON_LINE => {
                *r = crate::physics::reset_car_on_line::reset_car_on_line(r, &mut game_part)?
            }
            NEAREST_IN => *r = crate::race::start_place::nearest_checkpoint_in(r)?,
            FIND_GROUND => {
                crate::physics::find_ground::find_ground()?;
            }
            COUNT_PROBE => {
                *r = crate::physics::count_ground_probe::count_ground_probe(r, &mut game_part)?
            }
            _ => crate::race::car_starts::game_part(place, r)?,
        }
    }
    Ok(())
}

/// The hooks' dispatch: the original alone in shadow mode, else `rust` with the game's
/// calls.
pub fn dispatch_reset(
    hook: &RawHook,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: unsafe fn(&Regs, Part) -> Result<Regs, Refused>,
) {
    hook.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { rust(r, &mut game_part) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

// ---- for the self-tests ----

/// Which calls to place 13 say blocked: bit k (mod 64) for the k-th.
static PATTERN: AtomicU64 = AtomicU64::new(0);
static BLOCKED_CALLS: AtomicU32 = AtomicU32::new(0);

/// The stand-ins: stream A's `note`, and for place 13 the carry by the pattern.
pub fn handler(place: u32, r: &mut Regs) {
    note(place, r);
    if place == BLOCKED {
        let k = BLOCKED_CALLS.fetch_add(1, Ordering::SeqCst);
        r.set_carry((PATTERN.load(Ordering::SeqCst) >> (k % 64)) & 1 != 0);
    }
}

/// The Rust side's part in a test.
pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    handler(place, r);
    Ok(())
}

/// Sets the pattern and starts the count over (and the noted calls).
pub fn start_case(pattern: u64) {
    PATTERN.store(pattern, Ordering::SeqCst);
    BLOCKED_CALLS.store(0, Ordering::SeqCst);
    take_called();
}

/// A copy of a step whose calls (at `calls`) go to the stand-ins by place.
pub fn copy_of(
    address: usize,
    len: usize,
    entry: &[u8],
    calls: &[usize],
) -> Result<RedirectedCopy, &'static str> {
    crate::install::stand_ins::set_handler(handler);
    unsafe {
        RedirectedCopy::new(game_addr(rva(address)), len, entry, calls, &|c| {
            PLACES
                .iter()
                .position(|&k| rva(k) == c)
                .map(|n| n as u32)
                .or_else(|| EXTRA.iter().find(|e| rva(e.0) == c).map(|e| e.1))
                .map(crate::install::stand_ins::stand_in)
        })
    }
}

/// What the steps read and write: stream A's regions and the reset's two variables
/// (`dword_614C4C`, `dword_614C50`).
pub fn regions() -> Vec<Region> {
    let mut r = crate::race::car_starts::regions();
    r.push((game_addr(rva(0x614C4C)) as u32, 8));
    r
}

/// One case: `fill` puts it in place; the copy and `rust` from the same inputs, the calls
/// with their registers, the registers left and the memory compared.
pub fn both(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    pattern: u64,
    entry: Regs,
    fill: &dyn Fn(),
    rust: unsafe fn(&Regs, Part) -> Result<Regs, Refused>,
) {
    both_with(t, what, copy, pattern, entry, fill, rust, &[]);
}

/// `both`, with more regions compared.
#[allow(clippy::too_many_arguments)] // one case's whole description
pub fn both_with(
    t: &mut SelfTest,
    what: &str,
    copy: &RedirectedCopy,
    pattern: u64,
    entry: Regs,
    fill: &dyn Fn(),
    rust: unsafe fn(&Regs, Part) -> Result<Regs, Refused>,
    extra: &[Region],
) {
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    let mut all = regions();
    all.extend_from_slice(extra);
    fill();
    start_case(pattern);
    let mut ro = entry;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = (Some(regs(&ro)), take_called(), capture(&all));
    fill();
    start_case(pattern);
    let mine = unsafe { rust(&entry, &mut test_part) };
    let mine = (mine.as_ref().ok().map(regs), take_called(), capture(&all));
    t.check(theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}
