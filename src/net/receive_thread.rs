//! `net::receive_thread` (0x4183C3): the network's receiving thread ("NETRXTDB"), the
//! routine of the third slot `threading::start_threads` starts (`stru_518714`), from its
//! start to `ExitThread`.
//!
//! In:  the registers the thread starter leaves (ecx, edx, edi, ebp handed on to the calls)
//! Out: does not return (`ExitThread(0)`)
//! Its slot's byte +0xC (running) 1 already: straight to the end. Else running;
//! "NETRXTDB:START" (log level 2); `dword_52A4B0` (done) not 0: the end. Else round and
//! round: `threading::wait_for_object` on the event `dword_52AB68` (`net::set_up`'s) for
//! 1000 ms; signalled (zero clear): `dword_52A918` (receiving) 1 and, with `dword_52A910`
//! (a network session) 1, `net::take_messages` (DirectPlay's messages taken in; the
//! Rust version, in the standalone program too); then
//! `dword_52A918` 0; with `dword_52ACE4` 1 the call posted in `dword_52ACE8` (taken with an
//! `xchg`, eax its address) is made and its eax kept in `dword_52ACEC`, `dword_52ACF0` 1
//! (`sub_4184F8` posts them and waits); until `g_quitting` is 1. The end: done 1,
//! "NETRXTDB:QUIT", not running, `ExitThread(0)`.
//! The registers go on from call to call as in the original (the posted call's are kept).
//! The thunk is `ungated` (the thread is in it for the whole game); a turn's calls into the
//! game hold the gate, so a self-test's pause stops them.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, ZERO_FLAG, game_addr, hook_enter, hook_leave, rva};
use std::sync::atomic::{AtomicU32, Ordering};

// ---- game data ----

const START_TEXT_RVA: usize = rva(0x4DACD2); // "NETRXTDB:START"
const QUIT_TEXT_RVA: usize = rva(0x4DACE1); // "NETRXTDB:QUIT"
/// The wait for the event, in milliseconds.
const WAIT: u32 = 1000;
/// The calls by place: 0 `logfile::log_write_l2_nl` (eax), 1 `threading::wait_for_object`
/// (eax, ebx -> the zero flag), 2 `net::take_messages`, 3 the posted call (eax its address), 4
/// `ExitThread(0)` (its thunk, 0x4D5D10).
const CALLEES: [usize; 3] = [0x401439, 0x411063, 0x4194CC];
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct ReceiveVars {
    /// stru_518714, the thread's slot.
    pub slot: u32,
    /// dword_52A4B0
    pub done: u32,
    /// dword_52AB68
    pub event: u32,
    /// dword_52A918
    pub receiving: u32,
    /// dword_52A910
    pub session: u32,
    /// dword_52ACE4
    pub posting: u32,
    /// dword_52ACE8
    pub posted: u32,
    /// dword_52ACEC
    pub result: u32,
    /// dword_52ACF0
    pub answered: u32,
    /// g_quitting
    pub quitting: u32,
}

impl ReceiveVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            slot: a(0x518714),
            done: a(0x52A4B0),
            event: a(0x52AB68),
            receiving: a(0x52A918),
            session: a(0x52A910),
            posting: a(0x52ACE4),
            posted: a(0x52ACE8),
            result: a(0x52ACEC),
            answered: a(0x52ACF0),
            quitting: a(0x51837C),
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

unsafe fn set(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_volatile(value) };
}

/// Runs to the `ExitThread` (place 4), which in the game does not come back.
pub unsafe fn receive_thread(entry: &Regs, v: &ReceiveVars, part: Part) -> Result<(), Refused> {
    unsafe {
        let mut c = Regs {
            esi: v.slot,
            ..*entry
        };
        let running = (v.slot + 0xC) as *mut u8;
        if running.read_volatile() != 1 {
            running.write_volatile(1);
            part(
                0,
                &mut Regs {
                    eax: game_addr(START_TEXT_RVA) as u32,
                    ..c
                },
            )?;
            if get(v.done) == 0 {
                loop {
                    c.eax = get(v.event);
                    c.ebx = WAIT;
                    part(1, &mut c)?;
                    if c.eflags & ZERO_FLAG == 0 {
                        set(v.receiving, 1);
                        if get(v.session) == 1 {
                            part(2, &mut c)?;
                        }
                    }
                    set(v.receiving, 0);
                    if get(v.posting) == 1 {
                        c.eax = AtomicU32::from_ptr(v.posted as *mut u32).swap(0, Ordering::SeqCst);
                        if c.eax != 0 {
                            part(3, &mut c)?;
                            set(v.result, c.eax);
                            set(v.answered, 1);
                        }
                    }
                    if get(v.quitting) == 1 {
                        break;
                    }
                }
            }
        }
        set(v.done, 1);
        part(
            0,
            &mut Regs {
                eax: game_addr(QUIT_TEXT_RVA) as u32,
                ..c
            },
        )?;
        c.esi = v.slot;
        running.write_volatile(0);
        part(4, &mut c)?;
    }
    Ok(())
}

/// In the game: the log, the wait and the messages (`net::take_messages`) from the Rust
/// versions, the posted call through the game; a turn's messages and posted call hold the
/// gate.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    struct Gate;
    impl Drop for Gate {
        fn drop(&mut self) {
            hook_leave();
        }
    }
    match place {
        1 => {
            let (carry, zero) =
                crate::threading::wait_for_object::wait_for_object(r.eax, r.ebx).flags();
            r.set_carry(carry);
            r.set_zero(zero);
        }
        2 => {
            hook_enter();
            let _gate = Gate;
            crate::calls::call(rva(0x4194CC), r); // net::take_messages
        }
        3 => {
            hook_enter();
            let _gate = Gate;
            crate::menu::table_calls::call_any(r.eax, r);
        }
        4 => unsafe { windows_sys::Win32::System::Threading::ExitThread(0) },
        _ => crate::calls::call(rva(CALLEES[0]), r),
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { receive_thread(&entry, &ReceiveVars::game(), &mut game_part) }?;
    Ok(())
}
