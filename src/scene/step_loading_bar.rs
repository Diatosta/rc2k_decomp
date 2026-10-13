//! `scene::step_loading_bar` (0x446786): moves the loading's progress on and shows it.
//!
//! In:  eax = how many of the system textures are done
//! Out: every register preserved
//!
//! Unless there are no system textures (`dword_5DC974`): the progress `dword_623700`
//! = the low end `dword_623634` plus eax times (the high end `dword_623638` less the
//! low) over the textures' number, plus 1. Then the function `dword_623704` names is
//! called (the front end's set-up puts `sub_4467C7` there, which draws).
//! One caller, `scene::reload_system_textures`.
//! Refuses no function to call.
//! Shadow mode runs the original alone and checks the progress it left.
//! Self-test: the original itself, the function it calls being a variable that the
//! test points at a stand-in of its own; 30 cases.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::step_loading_bar", rva(0x446786), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// `dword_623634` the low end, `dword_623638` the high end.
const ENDS_RVA: usize = rva(0x623634);
const PROGRESS_RVA: usize = rva(0x623700); // dword_623700
const SHOW_RVA: usize = rva(0x623704); // dword_623704
const SYSTEM_TEXTURES_RVA: usize = rva(0x5DC974); // dword_5DC974

// ---- Rust version ----

/// The progress, or None where it is left as it is.
unsafe fn progress(done: u32) -> Option<u32> {
    unsafe {
        let textures: u32 = game_read(SYSTEM_TEXTURES_RVA);
        if textures == 0 {
            return None;
        }
        let [low, high]: [u32; 2] = game_read(ENDS_RVA);
        // The product's low half, divided unsigned.
        let span = done.wrapping_mul(high.wrapping_sub(low));
        Some((span / textures).wrapping_add(low).wrapping_add(1))
    }
}

/// `show`: what stands for the function `dword_623704` names, entered with the
/// registers given.
pub unsafe fn step_loading_bar(r: &Regs, show: &mut dyn FnMut(&mut Regs)) -> Result<(), BadInput> {
    unsafe {
        if game_read::<u32>(SHOW_RVA) == 0 {
            return Err(BadInput("no function to show the loading with"));
        }
        let mut inner = *r;
        inner.ebp = {
            let [low, high]: [u32; 2] = game_read(ENDS_RVA);
            high.wrapping_sub(low)
        };
        if let Some(progress) = progress(r.eax) {
            game_write(PROGRESS_RVA, progress);
            inner.eax = progress.wrapping_sub(1);
            inner.edx = 0;
            inner.ebx = game_read(SYSTEM_TEXTURES_RVA);
        }
        inner.esi = game_read(SHOW_RVA);
        show(&mut inner);
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        let said = unsafe { progress(r.eax) };
        let done = r.eax;
        unsafe { call_original(r) };
        let left: u32 = unsafe { game_read(PROGRESS_RVA) };
        if said.is_some_and(|said| said != left) {
            HOOK.mismatch(|| format!("{done:#x} done: original {left:#x}, rust {said:x?}"));
        }
        return;
    }
    let done = unsafe {
        step_loading_bar(r, &mut |inner| {
            let show = inner.esi as usize;
            call_game(inner, show);
        })
    };
    if let Err(e) = done {
        let e: Refused = e.into();
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

/// Each showing: the progress by then.
static CALLED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn handler(_: u32, _: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push(unsafe { game_read(PROGRESS_RVA) });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(PROGRESS_RVA)];
    let inputs = [
        (crate::framework::game_addr(ENDS_RVA) as u32, 8usize),
        at(SHOW_RVA),
        at(SYSTEM_TEXTURES_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    for (low, high, textures) in [
        (0u32, 100u32, 315u32),
        (20, 64, 87),
        (50, 50, 1),
        (90, 10, 7),
        (0, 0xFFFF_FFFF, 3),
        (5, 100, 0),
    ] {
        for done in [0u32, 1, 86, 314, 0x4000_0000] {
            let run = |rust: bool| unsafe {
                game_write(PROGRESS_RVA, 0xAAAA_AAAAu32);
                game_write(ENDS_RVA, [low, high]);
                game_write(SYSTEM_TEXTURES_RVA, textures);
                game_write(SHOW_RVA, stand_in(0) as u32);
                CALLED.lock().unwrap().clear();
                set_handler(handler);
                let before = Regs {
                    eax: done,
                    ..Regs::SENTINEL
                };
                let kept = if rust {
                    step_loading_bar(&before, &mut |inner| {
                        let show = inner.esi as usize;
                        call_game(inner, show);
                    })
                    .is_ok()
                } else {
                    let mut r = before;
                    call_original(&mut r);
                    r.preserved_except(&before, 0)
                };
                (kept, capture(&regions), CALLED.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0 && theirs.2.len() == 1, || {
                format!(
                    "{done:#x} of {textures} done, {low:#x} .. {high:#x}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
