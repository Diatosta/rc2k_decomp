//! `audio::close_a3d2` (0x40CAD9): A3D's second interface closed.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `audio/a3d.rs`: `g_a3d2Interface` released and 0, `audio::release_a3d`, `g_useA3d` 0. Callers: `audio::start`, the A3D twin of `audio::close_sound`. Shadow mode runs the original alone. Self-test: a redirected copy whose `release_a3d` call goes to a stand-in, 2 cases.

use crate::audio::a3d::*;
use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::close_a3d2", rva(0x40CAD9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe {
        close_a3d2(
            game_addr(A3D2_INTERFACE_RVA) as u32,
            game_addr(crate::audio::voices::A3D_RVA) as u32,
            &mut crate::audio::sound_device::game_part,
        )
    }?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

fn self_test() -> (usize, usize) {
    use crate::audio::sound_device::{copy, handler, noted, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // [0] the interface, [1] use A3D.
    let mut vars = Box::new([0u32; 2]);
    let at = vars.as_mut_ptr() as u32;
    let c = match copy(
        &HOOK,
        0x40CAD9,
        0x30,
        &[0x60, 0x83, 0x3D, 0x58, 0x6D, 0x50, 0x00, 0x00],
        &[0x1F],
        &[],
        &[(0x506D58, at), (0x506D60, at + 4)],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    crate::install::stand_ins::set_handler(handler);
    let sound = TestSound::new();
    for interface in [0, sound.buffer(3)] {
        let run = |rust: bool| {
            unsafe { (at as *mut [u32; 2]).write_volatile([interface, 1]) };
            sound.begin(Results::default());
            noted();
            let before = Regs::SENTINEL;
            let mut ro = before;
            let done = if rust {
                unsafe { close_a3d2(at, at + 4, &mut test_part) }.is_ok()
            } else {
                unsafe { call_game(&mut ro, c.addr()) };
                ro.preserved_except(&before, 0)
            };
            (done, sound.noted(), noted(), unsafe {
                (at as *const [u32; 2]).read_volatile()
            })
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0 && theirs == mine, || {
            format!("interface {interface:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop(vars);
    t.finish()
}
