//! `scene::set_up_engine` (0x442AF6): the 3D code's tables, and the cars' figures.
//!
//! In:  nothing
//! Out: as `sub_4217E4` leaves them
//!
//! `scene::fill_engine_tables`, then `sub_4217E4` (the cars' physics run for the
//! figures of the car specification screen, by the look of it; not ported,
//! `call_game`). 22 call sites, among them `menu::main_menu_loop` (twice) and
//! `menu::play_films`.
//! Manual patch with a private copy: the function is two calls.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose two calls note the registers they get.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::fill_engine_tables::fill_engine_tables;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::set_up_engine", rva(0x442AF6), thunk, &ORIGINAL)
    .private_copy(0xB)
    .relocate_calls(&[0, 5])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TABLES_RVA: usize = rva(0x41F6BA); // scene::fill_engine_tables
const CARS_RVA: usize = rva(0x4217E4); // sub_4217E4
const ENTRY_BYTES: [u8; 5] = [0xE8, 0xBF, 0xCB, 0xFD, 0xFF];

// ---- Rust version ----

/// `step` stands for the two calls: 0 the tables, 1 the cars.
pub fn set_up_engine(r: &mut Regs, step: &mut dyn FnMut(u32, &mut Regs)) {
    step(0, r);
    step(1, r);
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { set_up_engine_in_game(r) };
}

/// With the tables' Rust version and the cars through the game.
pub unsafe fn set_up_engine_in_game(r: &mut Regs) {
    set_up_engine(r, &mut |n, r| unsafe {
        if n == 0 {
            fill_engine_tables(r);
        } else {
            call_game(r, game_addr(CARS_RVA));
        }
    });
}

// ---- self-test ----

static CALLED: Mutex<Vec<[u32; 8]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push([place, r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]);
    r.eax = r.eax.wrapping_add(place + 1);
    r.esi ^= 0x55;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x442AF6)),
            0xB,
            &ENTRY_BYTES,
            &[0, 5],
            &|callee| {
                [TABLES_RVA, CARS_RVA]
                    .iter()
                    .position(|&known| known == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    let before = Regs::SENTINEL;
    CALLED.lock().unwrap().clear();
    let mut theirs = before;
    unsafe { call_game(&mut theirs, copy.addr()) };
    let their_calls = std::mem::take(&mut *CALLED.lock().unwrap());
    let mut mine = before;
    set_up_engine(&mut mine, &mut |n, r| act(n, r));
    let my_calls = std::mem::take(&mut *CALLED.lock().unwrap());
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    t.check(
        regs(&theirs) == regs(&mine) && their_calls == my_calls,
        || {
            format!(
                "regs {:x?} / {:x?}; calls {their_calls:x?} / {my_calls:x?}",
                regs(&theirs),
                regs(&mine)
            )
        },
    );
    t.finish()
}
