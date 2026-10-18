//! `race::tyre_change` (0x444F7A): copies `dword_5D1444` / `dword_5D2444` to
//! `dword_5BEBE0` / `dword_5BEBE4`, then runs the tyre change's gauge `sub_41C1D6` for car
//! 0 (block at 0x70F3EC, "c1 tyrec" in the field table at 0x5ED068).
//!
//! In:  nothing
//! Out: eax (the gauge's result, or +0x298 & 0x1E when it is not called), esi 0 when it is
//!      called; the rest kept (the caller goes on with a `call`: no flags read)
//!
//! Starts the gauge (eax 0, esi 0; then +0x26C = 1) when not all four bits 1 .. 4 of
//! +0x298 are set, +0x26C is 0 and +0x268 is at least 300. While +0x26C is 1 it runs it
//! (eax 1); a time (not 0) is added to +0x88, +0x26C becomes -301, and each of the bits
//! not set before is set with its dword of +0x354 .. +0x360 cleared. A reading (the
//! gauge's on-screen text not looked at): the roadside tyre change, timed by a button
//! press on a gauge; see `notes/history.md`.
//! `sub_41C1D6` is not ported (it draws through `sub_41C38C`'s DirectDraw lock): called in
//! the game (section 15). Shadow mode runs the original alone (the gauge reads the clock
//! and the buttons and has its own state: it must not run twice). Self-test: a copy whose
//! variables are the test's and whose gauge call goes to a stand-in that notes eax and
//! esi and gives a time, against the Rust version on the same.

use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva, shadow};
use crate::race::frame::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::tyre_change", rva(0x444F7A), thunk, &ORIGINAL).self_test(self_test);

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
    match unsafe { tyre_change_at(r, &tyre_variables(), &mut tyre_part) } {
        Ok(after) => {
            r.eax = after.eax;
            r.esi = after.esi;
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

static CALLED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());
static GIVES: AtomicU32 = AtomicU32::new(0);

fn gauge(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push((r.eax, r.esi));
    r.eax = GIVES.load(Ordering::SeqCst);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(gauge);
    // Eight test dwords, then the four of +0x354.
    let mut cells = [0u32; 12];
    let base = cells.as_mut_ptr() as u32;
    let v: Vec<u32> = (0..9).map(|k| base + 4 * k as u32).collect();
    let mut patches: Vec<(usize, u32)> = TYRE_VARIABLES
        .iter()
        .zip(&v)
        .map(|(&a, &c)| (a, c))
        .collect();
    for k in 1..4 {
        patches.push((0x70F740 + 4 * k, v[8] + 4 * k as u32));
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x444F7A,
            len: 0xCD,
            entry: &[0xA1, 0x44, 0x14, 0x5D, 0x00],
            calls: &[0x40, 0x62],
            jumps_out: &[],
        },
        &|c| (c == rva(0x41C1D6)).then(|| crate::install::stand_ins::stand_in(1)),
        &patches,
        19,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let snapshot = |cells: *const u32| unsafe { std::slice::from_raw_parts(cells, 12).to_vec() };
    for (count, state, bits, gives) in [
        (300u32, 0u32, 0u32, 0u32),
        (300, 0, 0, 1234),
        (299, 0, 0, 1234),
        (0x8000_0000, 0, 0x1E, 55),
        (0x8000_0000, 0, 0x1C, 55),
        (500, 1, 0x0A, 777),
        (500, 1, 0xFFFF_FFFF, 777),
        (500, 1, 0, 0),
        (500, 0xFFFF_FED3, 0, 9),
        (500, 2, 0x10, 9),
    ] {
        GIVES.store(gives, Ordering::SeqCst);
        let prepare = |cells: *mut u32| unsafe {
            for k in 0..12 {
                cells.add(k).write(0xA000_0000 + k as u32);
            }
            cells.add(4).write(count);
            cells.add(5).write(state);
            cells.add(6).write(bits);
        };
        let p = cells.as_mut_ptr();
        prepare(p);
        CALLED.lock().unwrap().clear();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = (
            [ro.eax, ro.ebx, ro.ecx, ro.edx, ro.esi, ro.edi, ro.ebp],
            std::mem::take(&mut *CALLED.lock().unwrap()),
            snapshot(p),
        );
        prepare(p);
        let mine = unsafe {
            tyre_change_at(&Regs::SENTINEL, &v, &mut |place, r| {
                gauge(place, r);
                Ok(())
            })
        };
        let Ok(m) = mine else {
            t.check(false, || format!("count {count} state {state:#x}: refused"));
            continue;
        };
        let mine = (
            [m.eax, m.ebx, m.ecx, m.edx, m.esi, m.edi, m.ebp],
            std::mem::take(&mut *CALLED.lock().unwrap()),
            snapshot(p),
        );
        t.check(theirs == mine, || {
            format!(
                "count {count:#x} state {state:#x} bits {bits:#x} gives {gives}: original \
                 {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    t.finish()
}
