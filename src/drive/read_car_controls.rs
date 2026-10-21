//! `drive::read_car_controls` (0x43A35A): the work car's controls for this step:
//! - the keys (`dword_6138C4` .. `D0`) into the car's (`dword_71A43C` .. `48`), the
//!   held-before into `dword_5C694C` and the newly pressed (`dword_71A44C`, bits not held
//!   before); steering, throttle and brake into 0x613890 .. 0x613898, and three bits as
//!   0x61389C (bit 2 times 0x60), 0x6138A0, 0x6138A4;
//! - for a player's car (or in a replay, `dword_52A910`): bit 3 newly pressed flips
//!   `dword_71A05C`, bit 5 changes the view (`drive::next_camera_view` on `g_car`);
//! - for a computer's car (`dword_71C3E0` set, or `dword_62389C` and `dword_71A234` 2):
//!   `drive::start_blend`, `flt_71BFA4` = `flt_71A658`, `drive::drive_computer_car`, and
//!   its six controls (0x71BF88 ..) into 0x613890 .. 0x6138A4.
//!
//! The bytes after its `retn` (0x43A4E5 ..) are another entry that nothing calls.
//!
//! In:  nothing
//! Out: eax as the path leaves it (the last bit's, `next_camera_view`'s, or
//!      `start_blend`'s count); esi `g_car` when the view changed; the rest kept; no
//!      flags read (callers go on with calls)
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`, `sub_43A1CA`.
//! Refuses what `drive_computer_car` refuses (before anything).
//! Self-test: in a race only: a copy of the original (its calls the game's) against the
//! Rust version on the game's state, keys and switches by lot; everything written saved
//! and put back.

use crate::drive::drive_computer_car::drive_computer_car;
use crate::drive::next_camera_view::next_camera_view;
use crate::drive::start_blend::start_blend;
use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use rc2k_core::drive::controls::{FLIP_BIT, VIEW_BIT, computer_drives, flipped, read_keys};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::read_car_controls", rva(0x43A35A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

fn a(x: u32) -> u32 {
    game_addr(rva(x as usize)) as u32
}

const CURRENT: u32 = 0x71B3F8; // g_currentCar

/// Does a computer drive this step?
fn computer() -> bool {
    computer_drives(g(0x71C3E0), || g(0x62389C), || g(0x71A234))
}

/// The Rust version: `r` as the original leaves it.
// Converted: `rc2k_core::drive::controls::{read_keys, computer_drives, flipped}`.
pub unsafe fn read_car_controls(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if computer() && g(0x623640) == 0 && g(0x719FEC) >= 0x320 {
            // `drive_computer_car`'s refusal, before anything here is written.
            let reaches = crate::math::x87_run::compare_full(g(0x71A0A0), g(0x71BF70)) & 0x100 == 0;
            if reaches {
                return Err(BadInput("no checkpoints for the computer's driving").into());
            }
        }
        p(0x5C694C, g(0x71A440));
        p(0x71A440, 0);
        p(0x71A44C, 0);
        for (from, to) in [
            (0x6138C4, 0x71A43C),
            (0x6138C8, 0x71A440),
            (0x6138CC, 0x71A444),
            (0x6138D0, 0x71A448),
        ] {
            p(to, g(from));
        }
        let keys = read_keys(g(0x5C694C), g(0x71A440));
        let pressed = keys.pressed;
        p(0x71A44C, pressed);
        p(0x613894, g(0x71A43C));
        p(0x613890, g(0x71A444));
        p(0x613898, g(0x71A448));
        p(0x61389C, keys.hand);
        p(0x6138A0, keys.up);
        r.eax = keys.down;
        p(0x6138A4, r.eax);
        let replay = g(0x52A910) != 0;
        if !replay {
            r.eax = g(CURRENT);
        }
        if replay || g(CURRENT) < g(0x63DB94) {
            r.eax = pressed & FLIP_BIT;
            if r.eax != 0 {
                p(0x71A05C, flipped(g(0x71A05C)));
            }
            r.eax = pressed & VIEW_BIT;
            if r.eax != 0 {
                r.esi = a(0x719FD8);
                next_camera_view(r.esi).apply(r);
            }
        }
        if !computer() {
            return Ok(());
        }
        r.eax = g(CURRENT);
        p(0x71C0CC, r.eax);
        r.eax = start_blend();
        p(0x71BFA4, x87::loaded(g(0x71A658)));
        drive_computer_car()?;
        for (k, to) in [
            0x613890u32,
            0x613894,
            0x613898,
            0x61389C,
            0x6138A0,
            0x6138A4,
        ]
        .into_iter()
        .enumerate()
        {
            p(to, g(0x71BF88 + 4 * k as u32));
        }
        Ok(())
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { read_car_controls(&mut g) }?;
    Ok(Left::nothing().with(EAX, g.eax).with(ESI, g.esi))
}

/// Everything it writes.
pub fn regions() -> Vec<Region> {
    let mut r = crate::drive::drive_computer_car::regions();
    r.extend([
        (a(0x5C694C), 4),
        (a(0x71A43C), 0x14),
        (a(0x613890), 0x18),
        (a(0x71A05C), 4),
        (a(0x719FD8 + 0x684), 0x1C),
        (a(0x71A654), 8),
    ]);
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !crate::physics::move_slot::in_race() || g(0x623640) < 0x20 {
        return t.skip("not in a race (no stage's checkpoints)");
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43A35A,
            len: 0x1EC,
            entry: &[0xFF, 0x35, 0x40, 0xA4, 0x71, 0x00],
            calls: &[0xFA, 0x12C, 0x13D, 0x19D],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let inputs = [
        0x6138C4u32,
        0x6138C8,
        0x6138CC,
        0x6138D0,
        0x52A910,
        CURRENT,
        0x63DB94,
        0x71C3E0,
        0x62389C,
        0x71A234,
        0x71A070,
    ];
    let mut all = regions();
    all.extend(inputs.iter().map(|&v| (a(v), 4)));
    let saved = capture(&all);
    for seed in 1..=16u32 {
        let mut x = seed.wrapping_mul(0x9E37_79B9) | 1;
        let mut next = move || {
            x = x.wrapping_mul(1_103_515_245).wrapping_add(12345);
            x >> 8
        };
        p(0x6138C4, next() % 0x200);
        p(0x6138C8, next() & 0x2F);
        p(0x6138CC, next() % 0x200);
        p(0x6138D0, next() % 0x200);
        p(0x71A440, next() & 0x2F);
        p(0x52A910, u32::from(seed % 5 == 0));
        p(CURRENT, next() % 4);
        p(0x63DB94, 1 + next() % 2);
        p(0x71C3E0, u32::from(seed % 3 == 0));
        p(0x62389C, next() % 4);
        p(0x71A234, next() % 3);
        p(0x71A070, next() % g(0x623640));
        let input = capture(&all);
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = capture(&all);
        restore(&all, &input);
        let mut gr = Regs::SENTINEL;
        let mine = unsafe { read_car_controls(&mut gr) };
        let state = capture(&all);
        restore(&all, &saved);
        let regs_ok = mine.is_ok()
            && Left::nothing()
                .with(EAX, gr.eax)
                .with(ESI, gr.esi)
                .matches(&Regs::SENTINEL, &ro);
        t.check(regs_ok && theirs == state, || {
            let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
            format!("seed {seed}: regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}")
        });
    }
    drop(copy);
    t.finish()
}
