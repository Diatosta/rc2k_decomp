//! `audio::open_listener` (0x40C5F9): the 3D listener taken from the primary buffer.
//!
//! In:  nothing
//! Out: carry = none (logged, and `audio::release_sound_listener`); every register
//!      preserved
//!
//! "AUD: open ds3d"; only with `g_ds3dSetting` 1 and the dword at 0x506B42 not 1:
//! `QueryInterface` of `g_primaryBuffer` with `g_iidListener3d` into `g_soundListener`.
//! Otherwise "AUD: open ds3d failed rc=" and the code (0xFFFFFF91 for the setting,
//! 0xFFFFFF90 for 0x506B42, else the result). One caller, `audio::make_primary`. See
//! `audio/primary.rs`. Shadow mode runs the original alone. Self-test: a redirected copy on
//! the test DirectSound's buffer whose log and release calls go to stand-ins, 8 cases.

use crate::audio::buffer_play::{LISTENER_3D_RVA, Leaves, dispatch_leaf};
use crate::audio::primary::*;
use crate::audio::sound_device::{copy, game_part, handler, noted, test_part};
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::open_listener", rva(0x40C5F9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let ok = unsafe {
        open_listener(
            game_addr(crate::audio::voices::PRIMARY_RVA) as u32,
            game_addr(LISTENER_3D_RVA) as u32,
            game_addr(DS3D_SETTING_RVA) as u32,
            game_addr(NO_LISTENER_RVA) as u32,
            &mut game_part,
        )
    }?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!ok),
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
    // [0] primary, [1] listener, [2] the setting, [3] the 0x506B42 dword.
    let mut vars = Box::new([0u32; 4]);
    let at = vars.as_mut_ptr() as u32;
    let c = match copy(
        &HOOK,
        0x40C5F9,
        0x5C,
        &[0x60, 0x60, 0x8D, 0x05, 0x15, 0xA7, 0x4D, 0x00],
        &[0x08, 0x47, 0x4E, 0x54],
        &[],
        &[
            (0x506A8C, at),
            (0x506A90, at + 4),
            (0x5183EC, at + 8),
            (0x506B42, at + 12),
        ],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let sound = TestSound::new();
    for setting in [1u32, 0] {
        for no_listener in [0u32, 1] {
            for query in [0, 0x8000_4002u32 as i32] {
                let run = |rust: bool| {
                    unsafe {
                        (at as *mut [u32; 4]).write_volatile([
                            sound.buffer(1),
                            0,
                            setting,
                            no_listener,
                        ])
                    };
                    sound.begin(Results {
                        query,
                        ..Results::default()
                    });
                    noted();
                    let before = Regs::SENTINEL;
                    let mut ro = before;
                    let carry = if rust {
                        unsafe { open_listener(at, at + 4, at + 8, at + 12, &mut test_part) }
                            .ok()
                            .map(|ok| !ok)
                    } else {
                        unsafe { call_game(&mut ro, c.addr()) };
                        ro.preserved_except(&before, 0).then(|| ro.carry())
                    };
                    (carry, sound.noted(), noted(), unsafe {
                        (at as *const [u32; 4]).read_volatile()
                    })
                };
                let theirs = run(false);
                let mine = run(true);
                t.check(theirs.0.is_some() && theirs == mine, || {
                    format!("setting {setting}, 0x506B42 {no_listener}, query {query:#x}: original {theirs:x?}; rust {mine:x?}")
                });
            }
        }
    }
    drop(vars);
    t.finish()
}
