//! `scene::load_wanted_textures` (0x4256C8): works out what every texture slot should hold and loads what is missing.
//!
//! In:  nothing
//! Out: as `scene::load_slot_textures` leaves them
//!
//! `scene::assign_kind_textures`, `scene::map_texture_places`,
//! `scene::load_slot_textures`.
//! One caller, `scene::reload_slot_textures`.
//! Manual patch with a private copy: the function begins with a call.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls note themselves.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::assign_kind_textures::assign_kind_textures;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::load_slot_textures::load_slot_textures;
use crate::scene::map_texture_places::map_texture_places;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::load_wanted_textures",
    rva(0x4256C8),
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
const CALLEES: [usize; 3] = [rva(0x4256D8), rva(0x42570D), rva(0x425778)];
/// The calls made, as places in `CALLEES`.
const ORDER: [usize; 3] = [0, 1, 2];

// ---- Rust version ----

/// `step`: what stands for each call, by its place in `ORDER`; it is entered with the
/// registers the call before left.
pub unsafe fn load_wanted_textures(
    r: &mut Regs,
    step: &mut dyn FnMut(usize, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    for n in 0..ORDER.len() {
        step(n, r)?;
    }
    Ok(())
}

/// With the Rust versions of what it calls.
pub unsafe fn load_wanted_textures_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        load_wanted_textures(r, &mut |n, r| {
            match n {
                0 => assign_kind_textures()?.apply(r),
                1 => map_texture_places().apply(r),
                _ => {
                    let entry = *r;
                    load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry))?
                        .apply(r)
                }
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
    if let Err(e) = unsafe { load_wanted_textures_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    if let Err(e) = unsafe { load_wanted_textures_in_game(r) } {
        *r = entry;
        return Err(e);
    }
    Ok(())
}

// ---- self-test ----

const LEN: usize = 0x10;
const ENTRY_BYTES: [u8; 5] = [0xE8, 0x0B, 0x00, 0x00, 0x00];
const CALLS: [usize; 3] = [0x00, 0x05, 0x0A];

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
            game_addr(rva(0x4256C8)),
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
                let done = load_wanted_textures(&mut r, &mut |n, r| {
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
