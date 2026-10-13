//! `sound::play_menu_sound` (0x40F059): plays one of the menus' sounds.
//!
//! In:  eax = which, as bits; esi goes into the command as it is
//! Out: every register preserved
//! Nothing for eax = 0, and nothing before the time kept in `dword_511B0C` (125 ms after
//! the last sound; compared by the sign of the difference). The time is moved on before
//! the bits are looked at. The first bit set of these, in this order, picks the sound
//! and the volume it is played at:
//! 0x80 sound 0xE4 at `dword_755694`; 0x10, 0x20, 0x40 sounds 0x163, 0x197, 0x1CB at
//! `dword_7554E8`; 0x100, 8, 2, 4, 1 sounds 0x1CC, 0x1CA, 0x1C8, 0x1C9, 0x1C7 at
//! `dword_755690`. Then `sound::play_sound` with edx = 0xFFFFFF91 and ecx, edi, ebp = 0.
//! 25 callers (1 is the highlight's click, 2 Enter's sound: `menu::handle_keys`).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose clock and `play_sound` are the test's.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{at, capture, restore};
use crate::sound::play_sound::play_sound_in_game;
use crate::sound::queue::sent;
use crate::system::tick_count::tick_count;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_menu_sound", rva(0x40F059), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const NEXT_TIME_RVA: usize = rva(0x511B0C); // dword_511B0C
const VOLUME_A_RVA: usize = rva(0x755694); // dword_755694
const VOLUME_B_RVA: usize = rva(0x7554E8); // dword_7554E8
const VOLUME_C_RVA: usize = rva(0x755690); // dword_755690

/// In the order they are looked at: the bit, the sound, the volume's variable.
const SOUNDS: [(u32, u32, usize); 9] = [
    (0x80, 0xE4, VOLUME_A_RVA),
    (0x10, 0x163, VOLUME_B_RVA),
    (0x20, 0x197, VOLUME_B_RVA),
    (0x40, 0x1CB, VOLUME_B_RVA),
    (0x100, 0x1CC, VOLUME_C_RVA),
    (0x08, 0x1CA, VOLUME_C_RVA),
    (0x02, 0x1C8, VOLUME_C_RVA),
    (0x04, 0x1C9, VOLUME_C_RVA),
    (0x01, 0x1C7, VOLUME_C_RVA),
];

// ---- Rust version ----

/// `now` is asked for the clock; `play` gets the registers `sound::play_sound` is
/// called with.
pub unsafe fn play_menu_sound(
    entry: &Regs,
    now: &mut dyn FnMut() -> u32,
    play: &mut dyn FnMut(&Regs),
) {
    unsafe {
        let which = entry.eax;
        if which == 0 {
            return;
        }
        let now = now();
        if (now.wrapping_sub(game_read::<u32>(NEXT_TIME_RVA)) as i32) < 0 {
            return;
        }
        game_write(NEXT_TIME_RVA, now.wrapping_add(0x7D));
        let Some(&(_, sound, volume)) = SOUNDS.iter().find(|&&(bit, ..)| which & bit != 0) else {
            return;
        };
        play(&Regs {
            eax: sound,
            ebx: game_read(volume),
            ecx: 0,
            edx: 0xFFFF_FF91,
            edi: 0,
            ebp: 0,
            ..*entry
        });
    }
}

/// With the game's clock and queue: what other Rust code calls.
pub fn play_menu_sound_in_game(entry: &Regs) {
    unsafe { play_menu_sound(entry, &mut tick_count, &mut |with| play_sound_in_game(with)) };
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    play_menu_sound_in_game(r);
}

// ---- self-test ----

const LEN: usize = 0xC3;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x85, 0xC0, 0x0F, 0x84, 0xB8, 0x00, 0x00];
/// `system::tick_count` and `sound::play_sound`.
const CALLS: [usize; 2] = [0x0B, 0xBC];
const CLOCK_RVA: usize = rva(0x411024);

/// The clock the test gives, and what `play_sound` was called with.
static CLOCK: AtomicU32 = AtomicU32::new(0);
static PLAYED: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    if place == 0 {
        r.eax = CLOCK.load(Ordering::SeqCst);
    } else {
        PLAYED.lock().unwrap().push(sent(r));
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x40F059)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| Some(stand_in((callee != CLOCK_RVA) as u32)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = [at(NEXT_TIME_RVA)];
    let inputs = [at(VOLUME_A_RVA), at(VOLUME_B_RVA), at(VOLUME_C_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    unsafe {
        game_write(VOLUME_A_RVA, 0x0A0Au32);
        game_write(VOLUME_B_RVA, 0x0B0Bu32);
        game_write(VOLUME_C_RVA, 0x0C0Cu32);
    }
    let mut which: Vec<u32> = (0..10).map(|bit| 1 << bit).collect();
    which.extend([
        0,
        0x1FF,
        0x7F,
        0x10F,
        0x0E,
        0x200,
        0xFFFF_FE00,
        0xFFFF_FFFF,
        0x106,
    ]);
    // (the time kept, the clock)
    let times = [
        (0u32, 0u32),
        (1000, 999),
        (1000, 1000),
        (1000, 5000),
        (0xFFFF_FFF0, 0x10),
        (0x10, 0xFFFF_FFF0),
        (0x7FFF_FFFF, 0),
    ];
    for &eax in &which {
        for &(kept, clock) in &times {
            let before = Regs {
                eax,
                ..Regs::SENTINEL
            };
            let run = |rust: bool| unsafe {
                game_write(NEXT_TIME_RVA, kept);
                CLOCK.store(clock, Ordering::SeqCst);
                PLAYED.lock().unwrap().clear();
                set_handler(handler);
                let preserved = if rust {
                    play_menu_sound(&before, &mut || CLOCK.load(Ordering::SeqCst), &mut |with| {
                        PLAYED.lock().unwrap().push(sent(with))
                    });
                    true
                } else {
                    let mut ro = before;
                    call_game(&mut ro, copy.addr());
                    ro.preserved_except(&before, 0)
                };
                (preserved, capture(&regions), PLAYED.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0, || {
                format!(
                    "eax {eax:#x}, time kept {kept:#x}, clock {clock:#x}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
