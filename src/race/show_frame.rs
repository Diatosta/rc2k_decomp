//! `race::show_frame` (0x4547AE): the race frame's end, after its 3D scene: the frame
//! locked (`scene::lock_frame`; carry: lost), the race's sound tick (eax 0x13), the credits
//! rolled while `dword_71C3E0` is set (`race::roll_credits`, eax 1), the frame rate drawn
//! (`race::draw_frame_rate`), the frame unlocked and shown (`scene::unlock_frame`,
//! `scene::present_frame`; carry: lost); the sound tick (eax 0x14), the clock restarted
//! (`scene::restart_clock`) and its time less `dword_63DC38` into `dword_63DC3C`, the frame
//! rate measured (`race::measure_frame_rate`); carry clear.
//!
//! In:  nothing
//! Out: carry set when the frame was lost (the lock or the showing failed), else clear; the
//!      registers as the last call leaves them
//!
//! Caller: `drive::race_frame` (its part 7). It begins with a call: a private copy (its
//! nine calls relocated) and a manual patch. Its calls through `part` by place
//! (`CALLEES`); in the game the sound tick's and the measuring's Rust versions, the others
//! (all hooked) through the game. Shadow mode runs the original alone (it shows the
//! frame). Self-test: a copy whose calls go to stand-ins (the lock and the showing failing
//! or not), against the Rust version, with and without the credits.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::mud::copy;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::show_frame", rva(0x4547AE), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x58;
const CALLS: [usize; 9] = [0x0, 0xC, 0x1F, 0x24, 0x29, 0x2E, 0x3A, 0x3F, 0x4F];

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 8] = [
    0x44E837, // 0 scene::lock_frame
    0x421BA9, // 1 sound::race_sound_tick
    0x415FC2, // 2 race::roll_credits
    0x4562B2, // 3 race::draw_frame_rate
    0x44E851, // 4 scene::unlock_frame
    0x44E895, // 5 scene::present_frame
    0x450381, // 6 scene::restart_clock
    0x42009E, // 7 race::measure_frame_rate
];

const CREDITS_RVA: usize = rva(0x71C3E0); // dword_71C3E0
const CLOCK_BASE_RVA: usize = rva(0x63DC38); // dword_63DC38
const FRAME_TIME_RVA: usize = rva(0x63DC3C); // dword_63DC3C

/// A call by place (`CALLEES`).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers and carry it leaves.
pub unsafe fn show_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    part(0, &mut r)?;
    if r.carry() {
        return Ok(r);
    }
    r.eax = 0x13;
    part(1, &mut r)?;
    if unsafe { game_read::<u32>(CREDITS_RVA) } != 0 {
        r.eax = 1;
        part(2, &mut r)?;
    }
    for place in [3, 4, 5] {
        part(place, &mut r)?;
    }
    if r.carry() {
        return Ok(r);
    }
    r.eax = 0x14;
    part(1, &mut r)?;
    part(6, &mut r)?;
    r.eax = r.eax.wrapping_sub(unsafe { game_read(CLOCK_BASE_RVA) });
    unsafe { game_write(FRAME_TIME_RVA, r.eax) };
    part(7, &mut r)?;
    r.set_carry(false);
    Ok(r)
}

/// In the game: the sound tick's and the measuring's Rust versions, the others' through
/// the game (their hooks).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => crate::sound::race_sound_tick::race_sound_tick_in(
            r,
            &mut crate::sound::turn_parts::game_part,
        ),
        7 => {
            let flags = r.eflags;
            *r = Regs {
                eflags: flags,
                ..unsafe {
                    // Converted: the measuring's `_plain` version (the frame's comparison
                    // runs the original alone).
                    crate::race::measure_frame_rate::measure_frame_rate_plain(
                        r,
                        &mut crate::race::measure_frame_rate::rdtsc,
                    )
                }
            };
            Ok(())
        }
        _ => {
            crate::calls::call(rva(CALLEES[place as usize]), r);
            Ok(())
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { show_frame(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// Which of the test's calls fail (carry), by place bit (1 the lock, 0x20 the showing).
static FAILING: AtomicU32 = AtomicU32::new(0);

/// The stand-ins: noted (`level_init::note`); the lock and the showing set carry by
/// `FAILING` (clear otherwise, as the calls before a `jb` do); the clock gives 0x5000.
fn note(place: u32, r: &mut Regs) {
    crate::race::level_init::note(place, r);
    match place {
        0 | 5 => r.set_carry(FAILING.load(Ordering::SeqCst) & (1 << place) != 0),
        6 => r.eax = 0x5000,
        _ => {}
    }
}

fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let places: &'static [(usize, u32)] = Box::leak(
        CALLEES
            .iter()
            .enumerate()
            .map(|(p, &a)| (a, p as u32))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    // call scene::lock_frame (its relative address from 0x4547AE)
    let rel = (0x44E837i64 - (0x4547AE + 5)) as i32;
    let b = rel.to_le_bytes();
    let copy = match copy(
        0x4547AE,
        LEN,
        &[0xE8, b[0], b[1], b[2], b[3]],
        &CALLS,
        places,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::install::stand_ins::set_handler(note);
    let regions = [at(CREDITS_RVA), at(CLOCK_BASE_RVA), at(FRAME_TIME_RVA)];
    let saved = capture(&regions);
    for (credits, failing) in [(0u32, 0u32), (1, 0), (0, 1), (1, 0x20), (0, 0x21)] {
        let what = format!("credits {credits}, failing {failing:#x}");
        let prepare = || unsafe {
            game_write(CREDITS_RVA, credits);
            game_write(CLOCK_BASE_RVA, 0x1234u32);
            game_write(FRAME_TIME_RVA, 0xAAAA_AAAAu32);
            FAILING.store(failing, Ordering::SeqCst);
        };
        // `run_both` without the carry: compared here with it.
        let regs = |r: &Regs| {
            [
                r.eax,
                r.ebx,
                r.ecx,
                r.edx,
                r.esi,
                r.edi,
                r.ebp,
                r.carry() as u32,
            ]
        };
        prepare();
        crate::race::level_init::take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            regs(&ro),
            crate::race::level_init::take_called(),
            capture(&regions),
        );
        prepare();
        let mine = unsafe { show_frame(&Regs::SENTINEL, &mut test_part) }.map(|r| regs(&r));
        let mine = (
            mine,
            crate::race::level_init::take_called(),
            capture(&regions),
        );
        t.check(
            Ok(theirs.0) == mine.0 && theirs.1 == mine.1 && theirs.2 == mine.2,
            || format!("{what}: original {theirs:x?}; rust {mine:x?}"),
        );
    }
    FAILING.store(0, Ordering::SeqCst);
    restore(&regions, &saved);
    t.finish()
}
