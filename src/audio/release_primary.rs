//! `audio::release_primary` (0x40A61F): the primary buffer given back.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `audio/sound_device.rs`. Shadow mode runs the original alone. Self-test: a redirected copy on the test DirectSound's buffer whose two calls go to stand-ins, 4 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::sound_device::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::release_primary", rva(0x40A61F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    unsafe {
        release_primary(
            game_addr(crate::audio::voices::PRIMARY_RVA) as u32,
            game_addr(VOLUME_KEPT_RVA) as u32,
            &mut game_part,
        )
    }?;
    Ok(Leaves {
        eax: r.eax,
        carry: None,
        zero: None,
    })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_leaf(&HOOK, r, call_original, rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut vars = Box::new([0u32; 2]);
    let at = vars.as_mut_ptr() as u32;
    let c = match copy(
        &HOOK,
        0x40A61F,
        0x58,
        &[0x60, 0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
        &[0x13, 0x3C],
        &[],
        &[(0x506A8C, at), (0x4F82DC, at + 4)],
        7,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let sound = TestSound::new();
    for primary in [0, sound.buffer(2)] {
        for kept in [1u32, 0xFFFF_F000] {
            let run = |rust: bool| {
                unsafe { (at as *mut [u32; 2]).write_volatile([primary, kept]) };
                sound.begin(Results::default());
                noted();
                let before = Regs::SENTINEL;
                let mut ro = before;
                let done = if rust {
                    unsafe { release_primary(at, at + 4, &mut test_part) }.is_ok()
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
                format!(
                    "primary {primary:#x}, kept {kept:#x}: original {theirs:x?}; rust {mine:x?}"
                )
            });
        }
    }
    drop(vars);
    t.finish()
}
