//! `audio::read_caps` (0x409EF8): DirectSound's capabilities read and logged.
//!
//! In:  nothing
//! Out: every register preserved; carry = `GetCaps` failed
//!
//! See `audio/sound_device.rs`. Shadow mode runs the original alone. Self-test: a redirected copy on the test DirectSound whose log calls go to stand-ins, 2 cases. Refuses no `g_directSound` (the original calls through 0).

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::sound_device::*;
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("audio::read_caps", rva(0x409EF8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let ok = unsafe {
        read_caps(
            game_addr(crate::audio::make_buffer::DIRECT_SOUND_RVA) as u32,
            game_addr(CAPS_RVA) as u32,
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
    let mut vars = Box::new([0u32; 1]);
    let mut caps = Box::new([0u32; 0x18]);
    let at = vars.as_mut_ptr() as u32;
    let cp = caps.as_mut_ptr() as u32;
    let mut variables = vec![(0x506A88usize, at), (0x506AA8, cp)];
    for n in (4..=0x40).step_by(4) {
        variables.push((0x506AA8 + n, cp + n as u32));
    }
    let calls = [
        0x0E, 0x3E, 0x4A, 0x56, 0x62, 0x6E, 0x7B, 0x87, 0x93, 0x9F, 0xAB, 0xB7, 0xC3, 0xD0, 0xDC,
        0xE8, 0xF4, 0x100, 0x10C, 0x118,
    ];
    let c = match copy(
        &HOOK,
        0x409EF8,
        0x125,
        &[0x60, 0x53, 0x51, 0x8D, 0x1D, 0xA8, 0x6A, 0x50],
        &calls,
        &[],
        &variables,
        20,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let sound = TestSound::new();
    for result in [0, 0x8878_000Au32 as i32] {
        let run = |rust: bool| {
            unsafe {
                (at as *mut u32).write_volatile(sound.sound());
                (cp as *mut [u32; 0x18]).write_volatile([0xAAAA_AAAA; 0x18]);
            }
            sound.begin(Results {
                caps: result,
                ..Results::default()
            });
            noted();
            let before = Regs::SENTINEL;
            let mut ro = before;
            let carry = if rust {
                unsafe { read_caps(at, cp, &mut test_part) }
                    .map(|ok| !ok)
                    .ok()
            } else {
                unsafe { call_game(&mut ro, c.addr()) };
                ro.preserved_except(&before, 0).then(|| ro.carry())
            };
            (carry, sound.noted(), noted(), unsafe {
                (cp as *const [u32; 0x18]).read_volatile()
            })
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0.is_some() && theirs == mine, || {
            format!("GetCaps {result:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    drop((vars, caps));
    t.finish()
}
