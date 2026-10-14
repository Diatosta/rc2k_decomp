//! `scene::begin_scene` (0x44E8A1): the 3D part of a frame begins.
//!
//! In:  nothing
//! Out: the registers and flags `d3d::begin_scene` leaves (carry = it failed)
//!
//! `d3d::begin_scene` (0x41ABC7: Direct3D's BeginScene and the layer's own states,
//! platform), then, whatever it says, `scene::set_filters` and render states 0x15 = 4
//! (the texture's blending), 0x1C = 1 (fog on), 0x13 = 5 and 0x14 = 6 (the source and
//! destination blend), 0x1B = 1 (blending on), all between `pusha` / `pushf` and
//! `popf` / `popa`. Nine callers, `menu::frame_callback` among them.
//! Manual patch with a private copy: the function begins with a call.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose `d3d::begin_scene` is the test's, the states
//! recorded on the layer, nothing drawn.

use crate::d3d::recorder::record;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::set_filters::set_filters;
use crate::scene::set_render_state::set_render_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::begin_scene", rva(0x44E8A1), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&CALLS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LAYER_RVA: usize = rva(0x41ABC7); // d3d::begin_scene
const LEN: usize = 0x5A;
const ENTRY_BYTES: [u8; 5] = [0xE8, 0x21, 0xC3, 0xFC, 0xFF];
const CALLS: [usize; 7] = [0x00, 0x07, 0x16, 0x25, 0x34, 0x43, 0x52];
/// The states after the filters: (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (1, 0x1C), (5, 0x13), (6, 0x14), (1, 0x1B)];

// ---- Rust version ----

/// `begin` stands for `d3d::begin_scene`.
pub unsafe fn begin_scene(r: &mut Regs, begin: &mut dyn FnMut(&mut Regs)) {
    begin(r);
    let mut work = *r;
    unsafe {
        set_filters(&mut work);
        for (value, state) in STATES {
            set_render_state(&mut work, value, state);
        }
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    unsafe { begin_scene(r, &mut |r| call_game(r, game_addr(LAYER_RVA))) };
}

// ---- self-test ----

fn handler(_place: u32, r: &mut Regs) {
    r.eax = 0x1234_5678;
    r.ecx = r.ecx.wrapping_add(1);
    let fail = r.edx == 1;
    r.set_carry(fail);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x44E8A1)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == LAYER_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    set_handler(handler);
    for fails in [0u32, 1] {
        let before = Regs {
            edx: fails,
            ..Regs::SENTINEL
        };
        let (theirs, their_calls) = record(true, || {
            let mut r = before;
            unsafe { call_game(&mut r, copy.addr()) };
            r
        });
        let (mine, my_calls) = record(true, || {
            let mut r = before;
            unsafe { begin_scene(&mut r, &mut |r| act(0, r)) };
            r
        });
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        t.check(
            regs(&theirs) == regs(&mine)
                && theirs.carry() == mine.carry()
                && their_calls == my_calls,
            || {
                format!(
                    "failing {fails}: regs {:x?} carry {} / {:x?} carry {}; calls {their_calls:x?} / {my_calls:x?}",
                    regs(&theirs),
                    theirs.carry(),
                    regs(&mine),
                    mine.carry()
                )
            },
        );
    }
    t.finish()
}
