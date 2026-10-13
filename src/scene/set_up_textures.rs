//! `scene::set_up_textures` (0x44E7A5): the texture memory for everything that is
//! needed, and one texture more for the game's own use.
//!
//! In:  nothing
//! Out: carry clear; eax and ebx as `gfx::allocate_textures` leaves them
//!
//! `scene::build_texture_catalogue`; `dword_624234` = 0; then one system texture of
//! size code 3 and pixel kind 4 without mip levels from `gfx::allocate_textures`
//! (0x4051ED, Direct3D: not ported; eax = 0, ebx = 1, ecx = 3, edx = 4, ebp = 0), whose
//! number (ebx) goes to `dword_624234`. If that one is refused: fatal error
//! 0x80600943.
//! Two callers.
//! Shadow mode runs the original alone.
//! Manual patch with a private copy: the function begins with a call.
//! Self-test: a redirected copy whose two calls are the test's.

use crate::errors::{FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::build_texture_catalogue::build_texture_catalogue_in_game;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::set_up_textures", rva(0x44E7A5), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&[0x00, 0x2A, 0x2F])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const OWN_TEXTURE_RVA: usize = rva(0x624234); // dword_624234
const ALLOCATE_RVA: usize = rva(0x4051ED); // gfx::allocate_textures

const NO_OWN_TEXTURE: FatalError = FatalError {
    code: 0x8060_0943,
    message: "cant alloc sys video texture",
};

// ---- Rust version ----

/// `catalogue`: what stands for `scene::build_texture_catalogue`; `allocate`: for
/// `gfx::allocate_textures`, given the registers it is entered with, answering false
/// if it refuses.
pub unsafe fn set_up_textures(
    r: &mut Regs,
    catalogue: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
    allocate: &mut dyn FnMut(&mut Regs) -> bool,
) -> Result<(), Refused> {
    unsafe {
        catalogue(r)?;
        game_write(OWN_TEXTURE_RVA, 0u32);
        (r.eax, r.ebx, r.ecx, r.edx, r.ebp) = (0, 1, 3, 4, 0);
        if !allocate(r) {
            return Err(NO_OWN_TEXTURE.into());
        }
        game_write(OWN_TEXTURE_RVA, r.ebx);
        Ok(())
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
    let done = unsafe {
        set_up_textures(r, &mut |_| build_texture_catalogue_in_game(), &mut |r| {
            call_game(r, game_addr(ALLOCATE_RVA));
            !r.carry()
        })
    };
    match done {
        Ok(()) => r.set_carry(false),
        Err(e) => {
            *r = entry;
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
        }
    }
}

// ---- self-test ----

const LEN: usize = 0x3F;
const ENTRY_BYTES: [u8; 5] = [0xE8, 0xA1, 0x3F, 0xFD, 0xFF];
const CALLS: [usize; 2] = [0x00, 0x2A];
/// The operand of `jb loc_4D91C6` at 0x44E7D4.
const FATAL_JUMP: usize = 0x31;

static WENT_FATAL: AtomicU32 = AtomicU32::new(0);

#[unsafe(naked)]
unsafe extern "C" fn landing() {
    core::arch::naked_asm!(
        "mov dword ptr [{flag}], 1",
        "ret",
        flag = sym WENT_FATAL,
    )
}

/// Each call: its place, the own texture's number by then, and eax, ebx, ecx, edx, ebp.
static CALLED: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());
/// Whether the allocation is refused.
static REFUSE: AtomicU32 = AtomicU32::new(0);

fn handler(place: u32, r: &mut Regs) {
    let own: u32 = unsafe { game_read(OWN_TEXTURE_RVA) };
    CALLED
        .lock()
        .unwrap()
        .push([place, own, r.eax, r.ebx, r.ecx, r.edx, r.ebp]);
    if place == 0 {
        // The catalogue comes back with the carry clear and registers changed.
        r.set_carry(false);
        r.eax = 0x1111;
        r.edi = 0x2222;
    } else {
        r.set_carry(REFUSE.load(Ordering::SeqCst) != 0);
        r.eax = 0x30;
        r.ebx = 0x77;
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x44E7A5)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                if callee == rva(0x42274B) {
                    Some(stand_in(0))
                } else if callee == ALLOCATE_RVA {
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
    unsafe { copy.retarget(FATAL_JUMP, landing as *const () as usize) };
    let regions = [at(OWN_TEXTURE_RVA)];
    let saved = capture(&regions);
    for refuse in [0u32, 1] {
        let run = |rust: bool| unsafe {
            game_write(OWN_TEXTURE_RVA, 0xAAAA_AAAAu32);
            CALLED.lock().unwrap().clear();
            REFUSE.store(refuse, Ordering::SeqCst);
            WENT_FATAL.store(0, Ordering::SeqCst);
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            let failed = if rust {
                let done = set_up_textures(
                    &mut r,
                    &mut |r| {
                        act(0, r);
                        Ok(())
                    },
                    &mut |r| {
                        act(1, r);
                        !r.carry()
                    },
                );
                done.is_err() as u32
            } else {
                call_game(&mut r, copy.addr());
                WENT_FATAL.load(Ordering::SeqCst)
            };
            (
                failed,
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                game_read::<u32>(OWN_TEXTURE_RVA),
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.3.len() == 2, || {
            format!("refused {refuse}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
