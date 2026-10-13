//! `scene::reload_slot_textures` (0x425669): every texture slot is loaded anew ("reload video plates").
//!
//! In:  nothing
//! Out: as `scene::load_wanted_textures` leaves them
//!
//! Two lines in the game's log (a rule and "reload video plates - start");
//! `scene::clear_slot_textures`; `scene::load_wanted_textures`; two lines more
//! ("reload video plates - finis" and a rule).
//! Two callers (the front end's 3D set-up and a race's).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note themselves.

use crate::errors::Refused;
use crate::framework::game_ptr;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::logfile::log_write::log_write;
use crate::redirect::RedirectedCopy;
use crate::scene::clear_slot_textures::clear_slot_textures;
use crate::scene::load_wanted_textures::load_wanted_textures_in_game;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::reload_slot_textures",
    rva(0x425669),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// What it calls, in the order of their first calls.
const CALLEES: [usize; 3] = [rva(0x40142B), rva(0x4256A8), rva(0x4256C8)];
/// The calls made, as places in `CALLEES`.
const ORDER: [usize; 6] = [0, 0, 1, 2, 0, 0];

// ---- Rust version ----

/// `step`: what stands for each call, by its place in `ORDER`; it is entered with the
/// registers the call before left.
pub unsafe fn reload_slot_textures(
    r: &mut Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    for n in 0..ORDER.len() {
        step(n, r)?;
    }
    Ok(())
}

/// With the Rust versions of what it calls.
pub unsafe fn reload_slot_textures_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        reload_slot_textures(r, &mut |n, r| {
            match n {
                0 => log_write(game_ptr(rva(0x4DB858)), true, 1),
                1 => log_write(game_ptr(rva(0x4DB87D)), true, 1),
                2 => clear_slot_textures().apply(r),
                3 => load_wanted_textures_in_game(r)?,
                4 => log_write(game_ptr(rva(0x4DB899)), true, 1),
                _ => log_write(game_ptr(rva(0x4DB8B5)), true, 1),
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
    if let Err(e) = unsafe { reload_slot_textures_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x3F;
const ENTRY_BYTES: [u8; 7] = [0x60, 0x8D, 0x05, 0x58, 0xB8, 0x4D, 0x00];
const CALLS: [usize; 6] = [0x07, 0x14, 0x1A, 0x1F, 0x2B, 0x38];

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
const LOGS: [usize; 1] = [0];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x425669)),
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
                let done = reload_slot_textures(&mut r, &mut |n, r| {
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
