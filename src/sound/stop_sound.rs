//! `sound::stop_sound` (0x40F1D1): plays a request's sound at volume 0.
//!
//! In:  esi = the request (its first dword is the sound; bit 0x8000: none); ecx, edx,
//!      edi, ebp go into the command as they are
//! Out: every register preserved
//! Unless the sound has bit 0x8000: `sound::play_sound` with eax the sound and ebx = 0.
//! That it stops the sound is a reading (a volume of 0).
//! 14 callers (the race's sounds). Shadow mode runs the original alone.
//! Self-test: a redirected copy whose `play_sound` notes its registers.
//! Refuses a null request.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::sound::play_sound::play_sound_in_game;
use crate::sound::queue::sent;
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::stop_sound", rva(0x40F1D1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The registers `sound::play_sound` is called with, or None if it is not called.
pub unsafe fn stop_sound(entry: &Regs) -> Result<Option<Regs>, BadInput> {
    let request = entry.esi as *const u32;
    if request.is_null() {
        return Err(BadInput("no request"));
    }
    let sound = unsafe { request.read_unaligned() };
    // Converted: `rc2k_core::sound::senders::stopped`.
    Ok(rc2k_core::sound::senders::stopped(sound).map(|sound| Regs {
        eax: sound,
        ebx: 0,
        ..*entry
    }))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { stop_sound(r) } {
        Ok(Some(with)) => play_sound_in_game(&with),
        Ok(None) => {}
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0x18;
const ENTRY_BYTES: [u8; 8] = [0x50, 0x53, 0x8B, 0x06, 0xA9, 0x00, 0x80, 0x00];
/// `sound::play_sound`.
const CALLS: [usize; 1] = [0x10];

static PLAYED: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    PLAYED.lock().unwrap().push(sent(r));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x40F1D1)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            Some(stand_in(0))
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    for sound in [
        0u32,
        5,
        0x1CC,
        0x7FFF,
        0x8000,
        0x8005,
        0xFFFF_7FFF,
        0xFFFF_FFFF,
    ] {
        let request = [sound, 0xAAAA_AAAA];
        let before = Regs {
            esi: request.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let run = |rust: bool| unsafe {
            PLAYED.lock().unwrap().clear();
            set_handler(handler);
            let preserved = if rust {
                if let Ok(Some(with)) = stop_sound(&before) {
                    PLAYED.lock().unwrap().push(sent(&with));
                }
                true
            } else {
                let mut ro = before;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&before, 0)
            };
            (preserved, request, PLAYED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!("sound {sound:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    t.finish()
}
