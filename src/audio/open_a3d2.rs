//! `audio::open_a3d2` (0x40CA17): A3D's second interface tried.
//!
//! In:  eax = the device's GUID (read only when A3D is there)
//! Out: carry = A3D not used; every register preserved
//!
//! See `audio/a3d.rs`: with `g_noA3dSetting` bit 2: the A3D object released and 0, carry. Else `g_useA3d` 0 and `CoCreateInstance` of A3D2's class into `g_a3dObject`; failing: 0, carry. When it works the Rust version releases it and refuses. Callers: `audio::open_sound` (mode bit 0x2000), `audio::start`. Shadow mode runs the original alone. Self-test: a redirected copy whose `CoCreateInstance` is a stand-in that finds no class, 8 cases.

use crate::audio::a3d::*;
use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::open_a3d2", rva(0x40CA17), thunk, &ORIGINAL).self_test(self_test);

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
    let ok = unsafe { open_a3d2(&A3dVars::game(), &mut game_create) }?;
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
            address: 0x40CA17,
            len: 0xC2,
            entry: &[0x60, 0x8B, 0xD8, 0xF7, 0x05, 0x5C, 0x6C, 0x50, 0x00, 0x02],
            // CoCreateInstance; the log (a path the tests do not take).
            calls: &[0x30, 0x8D],
            jumps_out: &[],
        },
        &|callee| (callee == rva(0x4D7CCE)).then_some(no_class as *const () as usize),
        &[
            (0x506C5C, v.setting),
            (0x506C68, v.object),
            (0x506D60, v.use_a3d),
            (0x506D58, at + 20),
        ],
        12,
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
                    unsafe { open_a3d2(&v, &mut test_create) }
                        .ok()
                        .map(|ok| !ok)
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
