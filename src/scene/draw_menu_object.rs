//! `scene::draw_menu_object` (0x4907E7): draws the object `g_objectNumber` the way the
//! menus show theirs.
//!
//! In:  nothing
//! Out: eax = 0x100; the rest as `scene::draw_scene_object` leaves them
//!
//! `g_shine` = 0x200 (`scene::set_shine` 0x422292, inlined); `scene::make_object_matrix`
//! from the angles, place and size the caller set; `dword_6365C8` = 5; `g_lightKind`
//! = 4; `scene::draw_scene_object`; `g_shine` = 0x100.
//! 14 callers: the carousel's entry and thirteen menus' objects.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose two calls note what was set by then.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::matrix::make_object_matrix;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_menu_object", rva(0x4907E7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SHINE_RVA: usize = rva(0x5CCE74); // g_shine
const VIEW_KIND_RVA: usize = rva(0x6365C8); // dword_6365C8
const LIGHT_KIND_RVA: usize = rva(0x62DAF4); // g_lightKind

/// The two calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Matrix,
    Draw,
}

// ---- Rust version ----

pub unsafe fn draw_menu_object(
    r: &mut Regs,
    step: &mut dyn FnMut(Step, &mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        r.eax = 0x200;
        game_write(SHINE_RVA, 0x200u32);
        step(Step::Matrix, r)?;
        game_write(VIEW_KIND_RVA, 5u32);
        game_write(LIGHT_KIND_RVA, 4u32);
        step(Step::Draw, r)?;
        r.eax = 0x100;
        game_write(SHINE_RVA, 0x100u32);
        Ok(())
    }
}

/// With the Rust versions of what it calls.
pub unsafe fn draw_menu_object_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        draw_menu_object(r, &mut |step, r| match step {
            Step::Matrix => {
                make_object_matrix().apply(r);
                Ok(())
            }
            Step::Draw => crate::scene::draw_scene_object::draw_scene_object_in_game(r),
        })
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
    if let Err(e) = unsafe { draw_menu_object_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const LEN: usize = 0x33;
const ENTRY_BYTES: [u8; 5] = [0xB8, 0x00, 0x02, 0x00, 0x00];
const CALLS: [usize; 4] = [0x05, 0x0A, 0x23, 0x2D];

/// Each call: its place, the three variables by then, and eax as it was entered with.
static CALLED: Mutex<Vec<[u32; 5]>> = Mutex::new(Vec::new());

fn handler(place: u32, r: &mut Regs) {
    unsafe {
        CALLED.lock().unwrap().push([
            place,
            game_read(SHINE_RVA),
            game_read(VIEW_KIND_RVA),
            game_read(LIGHT_KIND_RVA),
            r.eax,
        ]);
    }
    r.ebx = r.ebx.wrapping_add(place + 3);
    r.esi ^= 0x00FF_0000 << place;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x4907E7)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                if callee == rva(0x427812) {
                    Some(stand_in(0))
                } else if callee == rva(0x451E3E) {
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
    let regions = [at(SHINE_RVA), at(VIEW_KIND_RVA), at(LIGHT_KIND_RVA)];
    let saved = capture(&regions);
    for fill in [0xAAu8, 0x00, 0xFF] {
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![fill; saved.len()]);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                let done = draw_menu_object(&mut r, &mut |step, r| {
                    act(step as u32, r);
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
                capture(&regions),
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.2.len() == 2, || {
            format!("filled with {fill:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
