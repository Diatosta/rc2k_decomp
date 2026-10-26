//! `race::wake_car_sounds` (0x45D255): the cars' sound objects woken at the stage's loading.
//!
//! In:  nothing
//! Out: ebx = the cars' count and edi = the last object woken, when any is; every other
//!      register preserved
//!
//! See `race/sounds.rs`. Callers: `race::load_stage` and two more. Refuses 0x10000 cars or
//! more, a sound object missing. Shadow mode runs the original alone. Self-test: a
//! redirected copy whose call of `race::start_race_sounds` notes itself, the game's
//! objects' flags, `dword_71C3E0` 0 and 1, the cars 0, 1, 5 and 9, put back afterwards.

use crate::framework::{
    EBX, EDI, RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::race::sounds::*;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::wake_car_sounds", rva(0x45D255), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { wake_car_sounds(r, &mut game_wake_part) } {
        Ok(left) => {
            r.ebx = left.ebx;
            r.edi = left.edi;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { wake_car_sounds(r, &mut game_wake_part) }?;
        r.ebx = left.ebx;
        r.edi = left.edi;
    }
    Ok(())
}

static CALLED: Mutex<Vec<(u32, u32, u32)>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((place, r.ebx, r.edi));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !objects_there() {
        return t.skip("a race sound object missing");
    }
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x45D255)),
            0x39,
            &[0x83, 0x3D, 0xE0, 0xC3, 0x71, 0x00, 0x00],
            &[0x33],
            &|_| Some(stand_in(2)),
        )
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    let regions = regions();
    let mut all = regions.clone();
    all.extend([at(CARS_RVA), at(REPLAY_RVA)]);
    let saved = capture(&all);
    for replay in [0u32, 1] {
        for cars in [0u32, 1, 5, 9] {
            let run = |rust: bool| {
                restore(&all, &saved);
                unsafe {
                    game_write(CARS_RVA, cars);
                    game_write(REPLAY_RVA, replay);
                }
                for &(at, _) in &regions[..8] {
                    unsafe { (at as *mut u32).write_unaligned(0) };
                }
                CALLED.lock().unwrap().clear();
                let before = Regs::SENTINEL;
                let left = if rust {
                    unsafe {
                        wake_car_sounds(&before, &mut |p, r| {
                            handler(p, r);
                            Ok(())
                        })
                    }
                    .ok()
                    .filter(|l| l.preserved_except(&before, EBX | EDI))
                    .map(|l| (l.ebx, l.edi))
                } else {
                    let mut ro = before;
                    unsafe { call_game(&mut ro, copy.addr()) };
                    ro.preserved_except(&before, EBX | EDI)
                        .then_some((ro.ebx, ro.edi))
                };
                (
                    left,
                    std::mem::take(&mut *CALLED.lock().unwrap()),
                    capture(&regions),
                )
            };
            let theirs = run(false);
            let mine = run(true);
            t.check(theirs.0.is_some() && theirs == mine, || {
                format!("replay {replay}, cars {cars}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    restore(&all, &saved);
    t.finish()
}
