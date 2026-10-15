//! `audio::set_up_primary_mixer` (0x40BAD2): the game's own mixer set up on the primary
//! buffer (the Dolby mode).
//!
//! In:  nothing
//! Out: carry = failed (the mixer closed again); every register preserved
//!
//! See `audio/mixer_set_up.rs`. Entered only by `audio::open_way`'s `jnz` (IDA kept it as a
//! chunk of that function until 2026-10-15), so it returns to that one's caller. Shadow
//! mode runs the original alone. Self-test: a redirected copy on the test DirectSound with
//! the calls at stand-ins, 34 cases.

use crate::audio::buffer_play::{Leaves, dispatch_leaf};
use crate::audio::mixer_set_up::*;
use crate::audio::restore::{CARRIES, FORMAT_FAILS, copy, game_part, handler, noted, test_part};
use crate::audio::test_sound::{Results, TestSound};
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::install::stand_ins::set_handler;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "audio::set_up_primary_mixer",
    rva(0x40BAD2),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn rust(r: &Regs) -> Result<Leaves, Refused> {
    let ok = unsafe { set_up_primary_mixer(&MixerVars::game(), &mut game_part) }?;
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
    // 0 mode, 1 DirectSound, 2 window, 3 primary, 4 .. 8 the wave format (0x12 bytes),
    // 9 rate, 10 most, 11 shift, 12 play, 13 write, 14 .. 18 the caps, 19 size.
    let mut m = vec![0u32; 20];
    let at = m.as_mut_ptr() as u32;
    let a = |n: u32| at + n * 4;
    let v = MixerVars {
        mode: a(0),
        direct_sound: a(1),
        window: a(2),
        primary: a(3),
        wave: a(4),
        rate: a(9),
        most: a(10),
        shift: a(11),
        play: a(12),
        write: a(13),
        caps: a(14),
        size: a(19),
    };
    let c = match copy(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x40BAD2,
            len: 0xF9,
            entry: &[
                0x60, 0xF7, 0x05, 0xBC, 0x82, 0x4F, 0x00, 0x00, 0x00, 0x04, 0x00,
            ],
            calls: &[0x11, 0x31, 0x41, 0x50, 0x61, 0xB9, 0xE8, 0xF1],
            jumps_out: &[],
        },
        &[
            (0x4F82BC, v.mode),
            (0x506A88, v.direct_sound),
            (0x5184A4, v.window),
            (0x506A8C, v.primary),
            (0x506B34, v.wave + 4),
            (0x506B3C, v.wave + 0xC),
            (0x5068D4, v.rate),
            (0x5068DC, v.most),
            (0x5068E0, v.shift),
            (0x5068E4, v.play),
            (0x5068E8, v.write),
            (0x506B08, v.caps),
            (0x506B10, v.caps + 8),
            (0x5068D8, v.size),
        ],
        18,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    set_handler(handler);
    let sound = TestSound::new();
    let mut cases = Vec::new();
    for cooperative in [0, 0x8878_000Au32 as i32] {
        for make_fails in [false, true] {
            for format_fails in 0..4u32 {
                for caps in [0, 0x8878_0032u32 as i32] {
                    cases.push((0x2441220u32, cooperative, make_fails, format_fails, caps));
                }
            }
        }
    }
    cases.push((0x11221, 0, false, 0, 0));
    cases.push((0x11221, 1, true, 3, 1));
    for (mode, cooperative, make_fails, format_fails, caps) in cases {
        let run = |rust: bool| {
            unsafe {
                let p = at as *mut u32;
                for n in 0..20 {
                    p.add(n).write_volatile(0x5555_5555);
                }
                p.write_volatile(mode);
                p.add(1).write_volatile(sound.sound());
                p.add(2).write_volatile(0x1234);
                p.add(3).write_volatile(sound.buffer(2));
                p.add(5).write_volatile(22050);
                p.add(7).write_volatile(0x0010_0004);
            }
            {
                let mut c = CARRIES.lock().unwrap();
                *c = [false; 12];
                c[8] = make_fails;
            }
            FORMAT_FAILS.store(format_fails, Ordering::SeqCst);
            sound.begin(Results {
                cooperative,
                caps,
                bytes: 0x8000,
                ..Results::default()
            });
            noted();
            let before = Regs::SENTINEL;
            let mut ro = before;
            let carry = if rust {
                unsafe { set_up_primary_mixer(&v, &mut test_part) }
                    .ok()
                    .map(|ok| !ok)
            } else {
                unsafe { call_game(&mut ro, c.addr()) };
                ro.preserved_except(&before, 0).then(|| ro.carry())
            };
            (carry, noted(), sound.noted(), unsafe {
                (at as *const [u32; 20]).read_volatile()
            })
        };
        let theirs = run(false);
        let mine = run(true);
        t.check(theirs.0.is_some() && theirs == mine, || {
            format!("mode {mode:#x}, level {cooperative:#x}, make fails {make_fails}, formats failing {format_fails}, caps {caps:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    *CARRIES.lock().unwrap() = [false; 12];
    FORMAT_FAILS.store(0, Ordering::SeqCst);
    drop(m);
    t.finish()
}
