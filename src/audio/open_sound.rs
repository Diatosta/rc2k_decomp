//! `audio::open_sound` (0x409E1D): DirectSound opened.
//!
//! In:  eax = the device's GUID (0: the default one)
//! Out: carry = failed; every register preserved
//!
//! See `audio/sound_device.rs`: logged; mode bit 0x2000 `audio::open_a3d2` (done if it
//! works; else the mode's 0x3000 bits flipped); only with bit 0x1000: `audio::open_a3d`,
//! failing which `DirectSoundCreate` (the game's import, the ASI loader's dsound.dll);
//! cooperative level 3 with `hWnd`. Callers: `audio::start`, `audio::reopen_sound`. Shadow
//! mode runs the original alone. Self-test: a redirected copy on the test DirectSound whose
//! calls go to stand-ins (`DirectSoundCreate` too), 128 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::sound_device::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::open_sound", rva(0x409E1D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let ok = unsafe {
        open_sound(
            r.eax,
            &OpenVars::game(),
            &mut game_part,
            &mut game_direct_sound_create,
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
    // [0] mode, [1] DirectSound, [2] window.
    let mut vars = Box::new([0u32; 3]);
    let at = vars.as_mut_ptr() as u32;
    let v = OpenVars {
        mode: at,
        direct_sound: at + 4,
        window: at + 8,
    };
    let c = match copy(
        &HOOK,
        0x409E1D,
        0x88,
        &[0x60, 0x60, 0x8D, 0x05, 0x0F, 0xA6, 0x4D, 0x00],
        &[0x08, 0x0F, 0x21, 0x3E, 0x4D, 0x78, 0x7F],
        &[],
        &[
            (0x4F82BC, v.mode),
            (0x506A88, v.direct_sound),
            (0x5184A4, v.window),
        ],
        6,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let sound = TestSound::new();
    TEST_DIRECT_SOUND.store(sound.sound(), Ordering::SeqCst);
    for mode in [0x11221u32, 0x13221, 0x12221, 0x10221] {
        for guid in [0u32, 0x1234] {
            for a3d2_fails in [true, false] {
                for a3d_fails in [true, false] {
                    for create in [0, 0x8878_0078u32 as i32] {
                        for cooperative in [0, 0x8878_000Au32 as i32] {
                            let run = |rust: bool| {
                                // The DirectSound there already: what a working `open_a3d` (a
                                // stand-in here) would have left.
                                unsafe {
                                    (at as *mut [u32; 3]).write_volatile([
                                        mode,
                                        sound.sound(),
                                        0x1234,
                                    ])
                                };
                                *PART_CARRIES.lock().unwrap() =
                                    [a3d2_fails, a3d_fails, false, false];
                                CREATE_RESULT.store(create, Ordering::SeqCst);
                                sound.begin(Results {
                                    cooperative,
                                    ..Results::default()
                                });
                                noted();
                                let before = Regs {
                                    eax: guid,
                                    ..Regs::SENTINEL
                                };
                                let mut ro = before;
                                let carry = if rust {
                                    unsafe {
                                        open_sound(
                                            guid,
                                            &v,
                                            &mut test_part,
                                            &mut test_direct_sound_create,
                                        )
                                    }
                                    .ok()
                                    .map(|ok| !ok)
                                } else {
                                    unsafe { call_game(&mut ro, c.addr()) };
                                    ro.preserved_except(&before, 0).then(|| ro.carry())
                                };
                                (carry, noted(), sound.noted(), unsafe {
                                    (at as *const [u32; 3]).read_volatile()
                                })
                            };
                            let theirs = run(false);
                            let mine = run(true);
                            t.check(theirs.0.is_some() && theirs == mine, || {
                                format!("mode {mode:#x}, GUID {guid:#x}, A3D2 fails {a3d2_fails}, A3D fails {a3d_fails}, create {create:#x}, level {cooperative:#x}: original {theirs:x?}; rust {mine:x?}")
                            });
                        }
                    }
                }
            }
        }
    }
    *PART_CARRIES.lock().unwrap() = [false; 4];
    CREATE_RESULT.store(0, Ordering::SeqCst);
    drop(vars);
    t.finish()
}
