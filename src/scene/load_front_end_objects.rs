//! `scene::load_front_end_objects` (0x425CA9): loads the shapes of all 0x100 object
//! slots.
//!
//! In:  nothing
//! Out: eax = the faces' memory used, ecx = 0x100, edi = the end of the kinds' table,
//!      ebx = the last slot that had a kind (unchanged if none had)
//!
//! `scene::reset_objects`; `dword_631B68` and `dword_631F6C` = 0; for each slot its
//! kind from `dword_5CCE7C` to `dword_634FE8`, the slot to `dword_634FEC`, and
//! `scene::load_object`; then `dword_634FA4` = the points used, in bytes, and
//! `dword_634FA8` = the faces' memory used.
//! Two callers (the front end's start and a stage's leaving).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose two calls note themselves.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::load_object::load_object_in_game;
use crate::scene::objects::{
    FACE_CURSOR_RVA, KIND_RVA, OBJECT_KINDS_RVA, POINT_COUNT_RVA, SLOT_RVA, SLOTS,
};
use crate::scene::reset_objects::reset_objects;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::load_front_end_objects",
    rva(0x425CA9),
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

// ---- game data ----

const TEXTURE_COUNT_RVA: usize = rva(0x631B68); // dword_631B68
const NO_NORMALS_RVA: usize = rva(0x631F6C); // dword_631F6C
const SLOT_KINDS_RVA: usize = rva(0x5CCE7C); // dword_5CCE7C
/// `dword_634FA4` the points used in bytes, `dword_634FA8` the faces' memory used.
const USED_RVA: usize = rva(0x634FA4);

// ---- Rust version ----

/// `reset`: what stands for `scene::reset_objects`; `load`: for `scene::load_object`.
pub unsafe fn load_front_end_objects(
    r: &mut Regs,
    reset: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
    load: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        reset(r)?;
        game_write(TEXTURE_COUNT_RVA, 0u32);
        game_write(NO_NORMALS_RVA, 0u32);
        for slot in 0..SLOTS {
            r.ecx = slot;
            r.eax = game_read(SLOT_KINDS_RVA + slot as usize * 4);
            game_write(KIND_RVA, r.eax);
            game_write(SLOT_RVA, slot);
            load(r)?;
        }
        r.ecx = SLOTS;
        let cursor: u32 = game_read(FACE_CURSOR_RVA);
        game_write(
            USED_RVA,
            [game_read::<u32>(POINT_COUNT_RVA).wrapping_mul(12), cursor],
        );
        r.eax = cursor;
        Ok(())
    }
}

// ---- hook ----

/// With the Rust versions of its two callees, for Rust callers.
pub unsafe fn load_front_end_objects_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        load_front_end_objects(
            r,
            &mut |r| {
                reset_objects()?.apply(r);
                Ok(())
            },
            &mut |r| load_object_in_game(r),
        )
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    let done = unsafe {
        load_front_end_objects(
            r,
            &mut |r| {
                reset_objects()?.apply(r);
                Ok(())
            },
            &mut |r| load_object_in_game(r),
        )
    };
    if let Err(e) = done {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x56;
const ENTRY_BYTES: [u8; 5] = [0xE8, 0x51, 0x00, 0x00, 0x00];
const CALLS: [usize; 2] = [0x00, 0x30];

/// Each call: its place, the kind and the slot by then, ecx.
static CALLED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    unsafe {
        CALLED
            .lock()
            .unwrap()
            .push([place, game_read(KIND_RVA), game_read(SLOT_RVA), r.ecx]);
        if place == 0 {
            // As the real one leaves them.
            r.eax = u32::MAX;
            r.ecx = 0;
            r.edi = game_addr(OBJECT_KINDS_RVA) as u32 + SLOTS * 4;
        } else if game_read::<u32>(KIND_RVA) != 0 {
            r.eax = game_read(KIND_RVA);
            r.ebx = game_read(SLOT_RVA);
            // A car's loading leaves another slot behind.
            game_write(SLOT_RVA, 0x5F_u32);
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x425CA9)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                if callee == rva(0x425CFF) {
                    Some(stand_in(0))
                } else if callee == rva(0x425D87) {
                    Some(stand_in(1))
                } else {
                    None
                }
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 5] = [
        at(TEXTURE_COUNT_RVA),
        at(NO_NORMALS_RVA),
        at(KIND_RVA),
        at(SLOT_RVA),
        (game_addr(USED_RVA) as u32, 8),
    ];
    let inputs: [Region; 3] = [
        (game_addr(SLOT_KINDS_RVA) as u32, SLOTS as usize * 4),
        at(FACE_CURSOR_RVA),
        at(POINT_COUNT_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    for case in 0..4u32 {
        let kinds: Vec<u32> = (0..SLOTS)
            .map(|slot| match case {
                0 => 0,
                1 => 0x200 + slot,
                _ => (slot.wrapping_mul(0x9E37_79B1) >> (24 + case)) * (slot % (case + 1)),
            })
            .collect();
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            std::ptr::copy_nonoverlapping(
                kinds.as_ptr() as *const u8,
                game_addr(SLOT_KINDS_RVA) as *mut u8,
                kinds.len() * 4,
            );
            game_write(FACE_CURSOR_RVA, 0x1230u32 + case);
            game_write(POINT_COUNT_RVA, 0x77u32 << (case * 8));
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                let done = load_front_end_objects(
                    &mut r,
                    &mut |r| {
                        act(0, r);
                        Ok(())
                    },
                    &mut |r| {
                        act(1, r);
                        Ok(())
                    },
                );
                if done.is_err() {
                    r = Regs::default();
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                capture(&regions),
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.2.len() == 0x101, || {
            let first = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
            format!(
                "case {case}: original regs {:x?}, {} calls; rust regs {:x?}, {} calls; memory same {}; the first call that differs {first:?}",
                theirs.0,
                theirs.2.len(),
                mine.0,
                mine.2.len(),
                theirs.1 == mine.1
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
