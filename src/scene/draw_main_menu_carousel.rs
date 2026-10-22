//! `scene::draw_main_menu_carousel` (0x49012C): the main menu's carousel, drawn the
//! plain way.
//!
//! In:  ecx, passed on
//! Out: as `scene::draw_carousel`
//!
//! `dword_6365C4` = 0 (the way `scene::draw_scene_object` draws by), then
//! `scene::draw_carousel`.
//! One caller, `scene::draw_main_menu_objects`.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose call notes the variable.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_carousel::draw_carousel_in_game;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::draw_main_menu_carousel",
    rva(0x49012C),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const MODE_RVA: usize = rva(0x6365C4); // dword_6365C4

// ---- Rust version ----

pub unsafe fn draw_main_menu_carousel(
    r: &mut Regs,
    draw: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe { game_write(MODE_RVA, 0u32) };
    draw(r)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { draw_main_menu_carousel(r, &mut |r| draw_carousel_in_game(r)) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    if let Err(e) = unsafe { draw_main_menu_carousel(r, &mut |r| draw_carousel_in_game(r)) } {
        *r = entry;
        return Err(e);
    }
    Ok(())
}

// ---- self-test ----

const LEN: usize = 0x10;
const ENTRY_BYTES: [u8; 10] = [0xC7, 0x05, 0xC4, 0x65, 0x63, 0x00, 0, 0, 0, 0];
const CALLS: [usize; 1] = [0x0A];

/// Each call: the variable by then, and ecx.
static CALLED: Mutex<Vec<[u32; 2]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED
        .lock()
        .unwrap()
        .push([unsafe { game_read(MODE_RVA) }, r.ecx]);
    r.ebx = 3;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x49012C)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x490649)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = [at(MODE_RVA)];
    let saved = capture(&regions);
    for mode in [0u32, 1, 2, 0xAAAA_AAAA] {
        let run = |rust: bool| unsafe {
            game_write(MODE_RVA, mode);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                let done = draw_main_menu_carousel(&mut r, &mut |r| {
                    act(0, r);
                    Ok(())
                });
                if done.is_err() {
                    r = Regs::default();
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                game_read::<u32>(MODE_RVA),
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.2.len() == 1, || {
            format!("mode {mode:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
