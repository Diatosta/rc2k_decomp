//! `audio::make_primary` (0x40A54B): the primary buffer made and set up.
//!
//! In:  nothing
//! Out: carry = no DirectSound, or the create failed; every register preserved
//!
//! See `audio/primary.rs`: nothing with A3D (carry clear); `g_bufferDescription` (flags
//! 0x81, + 0x10 in 3D); `CreateSoundBuffer` into `g_primaryBuffer`; `GetVolume` into
//! `g_primaryVolumeKept`; `audio::set_primary_format` (eax the mode); a master volume past
//! 0x40 becomes `audio::volume_index` of the kept volume; `g_masterVolumeShown`;
//! `audio::set_master_volume`; in 3D `audio::open_listener`, whose failure takes 3D out of
//! `g_soundMode`. Callers: `audio::set_mode`, `audio::close_primary_mixer`.
//! Shadow mode runs the original alone. Self-test: a redirected copy on the test
//! DirectSound whose three calls go to stand-ins (`volume_index` is the real one), 18 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::primary::*;
use crate::audio::sound_device::{LISTENER_FAILS, copy, game_part, handler, noted, test_part};
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use crate::service::harness::{capture, restore};
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::make_primary", rva(0x40A54B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let ok = unsafe { make_primary(&PrimaryVars::game(), &mut game_part) }?;
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
    if crate::audio::buffer_settings::a3d() {
        return t.skip("A3D is on");
    }
    // [0] DirectSound, [1] primary, [2] kept, [3] mode, [4] master, [5] shown.
    let mut vars = Box::new([0u32; 6]);
    let at = vars.as_mut_ptr() as u32;
    let v = PrimaryVars {
        direct_sound: at,
        primary: at + 4,
        kept: at + 8,
        mode: at + 12,
        master: at + 16,
        shown: at + 20,
    };
    let c = match copy(
        &HOOK,
        0x40A54B,
        0xD4,
        &[0x60, 0x83, 0x3D, 0x60, 0x6D, 0x50, 0x00, 0x01],
        &[0x28, 0x86, 0x9A, 0xA9, 0xBA],
        &[],
        &[
            (0x506A88, v.direct_sound),
            (0x506A8C, v.primary),
            (0x4F82DC, v.kept),
            (0x4F82BC, v.mode),
            (0x4F82D8, v.master),
            (0x755684, v.shown),
        ],
        13,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let description = [(
        game_addr(crate::audio::make_buffer::DESCRIPTION_RVA) as u32,
        0x14,
    )];
    let saved = capture(&description);
    let sound = TestSound::new();
    for ds in [true, false] {
        for create in [0, 0x8878_000Au32 as i32] {
            for mode in [0x11221u32, 0x111221] {
                for master in [0x20u32, 0x41] {
                    for fails in [false, true] {
                        if !ds && (create != 0 || master != 0x20 || fails) {
                            continue;
                        }
                        let run = |rust: bool| {
                            unsafe {
                                (at as *mut [u32; 6]).write_volatile([
                                    if ds { sound.sound() } else { 0 },
                                    0,
                                    1,
                                    mode,
                                    master,
                                    0x55,
                                ])
                            };
                            restore(&description, &saved);
                            LISTENER_FAILS.store(fails, Ordering::SeqCst);
                            sound.begin(Results {
                                create,
                                volume: 0xFFFF_FB50,
                                ..Results::default()
                            });
                            noted();
                            let before = Regs::SENTINEL;
                            let mut ro = before;
                            let carry = if rust {
                                unsafe { make_primary(&v, &mut test_part) }
                                    .ok()
                                    .map(|ok| !ok)
                            } else {
                                unsafe { call_game(&mut ro, c.addr()) };
                                ro.preserved_except(&before, 0).then(|| ro.carry())
                            };
                            (
                                carry,
                                sound.noted(),
                                noted(),
                                unsafe { (at as *const [u32; 6]).read_volatile() },
                                capture(&description),
                            )
                        };
                        let theirs = run(false);
                        let mine = run(true);
                        t.check(theirs.0.is_some() && theirs == mine, || {
                            format!("DirectSound {ds}, create {create:#x}, mode {mode:#x}, master {master:#x}, listener fails {fails}: original {theirs:x?}; rust {mine:x?}")
                        });
                    }
                }
            }
        }
    }
    LISTENER_FAILS.store(false, Ordering::SeqCst);
    restore(&description, &saved);
    drop(vars);
    t.finish()
}
