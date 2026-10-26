//! `scene::set_up_object_memory` (0x4433C9): the objects' memory and the kinds' records.
//!
//! In:  nothing
//! Out: as `scene::allocate_object_memory` leaves them
//!
//! `scene::allocate_object_memory`, then `scene::read_kind_records`.
//! Two callers (the front end's 3D set-up and a race's).
//! Manual patch with a private copy: the function begins with a call.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note themselves.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::allocate_object_memory::allocate_object_memory_in_game;
use crate::scene::read_kind_records::read_kind_records;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::set_up_object_memory",
    rva(0x4433C9),
    thunk,
    &ORIGINAL,
)
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

/// What it calls, in the order of their first calls.
const CALLEES: [usize; 2] = [rva(0x41F814), rva(0x439975)];
/// The calls made, as places in `CALLEES`.
const ORDER: [usize; 2] = [0, 1];

// ---- Rust version ----

/// `step`: what stands for each call, by its place in `ORDER`; it is entered with the
/// registers the call before left.
pub unsafe fn set_up_object_memory(
    r: &mut Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    for n in 0..ORDER.len() {
        step(n, r)?;
    }
    Ok(())
}

/// With the Rust versions of what it calls.
pub unsafe fn set_up_object_memory_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        set_up_object_memory(r, &mut |n, r| {
            match n {
                0 => allocate_object_memory_in_game()?.apply(r),
                _ => read_kind_records()?,
            }
            Ok(())
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { set_up_object_memory_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    if let Err(e) = unsafe { set_up_object_memory_in_game(r) } {
        *r = entry;
        return Err(e);
    }
    Ok(())
}

// ---- self-test ----

const LEN: usize = 0x0B;
const ENTRY_BYTES: [u8; 5] = [0xE8, 0x46, 0xC4, 0xFD, 0xFF];
const CALLS: [usize; 2] = [0x00, 0x05];

/// Each call: its callee's place, eax and ecx as it was entered with.
static CALLED: Mutex<Vec<[u32; 3]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([place, r.eax, r.ecx]);
    // A log call keeps the registers (the original wraps it in `pusha` and `popa`).
    if !LOGS.contains(&(place as usize)) {
        r.eax = r.eax.wrapping_mul(5).wrapping_add(place);
        r.ecx ^= 0x100 << place;
        r.edi = r.edi.wrapping_add(place + 1);
    }
}

/// The callees that are log calls.
const LOGS: [usize; 0] = [];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4433C9)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                CALLEES
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
    for eax in [0u32, 0x1234_5678, 0xFFFF_FFFF] {
        let run = |rust: bool| unsafe {
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs {
                eax,
                ..Regs::SENTINEL
            };
            if rust {
                let done = set_up_object_memory(&mut r, &mut |n, r| {
                    if LOGS.contains(&ORDER[n]) {
                        // The text's address is not the test's to compare.
                        act(ORDER[n] as u32, &mut { *r });
                    } else {
                        act(ORDER[n] as u32, r);
                    }
                    Ok(())
                });
                if done.is_err() {
                    r = Regs::default();
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            // A log call's eax is its text: only that it was made is compared.
            let called: Vec<[u32; 3]> = CALLED
                .lock()
                .unwrap()
                .iter()
                .map(|call| {
                    if LOGS.contains(&(call[0] as usize)) {
                        [call[0], 0, 0]
                    } else {
                        *call
                    }
                })
                .collect();
            ([r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp], called)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.1.len() == ORDER.len(), || {
            format!("eax {eax:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    t.finish()
}
