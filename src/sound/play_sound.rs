//! `sound::play_sound` (0x409B36): sends the audio thread a sound to play (command 1).
//!
//! In:  eax sound, ebx volume, edx; ecx, edi, ebp the sound's place when the mode is 3D
//! Out: every register preserved
//! Nothing while sounds are held back (`dword_506C84` is 1; `sub_409B6B` sets it,
//! `sub_409B76` clears it). Unless the sound mode in use (`dword_4F82BC`) has bit
//! 0x100000 (the 3D modes), the command goes with esi, edi and ebp = 0xFF800000; with
//! it, with esi = ecx and ecx = 0.
//! 14 callers. Shadow mode runs the original alone (a command must not be sent twice).
//! Self-test: a redirected copy whose two variables are the test's and whose command
//! notes its registers.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::sound::queue::{Queue, send_command, sent};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_sound", rva(0x409B36), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const HELD_BACK_RVA: usize = rva(0x506C84); // dword_506C84
const MODE_RVA: usize = rva(0x4F82BC); // dword_4F82BC
const PLAY_1: u32 = 1;
const MODE_3D: u32 = 0x10_0000;
const NO_PLACE: u32 = 0xFF80_0000;

// ---- Rust version ----

/// The registers command 1 is sent with, or None if nothing is sent.
pub fn play_sound(entry: &Regs, held_back: u32, mode: u32) -> Option<Regs> {
    if held_back == 1 {
        return None;
    }
    Some(if mode & MODE_3D == 0 {
        Regs {
            esi: NO_PLACE,
            edi: NO_PLACE,
            ebp: NO_PLACE,
            ..*entry
        }
    } else {
        Regs {
            esi: entry.ecx,
            ecx: 0,
            ..*entry
        }
    })
}

/// With the game's variables and queue: what other Rust code calls.
pub fn play_sound_in_game(entry: &Regs) {
    let (held_back, mode) = unsafe { (game_read(HELD_BACK_RVA), game_read(MODE_RVA)) };
    if let Some(with) = play_sound(entry, held_back, mode) {
        unsafe { send_command(&Queue::game(), PLAY_1, sent(&with)) };
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    play_sound_in_game(r);
}

// ---- self-test ----

const LEN: usize = 0x35;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0x84, 0x6C, 0x50, 0x00, 0x01];
/// `sound::send_command`.
const CALLS: [usize; 1] = [0x2D];
/// Where the code names the two variables.
const HELD_BACK_AT: usize = 0x03;
const MODE_AT: usize = 0x0C;

static SENT: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    SENT.lock().unwrap().push(sent(r));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x409B36)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            Some(stand_in(0))
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let vars = Box::new([0u32; 2]);
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let names = |at: usize, var: usize| {
            u32::from_le_bytes(code[at..at + 4].try_into().unwrap()) == game_addr(var) as u32
        };
        if !names(HELD_BACK_AT, HELD_BACK_RVA) || !names(MODE_AT, MODE_RVA) {
            return t.skip("the function does not name its variables where the test expects");
        }
        copy.patch_u32(HELD_BACK_AT, &raw const vars[0] as u32);
        copy.patch_u32(MODE_AT, &raw const vars[1] as u32);
    }
    let vars = Box::into_raw(vars);
    for held_back in [0u32, 1, 2] {
        for mode in [
            0u32,
            0x11221,
            0x0244_1220,
            0x10_1220,
            0xFFFF_FFFF,
            0xFFEF_FFFF,
        ] {
            let before = Regs::SENTINEL;
            let run = |rust: bool| unsafe {
                vars.write([held_back, mode]);
                SENT.lock().unwrap().clear();
                set_handler(handler);
                let preserved = if rust {
                    if let Some(with) = play_sound(&before, held_back, mode) {
                        SENT.lock().unwrap().push(sent(&with));
                    }
                    true
                } else {
                    let mut ro = before;
                    call_game(&mut ro, copy.addr());
                    ro.preserved_except(&before, 0)
                };
                (preserved, SENT.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0, || {
                format!(
                    "held back {held_back}, mode {mode:#x}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    drop(unsafe { Box::from_raw(vars) });
    t.finish()
}
