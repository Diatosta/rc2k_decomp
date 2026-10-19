//! `net::stop` (0x41838C): the network race ended (`dword_52A910` 0); while the receiving
//! thread is busy with a message (`dword_52A918`), a millisecond's sleep and the window's
//! messages, until it is done, or the game is closing (`g_quitting` 1) and the thread has
//! ended (`dword_52A4B0` 1); then `net::destroy_connection`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `net/mod.rs`. Four callers: `net::shut_down` and three of the network's menus
//! (`sub_416EEC`, `sub_416F0B`, `sub_417089`). Its calls through `part` (`sleep_1ms` as
//! `Sleep(1)`, the window's messages through the game, the platform's in the standalone).
//! It waits on another thread: the thunk does not wait for self-tests. Shadow mode runs
//! the original alone. Self-test: a copy whose four variables are the test's and whose
//! calls go to noting stand-ins, against the Rust version, on the cases that end (a stand-
//! in changes nothing): not busy; busy, closing and the thread ended.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{DESTROY, ON, PUMP, Part, SLEEP_1MS, a, dispatch_net};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::stop", rva(0x41838C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

const BUSY: usize = 0x52A918; // dword_52A918: the receiving thread is busy
const ENDED: usize = 0x52A4B0; // dword_52A4B0: the receiving thread has ended
const QUITTING: usize = 0x51837C; // g_quitting

/// Where the variables are.
#[derive(Clone, Copy)]
pub struct Vars {
    pub on: u32,
    pub busy: u32,
    pub ended: u32,
    pub quitting: u32,
}

/// The Rust version on the variables in `v`.
pub fn stop_with(r: &mut Regs, v: &Vars, part: Part) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_volatile() };
    unsafe { (v.on as *mut u32).write_volatile(0) };
    let mut g = *r;
    while rd(v.busy) != 0 {
        part(SLEEP_1MS, &mut g)?;
        part(PUMP, &mut g)?;
        if rd(v.quitting) == 1 && rd(v.ended) == 1 {
            break;
        }
    }
    part(DESTROY, &mut g)
}

/// The Rust version.
pub fn stop(r: &mut Regs, part: Part) -> Result<(), Refused> {
    let v = Vars {
        on: a(ON),
        busy: a(BUSY),
        ended: a(ENDED),
        quitting: a(QUITTING),
    };
    stop_with(r, &v, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, stop);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u32; 4];
    let base = mem.as_mut_ptr() as u32;
    let v = Vars {
        on: base,
        busy: base + 4,
        ended: base + 8,
        quitting: base + 12,
    };
    // pusha; mov dword_52A910, 0
    let entry = [
        0x60, 0xC7, 0x05, 0x10, 0xA9, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let copy = match copy_with(
        0x41838C,
        0x37,
        &entry,
        &[0x14, 0x19, 0x30],
        &[SLEEP_1MS, PUMP, DESTROY],
        &[],
        &[
            (ON, 4, v.on),
            (BUSY, 4, v.busy),
            (ENDED, 4, v.ended),
            (QUITTING, 4, v.quitting),
        ],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for (busy, ended, quitting) in [(0u32, 0u32, 0u32), (0, 1, 1), (1, 1, 1), (7, 1, 1)] {
        compare(
            &mut t,
            &format!("busy {busy}, ended {ended}, quitting {quitting}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe { (base as *mut [u32; 4]).write_volatile([1, busy, ended, quitting]) },
            &[(base, 16)],
            &|r, part| stop_with(r, &v, part),
            false,
        );
    }
    let _ = &mut mem;
    t.finish()
}
