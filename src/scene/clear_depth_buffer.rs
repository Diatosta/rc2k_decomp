//! `scene::clear_depth_buffer` (0x44E967): clears the depth buffer over the whole
//! screen.
//!
//! In:  nothing
//! Out: nothing changed without 3D; else eax, ecx, edx as Direct3D leaves them and
//!      ebx = the screen's height
//!
//! Only with `dword_5CDA8C` not 0: the rectangle `dword_624138` .. = 0, 0, the
//! screen's width and height, and `d3d::clear_viewport` (0x44E9D9: the viewport's
//! Clear with that one rectangle and flag 2, the depth buffer; Direct3D, not ported).
//! Three callers.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose call notes the rectangle.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::clear_depth_buffer", rva(0x44E967), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const HAS_3D_RVA: usize = rva(0x5CDA8C); // dword_5CDA8C
const RECTANGLE_RVA: usize = rva(0x624138); // dword_624138 .. dword_624144
const WIDTH_RVA: usize = rva(0x4F494C); // g_screenWidth
const HEIGHT_RVA: usize = rva(0x4F4950); // g_screenHeight
const CLEAR_RVA: usize = rva(0x44E9D9); // d3d::clear_viewport

// ---- Rust version ----

/// `clear`: what stands for `d3d::clear_viewport`.
pub unsafe fn clear_depth_buffer(r: &mut Regs, clear: &mut dyn FnMut(&mut Regs)) {
    unsafe {
        if game_read::<u32>(HAS_3D_RVA) == 0 {
            return;
        }
        r.eax = game_read(WIDTH_RVA);
        r.ebx = game_read(HEIGHT_RVA);
        game_write(RECTANGLE_RVA, [0, 0, r.eax, r.ebx]);
        clear(r);
    }
}

pub unsafe fn clear_depth_buffer_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe { clear_depth_buffer(r, &mut |r| call_game(r, game_addr(CLEAR_RVA))) };
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { clear_depth_buffer(r, &mut |r| call_game(r, game_addr(CLEAR_RVA))) };
}

// ---- self-test ----

const LEN: usize = 0x39;
const ENTRY_BYTES: [u8; 7] = [0x83, 0x3D, 0x8C, 0xDA, 0x5C, 0x00, 0x00];
const CALLS: [usize; 1] = [0x33];

/// Each call: the rectangle by then, eax and ebx.
static CALLED: Mutex<Vec<[u32; 6]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    let rectangle: [u32; 4] = unsafe { game_read(RECTANGLE_RVA) };
    CALLED.lock().unwrap().push([
        rectangle[0],
        rectangle[1],
        rectangle[2],
        rectangle[3],
        r.eax,
        r.ebx,
    ]);
    r.eax = 0;
    r.ecx = 0x77;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x44E967)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == CLEAR_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 1] = [(game_addr(RECTANGLE_RVA) as u32, 16)];
    let inputs = [at(HAS_3D_RVA), at(WIDTH_RVA), at(HEIGHT_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    for (has, width, height) in [
        (0u32, 800u32, 600u32),
        (1, 800, 600),
        (0x100, 640, 480),
        (1, 0, 0xFFFF_FFFF),
    ] {
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(HAS_3D_RVA, has);
            game_write(WIDTH_RVA, width);
            game_write(HEIGHT_RVA, height);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                clear_depth_buffer(&mut r, &mut |r| act(0, r));
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
        t.check(theirs == mine, || {
            format!("3D {has:#x}, {width} x {height}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
