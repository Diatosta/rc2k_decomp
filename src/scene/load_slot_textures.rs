//! `scene::load_slot_textures` (0x425778): every texture slot in use gets the texture
//! it wants.
//!
//! In:  nothing
//! Out: ecx = the number of slots; everything else preserved
//!
//! `scene::load_slot_texture` for the slots 0 .. `dword_5DC970`.
//! Six callers.
//! Refuses no slots (the original goes through 2^32) and more than the table has.
//! Shadow mode runs the original alone (each slot is checked in its own hook).
//! Self-test: a redirected copy whose call notes the slot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    ECX, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
    shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::load_slot_texture::load_slot_texture_in_game;
use crate::scene::textures::{SLOT_COUNT_RVA, SLOTS};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::load_slot_textures", rva(0x425778), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

// Converted: `rc2k_core::scene::textures::slots_in_use`
pub unsafe fn load_slot_textures(
    load: &mut dyn FnMut(u32) -> Result<(), BadInput>,
) -> Result<Left, BadInput> {
    unsafe {
        let count = rc2k_core::scene::textures::slots_in_use(game_read(SLOT_COUNT_RVA))?;
        for slot in 0..count {
            load(slot)?;
        }
        Ok(Left::nothing().with(ECX, count))
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
    match unsafe { load_slot_textures(&mut |slot| load_slot_texture_in_game(slot, &entry)) } {
        Ok(left) => left.apply(r),
        Err(e) => {
            let e: Refused = e.into();
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
        }
    }
}

// ---- self-test ----

const LEN: usize = 0x16;
const ENTRY_BYTES: [u8; 5] = [0xB9, 0x00, 0x00, 0x00, 0x00];
const CALLS: [usize; 1] = [0x06];

static CALLED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push(r.ecx);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x425778)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x425ADE)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let inputs = [at(SLOT_COUNT_RVA)];
    let saved = capture(&inputs);
    for count in [1u32, 2, 9, 0x100, SLOTS] {
        let run = |rust: bool| unsafe {
            game_write(SLOT_COUNT_RVA, count);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs::SENTINEL;
            let mut r = before;
            if rust {
                let left = load_slot_textures(&mut |slot| {
                    act(
                        0,
                        &mut Regs {
                            ecx: slot,
                            ..before
                        },
                    );
                    Ok(())
                });
                match left {
                    Ok(left) => left.apply(&mut r),
                    Err(_) => r = Regs::default(),
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.1.len() == count as usize, || {
            format!(
                "{count:#x} slots: original {:x?}, {} calls; rust {:x?}, {} calls",
                theirs.0,
                theirs.1.len(),
                mine.0,
                mine.1.len()
            )
        });
    }
    restore(&inputs, &saved);
    t.finish()
}
