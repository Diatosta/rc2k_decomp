//! `audio::open_a3d` (0x40C76D): A3D tried as the DirectSound.
//!
//! In:  eax = the device's GUID (read only when A3D is there)
//! Out: carry = A3D not used; every register preserved
//!
//! See `audio/a3d.rs`: with `g_noA3dSetting` bit 1: the A3D object and DirectSound released, carry. Else `g_a3dReady` 0 and `CoCreateInstance` of A3D's class into `g_directSound`; failing (as on any machine today): `g_directSound` 0, carry. When it works the Rust version releases what it made and refuses (the original's A3D set-up is not ported). One caller, `audio::open_sound`. Shadow mode runs the original alone. Self-test: a redirected copy whose `CoCreateInstance` is a stand-in that finds no class, 8 cases.

use crate::audio::a3d::*;
use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::open_a3d", rva(0x40C76D), thunk, &ORIGINAL).self_test(self_test);

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
    let ok = unsafe { open_a3d(&A3dVars::game(), &mut game_create) }?;
    Ok(Leaves {
        eax: r.eax,
        carry: Some(!ok),
        zero: None,
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // [0] the setting, [1] ready, [2] DirectSound, [3] the A3D object, [4] use A3D,
    // [5] the A3D2 interface.
    let mut vars = Box::new([0u32; 6]);
    let at = vars.as_mut_ptr() as u32;
    let v = A3dVars {
        setting: at,
        ready: at + 4,
        direct_sound: at + 8,
        object: at + 12,
        use_a3d: at + 16,
    };
    let c = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40C76D,
            len: 0x135,
            entry: &[0x60, 0x8B, 0xD8, 0xF7, 0x05, 0x5C, 0x6C, 0x50, 0x00, 0x01],
            // CoCreateInstance; the two logs (paths the tests do not take).
            calls: &[0x30, 0x9D, 0xE2],
            jumps_out: &[],
        },
        &|callee| (callee == rva(0x4D7CCE)).then_some(no_class as *const () as usize),
        &[
            (0x506C5C, v.setting),
            (0x506C64, v.ready),
            (0x506A88, v.direct_sound),
            (0x506C68, v.object),
        ],
        19,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let sound = TestSound::new();
    for setting in [0u32, 1, 2, 3] {
        for preset in [false, true] {
            let run = |rust: bool| {
                let (ds, object) = if preset {
                    (sound.buffer(1), sound.buffer(2))
                } else {
                    (0, 0)
                };
                unsafe {
                    (at as *mut [u32; 6]).write_volatile([setting, 0x55, ds, object, 0x55, 0x66])
                };
                sound.begin(Results::default());
                let before = Regs::SENTINEL;
                let mut ro = before;
                let carry = if rust {
                    unsafe { open_a3d(&v, &mut test_create) }.ok().map(|ok| !ok)
                } else {
                    unsafe { call_game(&mut ro, c.addr()) };
                    ro.preserved_except(&before, 0).then(|| ro.carry())
                };
                (carry, sound.noted(), unsafe {
                    (at as *const [u32; 6]).read_volatile()
                })
            };
            let theirs = run(false);
            let mine = run(true);
            t.check(theirs.0.is_some() && theirs == mine, || {
                format!("setting {setting}, objects there {preset}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    drop(vars);
    t.finish()
}
