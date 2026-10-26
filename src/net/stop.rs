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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{DESTROY, ON, PUMP, Part, SLEEP_1MS, a, net_entry};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, stop)
}
