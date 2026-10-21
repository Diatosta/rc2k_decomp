//! `physics::track_checkpoints` (0x44157B): the work car's checkpoints followed.
//! `physics::find_car_checkpoint` into +0x98 and `dword_6365E4`;
//! `physics::checkpoint_offset`; `physics::off_line_warnings`; at the last checkpoint
//! of the current split (`g_checkCounts[+0x98]` - 1 equal to +0x90) +0x94 = +0x98; then,
//! below `g_splitColourCount` and when `physics::passed_split_point` gives CF, +0x94 and
//! +0x98 the split's checkpoint (`dword_601030[+0x90]`) and +0x90 up by one.
//!
//! In:  nothing
//! Out: the registers as the calls leave them; no flags read (the caller calls on)
//!
//! Caller: `sub_43B635` (0x43B90F), every physics step. It begins with a call: a private
//! copy, its calls relocated, a hand-made patch.
//! Refuses (before writing) a checkpoint past 0x1418 or a split past 0x40, and what the
//! calls refuse.
//! Shadow mode runs the original alone (its calls send sounds and read the stage).
//! Self-test: a copy whose four calls go to stand-ins (the split's CF by a pattern),
//! against the Rust version with the same, on the game's variables (saved and put back).

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{capture, restore};
use rc2k_core::physics::splits::{at_split_end, check_tracked, split_ahead};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::track_checkpoints",
    rva(0x44157B),
    thunk,
    &ORIGINAL,
)
.private_copy(0x6C)
.relocate_calls(&[0, 0xF, 0x14, 0x47])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The calls by place: 0 `find_car_checkpoint`, 1 `checkpoint_offset`, 2
/// `off_line_warnings`, 3 `passed_split_point` (its CF in the registers' flags).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;
const PLACES: [usize; 4] = [0x44139D, 0x4415E7, 0x44148F, 0x44165E];

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            0 => {
                *r = crate::physics::find_car_checkpoint::find_car_checkpoint(
                    r,
                    &mut crate::physics::reset_parts::game_part,
                )?
            }
            1 => crate::physics::checkpoint_offset::checkpoint_offset()?.apply(r),
            2 => crate::physics::off_line_warnings::off_line_warnings_in(
                r,
                &mut crate::physics::sound_cues::game_part,
            )?,
            _ => crate::physics::passed_split_point::passed_split_point(r)?.apply(r),
        }
    }
    Ok(())
}

/// The steps on `r`.
// Converted: `rc2k_core::physics::splits::{check_tracked, at_split_end, split_ahead}`.
pub fn track_checkpoints_in(r: &mut Regs, part: Part) -> Result<(), Refused> {
    part(0, r)?;
    check_tracked(r.eax, g(0x71A068))?;
    p(0x71A070, r.eax);
    p(0x6365E4, r.eax);
    part(1, r)?;
    part(2, r)?;
    r.ebx = g(0x71A070);
    r.eax = g(0x9E87A0 + 4 * r.ebx as usize).wrapping_sub(1);
    if at_split_end(r.eax.wrapping_add(1), g(0x71A068)) {
        r.eax = g(0x71A070);
        p(0x71A06C, r.eax);
    }
    r.ecx = g(0x71A068);
    if !split_ahead(r.ecx, g(0x601830)) {
        return Ok(());
    }
    part(3, r)?;
    if !r.carry() {
        return Ok(());
    }
    r.ecx = g(0x71A068);
    r.eax = g(0x601030 + 4 * r.ecx as usize);
    p(0x71A06C, r.eax);
    p(0x71A070, r.eax);
    p(0x71A068, g(0x71A068).wrapping_add(1));
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let mut g = *r;
    match track_checkpoints_in(&mut g, &mut game_part) {
        Ok(()) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- for the self-test ----

static CALLED: Mutex<Vec<(u32, [u32; 7])>> = Mutex::new(Vec::new());
static PASSED: AtomicU32 = AtomicU32::new(0);
static FOUND: AtomicU32 = AtomicU32::new(0);

fn handler(place: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push((place, [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]));
    match place {
        0 => r.eax = FOUND.load(Ordering::SeqCst),
        3 => r.set_carry(PASSED.load(Ordering::SeqCst) != 0),
        _ => {}
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    handler(place, r);
    Ok(())
}

fn copy() -> Result<RedirectedCopy, &'static str> {
    set_handler(handler);
    unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x44157B)),
            0x6C,
            &[0xE8, 0x1D, 0xFE, 0xFF, 0xFF],
            &[0, 0xF, 0x14, 0x47],
            &|c| {
                PLACES
                    .iter()
                    .position(|&k| rva(k) == c)
                    .map(|n| stand_in(n as u32))
            },
        )
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match copy() {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let a = |x: usize| game_addr(rva(x)) as u32;
    let all = [
        (a(0x71A068), 12),
        (a(0x6365E4), 4),
        (a(0x9E87A0), 0x40),
        (a(0x601830), 4),
        (a(0x601030), 0x40),
    ];
    let saved = capture(&all);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for seed in 0..12u32 {
        let fill = || {
            FOUND.store(seed % 6, Ordering::SeqCst);
            PASSED.store(seed % 2, Ordering::SeqCst);
            p(0x71A068, seed % 4);
            p(0x71A06C, 0x55);
            p(0x71A070, 0x66);
            p(0x601830, 3);
            for k in 0..16usize {
                p(0x9E87A0 + 4 * k, (k as u32 % 5) + 1);
                p(0x601030 + 4 * k, 10 + k as u32);
            }
        };
        fill();
        CALLED.lock().unwrap().clear();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            Some(regs(&ro)),
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(&all),
        );
        fill();
        let mut g = Regs::SENTINEL;
        let ok = track_checkpoints_in(&mut g, &mut test_part).is_ok();
        let mine = (
            ok.then(|| regs(&g)),
            std::mem::take(&mut *CALLED.lock().unwrap()),
            capture(&all),
        );
        t.check(theirs == mine, || {
            format!(
                "seed {seed}: original {:x?} {:x?}; rust {:x?} {:x?}",
                theirs.0, theirs.1, mine.0, mine.1
            )
        });
    }
    restore(&all, &saved);
    drop(copy);
    t.finish()
}
