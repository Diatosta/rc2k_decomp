//! `scene::end_scene` (0x44E952): the 3D part of a frame ends.
//!
//! In:  nothing
//! Out: as `d3d::end_scene` (0x41AB8A: Direct3D's EndScene, platform) leaves them
//!
//! A call of `d3d::end_scene` and nothing more. Nine callers.
//! Manual patch with a private copy: the function is that call.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose `d3d::end_scene` is the test's.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("scene::end_scene", rva(0x44E952), thunk, &ORIGINAL)
    .private_copy(6)
    .relocate_calls(&[0])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LAYER_RVA: usize = rva(0x41AB8A); // d3d::end_scene
const ENTRY_BYTES: [u8; 5] = [0xE8, 0x33, 0xC2, 0xFC, 0xFF];

// ---- Rust version ----

/// `end` stands for `d3d::end_scene`.
pub fn end_scene(r: &mut Regs, end: &mut dyn FnMut(&mut Regs)) {
    end(r);
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    end_scene(r, &mut |r| unsafe { call_game(r, game_addr(LAYER_RVA)) });
}

// ---- self-test ----

fn handler(_place: u32, r: &mut Regs) {
    r.eax = r.edx.wrapping_mul(3);
    let fail = r.edx == 1;
    r.set_carry(fail);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x44E952)), 6, &ENTRY_BYTES, &[0], &|callee| {
            (callee == LAYER_RVA).then(|| stand_in(0))
        })
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
        let mut theirs = before;
        unsafe { call_game(&mut theirs, copy.addr()) };
        let mut mine = before;
        end_scene(&mut mine, &mut |r| act(0, r));
        let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
        t.check(
            regs(&theirs) == regs(&mine) && theirs.carry() == mine.carry(),
            || {
                format!(
                    "failing {fails}: {:x?} carry {} / {:x?} carry {}",
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
