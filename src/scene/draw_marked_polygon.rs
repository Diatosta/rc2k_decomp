//! `scene::draw_marked_polygon` (0x44B9FC): `scene::draw_polygon` with a colour set
//! for the time of the call when the scene's flags say so.
//!
//! In:  nothing
//! Out: as `scene::draw_polygon` (the original leaves every register changed; the Rust
//!      version keeps them)
//!
//! `dword_623DB8` = 0, or 0xFFFF00FF with the top bit of `dword_5CCCD4`; the polygon;
//! `dword_623DB8` = 0. The variable is read only by the line drawing at 0x44EB04 and
//! 0x44EB1E (not read: an outline for a developer's view, by the colour).
//! Callers `sub_44CCEF` and `sub_44CD90`.
//! Shadow mode runs the original alone (its call of `draw_polygon` is compared there).
//! Self-test: a redirected copy whose call notes the variable.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::draw_polygon::{Game, draw_polygon};
use crate::scene::submit_polygon;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::draw_marked_polygon",
    rva(0x44B9FC),
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

const MARK_RVA: usize = rva(0x623DB8); // dword_623DB8
const FLAGS_RVA: usize = rva(0x5CCCD4); // dword_5CCCD4
const MARK: u32 = 0xFFFF_00FF;

// ---- Rust version ----

pub unsafe fn draw_marked_polygon(
    draw: &mut dyn FnMut() -> Result<(), BadInput>,
) -> Result<(), BadInput> {
    unsafe {
        let marked = game_read::<u32>(FLAGS_RVA) & 0x8000_0000 != 0;
        game_write(MARK_RVA, if marked { MARK } else { 0 });
        let done = draw();
        game_write(MARK_RVA, 0u32);
        done
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
        draw_marked_polygon(&mut || draw_polygon(&mut Game(submit_polygon::Game(&entry))))
    };
    if let Err(e) = done {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0x30;
const ENTRY_BYTES: [u8; 10] = [0xC7, 0x05, 0xB8, 0x3D, 0x62, 0x00, 0, 0, 0, 0];
const CALLS: [usize; 1] = [0x20];

/// The variable at each call of the polygon's routine.
static SEEN: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn handler(_: u32, _: &mut Regs) {
    SEEN.lock().unwrap().push(unsafe { game_read(MARK_RVA) });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x44B9FC)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x44BA2C)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = [at(MARK_RVA)];
    let inputs = [at(FLAGS_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    for flags in [0u32, 0x8000_0000, 0x7FFF_FFFF, 0xFFFF_FFFF, 1] {
        let run = |rust: bool| unsafe {
            game_write(MARK_RVA, 0xAAAA_AAAAu32);
            game_write(FLAGS_RVA, flags);
            SEEN.lock().unwrap().clear();
            set_handler(handler);
            let preserved = if rust {
                let mut r = Regs::SENTINEL;
                draw_marked_polygon(&mut || {
                    handler(0, &mut r);
                    Ok(())
                })
                .is_ok()
            } else {
                let mut ro = Regs::SENTINEL;
                call_game(&mut ro, copy.addr());
                ro.preserved_except(&Regs::SENTINEL, 0)
            };
            (preserved, capture(&regions), SEEN.lock().unwrap().clone())
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0 && theirs.2.len() == 1, || {
            format!("flags {flags:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
