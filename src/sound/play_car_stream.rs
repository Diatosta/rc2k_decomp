//! `sound::play_car_stream` (0x469AA1): the car shown's sound stream started.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The car (as `menu::load_car_background` finds it, 0x21 counting as 0x17):
//! `dword_A8C188` = 1, `dword_A8C184` = `off_75A31E[car]`, `dword_A8C180` = 0, and
//! `sound::start_stream` with eax = the first dword there and ecx = `dword_755698`.
//! Five call sites, next to `load_car_background`'s.
//! Shadow mode runs the original alone (it sends a command).
//! Refuses what the record's lookup refuses.
//! Self-test: a redirected copy whose `start_stream` notes what it gets and what was set
//! by then, every page with records of the test's.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::menu::load_car_background::shown_car;
use crate::redirect::RedirectedCopy;
use crate::screens::car_caption::{TEST_CARS, TestRecords};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("sound::play_car_stream", rva(0x469AA1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const CAR_RVA: usize = rva(0x755990); // dword_755990
const STREAMS_RVA: usize = rva(0x75A31E); // off_75A31E
const ON_RVA: usize = rva(0xA8C188); // dword_A8C188
const STREAM_RVA: usize = rva(0xA8C184); // dword_A8C184
const PLACE_RVA: usize = rva(0xA8C180); // dword_A8C180
const VOLUME_RVA: usize = rva(0x755698); // dword_755698
const START_RVA: usize = rva(0x40F743); // sound::start_stream

// ---- Rust version ----

/// `start` gets the registers `sound::start_stream` is called with.
pub unsafe fn play_car_stream(r: &Regs, start: &mut dyn FnMut(&mut Regs)) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::set_up::car_stream`.
    unsafe {
        let car = shown_car()?;
        let s = rc2k_core::sound::set_up::car_stream(
            game_read(STREAMS_RVA + car as usize * 4),
            |list| (list as *const u32).read_unaligned(),
            game_read(VOLUME_RVA),
        );
        game_write(ON_RVA, s.on);
        game_write(STREAM_RVA, s.list);
        game_write(PLACE_RVA, s.place);
        let mut inner = Regs {
            eax: s.first,
            ecx: s.volume,
            ..*r
        };
        start(&mut inner);
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
    let done = unsafe { play_car_stream(r, &mut |r| call_game(r, game_addr(START_RVA))) };
    if let Err(e) = done {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x5C;
const ENTRY_BYTES: [u8; 7] = [0x60, 0x8B, 0x1D, 0x98, 0x59, 0x75, 0x00];
const CALLS: [usize; 1] = [0x469AF6 - 0x469AA1];

static CALLED: Mutex<Vec<(u32, u32, Vec<u8>)>> = Mutex::new(Vec::new());

fn written() -> [Region; 3] {
    [at(ON_RVA), at(STREAM_RVA), at(PLACE_RVA)]
}

fn handler(_: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push((r.eax, r.ecx, capture(&written())));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x469AA1)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == START_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let inputs = [at(CAR_RVA)];
    let saved = (capture(&inputs), capture(&written()));
    set_handler(handler);
    let records = TestRecords::new(&TEST_CARS, &[0x249]);
    for car in 0..records.len() {
        let run = |rust: bool| unsafe {
            game_write(CAR_RVA, car);
            restore(&written(), &[0xAA; 12]);
            CALLED.lock().unwrap().clear();
            let before = Regs::SENTINEL;
            let mut r = before;
            let kept = if rust {
                play_car_stream(&r, &mut |r| act(0, r)).is_ok()
            } else {
                call_game(&mut r, copy.addr());
                r.preserved_except(&before, 0)
            };
            (kept, capture(&written()), CALLED.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            format!("entry {car}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(records);
    restore(&inputs, &saved.0);
    restore(&written(), &saved.1);
    t.finish()
}
