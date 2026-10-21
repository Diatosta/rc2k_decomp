//! `sound::step_car_stream` (0x46F7AD): the car's sound stream goes on to its next part
//! when one has ended.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! Only with `dword_7556D0` at 2 and `dword_A8C188` at 1, and while the stream is not
//! playing (`sub_40C4D5`: `dword_506960` is not 1; one other caller): `dword_A8C180` + 1
//! and the part at that place of the list `dword_A8C184`; -1 ends it
//! (`dword_A8C188` = 0), else `sound::start_stream` with it and `dword_755698`. Once a
//! frame on four screens (the car specification, the slideshow, two stage screens).
//! Shadow mode runs the original alone (it sends a command).
//! Self-test: a redirected copy whose playing question and start are the test's, over
//! lists of the test's.

use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::step_car_stream", rva(0x46F7AD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const KIND_RVA: usize = rva(0x7556D0); // dword_7556D0
const ON_RVA: usize = rva(0xA8C188); // dword_A8C188
const STREAM_RVA: usize = rva(0xA8C184); // dword_A8C184
const PLACE_RVA: usize = rva(0xA8C180); // dword_A8C180
const VOLUME_RVA: usize = rva(0x755698); // dword_755698
const PLAYING_RVA: usize = rva(0x506960); // dword_506960
const START_RVA: usize = rva(0x40F743); // sound::start_stream
const ASK_RVA: usize = rva(0x40C4D5); // sub_40C4D5

// ---- Rust version ----

/// `playing` stands for `sub_40C4D5`, `start` for `sound::start_stream`.
pub unsafe fn step_car_stream(
    r: &Regs,
    playing: &mut dyn FnMut() -> bool,
    start: &mut dyn FnMut(&mut Regs),
) {
    unsafe {
        // Converted: `rc2k_core::sound::race::stream_step`.
        use rc2k_core::sound::race::{StreamStep, stream_step};
        let list: u32 = game_read(STREAM_RVA);
        let (place, part) = match stream_step(
            game_read(KIND_RVA),
            game_read(ON_RVA),
            playing,
            game_read(PLACE_RVA),
            |place| ((list.wrapping_add(place.wrapping_mul(4))) as *const u32).read_unaligned(),
        ) {
            StreamStep::Nothing => return,
            StreamStep::Ended { place } => {
                game_write(PLACE_RVA, place);
                game_write(ON_RVA, 0u32);
                return;
            }
            StreamStep::Start { place, part } => (place, part),
        };
        game_write(PLACE_RVA, place);
        let mut inner = Regs {
            eax: part,
            ebx: place,
            ecx: game_read(VOLUME_RVA),
            ..*r
        };
        start(&mut inner);
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe {
        step_car_stream(r, &mut || game_read::<u32>(PLAYING_RVA) == 1, &mut |r| {
            call_game(r, game_addr(START_RVA))
        })
    };
}

// ---- self-test ----

const LEN: usize = 0x50;
const ENTRY_BYTES: [u8; 6] = [0x60, 0x83, 0x3D, 0xD0, 0x56, 0x75];
const CALLS: [usize; 2] = [0x13, 0x48];

static CALLED: Mutex<Vec<[u32; 2]>> = Mutex::new(Vec::new());
static PLAYING: AtomicBool = AtomicBool::new(false);

fn handler(place: u32, r: &mut Regs) {
    if place == 0 {
        CALLED.lock().unwrap().push([r.eax, r.ecx]);
    } else {
        r.set_zero(PLAYING.load(Ordering::SeqCst));
    }
}

fn written() -> [Region; 2] {
    [at(ON_RVA), at(PLACE_RVA)]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x46F7AD)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                [START_RVA, ASK_RVA]
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
    let inputs = [at(KIND_RVA), at(STREAM_RVA), at(VOLUME_RVA)];
    let saved = (capture(&inputs), capture(&written()));
    set_handler(handler);
    let list = [0x11u32, 0x22, 0x33, u32::MAX, 0x44];
    for kind in [2u32, 0x13] {
        for on in [1u32, 0] {
            for playing in [false, true] {
                for place in [0u32, 1, 2] {
                    let run = |rust: bool| unsafe {
                        game_write(KIND_RVA, kind);
                        game_write(ON_RVA, on);
                        game_write(PLACE_RVA, place);
                        game_write(STREAM_RVA, list.as_ptr() as u32);
                        game_write(VOLUME_RVA, 0x55u32);
                        PLAYING.store(playing, Ordering::SeqCst);
                        CALLED.lock().unwrap().clear();
                        let before = Regs::SENTINEL;
                        let mut r = before;
                        let kept = if rust {
                            step_car_stream(
                                &r,
                                &mut || {
                                    let mut q = before;
                                    act(1, &mut q);
                                    q.eflags & 0x40 != 0
                                },
                                &mut |r| act(0, r),
                            );
                            true
                        } else {
                            call_game(&mut r, copy.addr());
                            r.preserved_except(&before, 0)
                        };
                        (kept, capture(&written()), CALLED.lock().unwrap().clone())
                    };
                    let (theirs, mine) = (run(false), run(true));
                    t.check(theirs == mine && theirs.0, || {
                        format!(
                            "kind {kind:#x}, on {on}, playing {playing}, place {place}: original {theirs:x?}; rust {mine:x?}"
                        )
                    });
                }
            }
        }
    }
    restore(&inputs, &saved.0);
    restore(&written(), &saved.1);
    t.finish()
}
