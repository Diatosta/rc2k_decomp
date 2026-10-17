//! `drive::race_mode_cars` (0x436758): by the race's mode `dword_62389C`: 2 runs
//! `drive::start_cars_behind`; 0, 1, 3, 4 and the others nothing (five separate `retn`s).
//!
//! In:  nothing
//! Out: as `start_cars_behind` leaves them (mode 2), else every register kept; no flags
//!      read (its callers return next, theirs go on with `popa` / `mov`)
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`.
//! Refuses what `start_cars_behind` refuses.
//! Shadow mode: as `start_cars_behind` (the original alone, in mode 2).
//! Self-test: a copy whose call goes to a stand-in, against the Rust version with the
//! same, modes 0 .. 5.

use crate::drive::start_cars_behind::{game_part, start_cars_behind};
use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::car_starts::Part;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::race_mode_cars", rva(0x436758), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const MODE_RVA: usize = rva(0x62389C); // dword_62389C

/// The Rust core, `start_cars_behind` through `part`.
pub unsafe fn race_mode_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if unsafe { game_read::<u32>(MODE_RVA) } == 2 {
        let mut r = *entry;
        part(0, &mut r)?;
        return Ok(r);
    }
    Ok(*entry)
}

fn game(_: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = unsafe { start_cars_behind(r, &mut game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() && unsafe { game_read::<u32>(MODE_RVA) } == 2 {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { race_mode_cars(r, &mut game) } {
        Ok(g) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

static CALLS: AtomicU32 = AtomicU32::new(0);

fn note(_: u32, r: &mut Regs) {
    CALLS.fetch_add(1, Ordering::SeqCst);
    r.eax ^= 0x5A5A_0001;
    r.esi = r.esi.wrapping_add(4);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match unsafe {
        crate::redirect::RedirectedCopy::new(
            game_addr(rva(0x436758)),
            0x38,
            &[0x83, 0x3D, 0x9C, 0x38, 0x62, 0x00, 0x02],
            &[0x2E],
            &|c| (c == rva(0x43703E)).then(|| crate::install::stand_ins::stand_in(0)),
        )
    } {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let saved: u32 = unsafe { game_read(MODE_RVA) };
    for mode in 0..6u32 {
        unsafe { game_write(MODE_RVA, mode) };
        CALLS.store(0, Ordering::SeqCst);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (CALLS.swap(0, Ordering::SeqCst), ro.eax, ro.esi);
        let mine = unsafe {
            race_mode_cars(&Regs::SENTINEL, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = mine.map(|g| (CALLS.swap(0, Ordering::SeqCst), g.eax, g.esi));
        let ok = mine.as_ref().is_ok_and(|m| *m == theirs)
            && ro.preserved_except(
                &Regs::SENTINEL,
                crate::framework::EAX | crate::framework::ESI,
            );
        t.check(ok, || {
            format!("mode {mode}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    unsafe { game_write(MODE_RVA, saved) };
    drop(copy);
    t.finish()
}
