//! `scene::load_slot_texture` (0x425ADE): gives a texture slot the texture it wants,
//! if it holds another.
//!
//! In:  ecx = the slot
//! Out: every register preserved
//!
//! Nothing if the slot wants none (`dword_5DA968[slot]` is -1) or holds what it wants
//! (`dword_5DB968[slot]`). Else `scene::load_texture` (0x44E778, which is only
//! `sub_41A8F9` of the Direct3D layer: not ported) with eax = the texture and ebx =
//! the slot, `dword_6415F8` + 1, and the slot holds it.
//! One caller, `scene::load_slot_textures`.
//! Refuses a slot past the table.
//! Shadow mode runs the original alone and checks what the slot holds afterwards.
//! Self-test: a redirected copy whose loading notes its registers.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::textures::{LOADED_RVA, LOADS_RVA, SLOTS, WANTED_RVA, entry};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::load_slot_texture", rva(0x425ADE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const LOAD_RVA: usize = rva(0x44E778); // scene::load_texture

// ---- Rust version ----

/// `load`: what stands for `scene::load_texture`, given the texture and the slot.
// Converted: `rc2k_core::scene::textures::slot_load`
pub unsafe fn load_slot_texture(slot: u32, load: &mut dyn FnMut(u32, u32)) -> Result<(), BadInput> {
    unsafe {
        let Some(wanted) = rc2k_core::scene::textures::slot_load(
            slot,
            || entry(WANTED_RVA, slot),
            || entry(LOADED_RVA, slot),
        )?
        else {
            return Ok(());
        };
        load(wanted, slot);
        game_write(LOADS_RVA, game_read::<u32>(LOADS_RVA).wrapping_add(1));
        game_write(LOADED_RVA + slot as usize * 4, wanted);
        Ok(())
    }
}

/// With the game's loading, entered with the registers the original has there.
pub unsafe fn load_slot_texture_in_game(slot: u32, entry_regs: &Regs) -> Result<(), BadInput> {
    unsafe {
        load_slot_texture(slot, &mut |texture, slot| {
            let mut r = Regs {
                eax: texture,
                ebx: slot,
                ecx: slot,
                ..*entry_regs
            };
            crate::calls::call(LOAD_RVA, &mut r);
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        let slot = r.ecx;
        let wanted = (slot < SLOTS).then(|| unsafe { entry(WANTED_RVA, slot) });
        unsafe { call_original(r) };
        if let Some(wanted) = wanted.filter(|&wanted| wanted != u32::MAX) {
            let holds = unsafe { entry(LOADED_RVA, slot) };
            if holds != wanted {
                HOOK.mismatch(|| {
                    format!("slot {slot:#x} wanted {wanted:#x}: it holds {holds:#x} afterwards")
                });
            }
        }
        return;
    }
    let entry_regs = *r;
    if let Err(e) = unsafe { load_slot_texture_in_game(r.ecx, &entry_regs) } {
        let e: Refused = e.into();
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x46;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x8B, 0x04, 0x8D, 0x68, 0xA9, 0x5D, 0x00];
const CALLS: [usize; 1] = [0x31];

/// Each loading: eax, ebx, ecx, and the two counters by then.
static CALLED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());
const SLOT: u32 = 0x3F7;

fn handler(_: u32, r: &mut Regs) {
    unsafe {
        CALLED.lock().unwrap().push([
            r.eax,
            r.ebx,
            r.ecx,
            game_read(LOADS_RVA),
            entry(LOADED_RVA, SLOT),
        ]);
    }
    r.eax = 0x5555;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x425ADE)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == LOAD_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 3] = [
        at(LOADS_RVA),
        at(LOADED_RVA + SLOT as usize * 4),
        at(WANTED_RVA + SLOT as usize * 4),
    ];
    let saved = capture(&regions);
    for (wanted, holds) in [
        (u32::MAX, 5u32),
        (u32::MAX, u32::MAX),
        (7, 7),
        (7, 8),
        (0, u32::MAX),
        (0x1234, 0),
        (0xFFFF_FFFE, 0xFFFF_FFFF),
    ] {
        let run = |rust: bool| unsafe {
            game_write(LOADS_RVA, 0xFFFF_FFFFu32.wrapping_sub(wanted & 1));
            game_write(LOADED_RVA + SLOT as usize * 4, holds);
            game_write(WANTED_RVA + SLOT as usize * 4, wanted);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs {
                ecx: SLOT,
                ..Regs::SENTINEL
            };
            let kept = if rust {
                load_slot_texture(SLOT, &mut |texture, slot| {
                    act(
                        0,
                        &mut Regs {
                            eax: texture,
                            ebx: slot,
                            ecx: slot,
                            ..before
                        },
                    )
                })
                .is_ok()
            } else {
                let mut r = before;
                call_game(&mut r, copy.addr());
                r.preserved_except(&before, 0)
            };
            (kept, capture(&regions), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!("wanted {wanted:#x}, holds {holds:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
