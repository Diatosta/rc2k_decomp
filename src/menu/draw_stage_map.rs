//! `menu::draw_stage_map` (0x4683CF): the UK map with the stage, as the stage screens'
//! background.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/stage_choice.rs`. 14 callers (the stage choice's settings, the stage
//! screens' backgrounds). The map that cannot be loaded raises fatal error 0x80712345.
//! Refuses a choice without its tables. Shadow mode runs the original alone. Self-test: a
//! redirected copy whose five calls go to stand-ins noting the registers (the record's
//! search giving a record of the game's, the loading the carry of the case) and whose jump
//! to the fatal stub lands in `fatal_landing`, on the game's stage tables, 3 modes.

use crate::errors::{FatalError, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{WENT_FATAL, fatal_landing, set_handler, stand_in};
use crate::menu::stage_choice::*;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::draw_stage_map", rva(0x4683CF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { draw_stage_map(r, &mut game_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0xB3;
const ENTRY: [u8; 8] = [0x60, 0x83, 0x3D, 0x00, 0x57, 0x75, 0x00, 0x02];
const CALLS: [usize; 5] = [0x64, 0x79, 0x84, 0x98, 0xAC];
// records::find_record, gfx::load_picture_with_overlay, convert_picture, picture_as_surface,
// blit_glyph.
const CALLEES: [usize; 5] = [0x47255D, 0x4118D1, 0x406126, 0x405FE7, 0x4069B2];
const JB_REL: usize = 0x80;

static NOTED: Mutex<Vec<[u32; 7]>> = Mutex::new(Vec::new());
static RECORD: Mutex<u32> = Mutex::new(0);
static LOAD_FAILS: AtomicBool = AtomicBool::new(false);

fn handler(place: u32, r: &mut Regs) {
    NOTED
        .lock()
        .unwrap()
        .push([place, r.eax, r.ebx, r.edi, r.esi, r.ecx, r.edx]);
    match place {
        0 => r.edi = *RECORD.lock().unwrap(),
        1 => r.set_carry(LOAD_FAILS.load(Ordering::SeqCst)),
        _ => {}
    }
}

fn inputs() -> Vec<Region> {
    [
        MODE_RVA,
        GROUP_RVA,
        SUB_RVA,
        NUMBER_RVA,
        rva(0x7587EC),
        rva(0x7587F0),
        rva(0x75595C),
        rva(0x755958),
        rva(0x4F49CC),
    ]
    .map(at)
    .to_vec()
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The callees as IDA shows them, checked against the copy's calls.
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x4683CF)), LEN, &ENTRY, &CALLS, &|callee| {
            CALLEES
                .iter()
                .position(|&c| rva(c) == callee)
                .map(|n| stand_in(n as u32))
        })
    };
    let copy = match copy {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    unsafe { copy.retarget(JB_REL, fatal_landing as *const () as usize) };
    let inputs = inputs();
    let saved = capture(&inputs);
    set_handler(handler);
    let chosen = unsafe { crate::screens::draw_stage_records::selection() };
    let championship = unsafe { (game_addr(rva(0x73555A)) as *const u32).read_unaligned() };
    let Some(chosen) = chosen else {
        restore(&inputs, &saved);
        return t.skip("no stage tables for the choice");
    };
    for mode in [0u32, 2, 3] {
        if mode == 2 && championship == 0 {
            continue;
        }
        for fails in [false, true] {
            let run = |rust: bool| {
                restore(&inputs, &saved);
                unsafe {
                    game_write(MODE_RVA, mode);
                    game_write(rva(0x7587EC), 0u32);
                    game_write(rva(0x7587F0), 1u32);
                    game_write(rva(0x75595C), 2u32);
                    game_write(rva(0x755958), 3u32);
                }
                *RECORD.lock().unwrap() = chosen.record;
                LOAD_FAILS.store(fails, Ordering::SeqCst);
                NOTED.lock().unwrap().clear();
                WENT_FATAL.store(0, Ordering::SeqCst);
                let before = Regs::SENTINEL;
                let end = if rust {
                    match unsafe {
                        draw_stage_map(&before, &mut |p, r| {
                            handler(p, r);
                            Ok(())
                        })
                    } {
                        Ok(()) => Some(false),
                        Err(Refused::Fatal(FatalError {
                            code: 0x8071_2345, ..
                        })) => Some(true),
                        Err(_) => None,
                    }
                } else {
                    let mut ro = before;
                    unsafe { call_game(&mut ro, copy.addr()) };
                    ro.preserved_except(&before, 0)
                        .then(|| WENT_FATAL.load(Ordering::SeqCst) == 1)
                };
                (
                    end,
                    std::mem::take(&mut *NOTED.lock().unwrap()),
                    capture(&[at(rva(0x4F49CC))]),
                )
            };
            let theirs = run(false);
            let mine = run(true);
            t.check(theirs.0.is_some() && theirs == mine, || {
                format!(
                    "mode {mode}, the loading fails {fails}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    restore(&inputs, &saved);
    t.finish()
}
