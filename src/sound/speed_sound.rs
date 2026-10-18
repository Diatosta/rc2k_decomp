//! `sound::speed_sound` (0x40DE09): a player's car's sound by its speed (+0x108, from
//! `sound::car_engine_state`), at the car.
//!
//! In:  ebx the car's sound object, edi its sound kind, esi the request
//! Out: eax the object's +0 (+ set +0 when stopped; +0x34 when placed), ebp the set
//!      (kind +0x60); everything else preserved
//! Nothing for a computer's car (+0 negative). The sound: kind +0x60's +0 + the
//! object's +0, into the request. With the state (+0x10C) 0 or 1, +0x24 1 (no gear),
//! +0xF4 or +0xAC not 0: `sound::stop_sound`. Else the request's +4 = +0x108 x set
//! +0x18 + set +0x10, +8 = +0x108 x set +0x24 + set +0x1C (x `flt_51637C` when +0xC is
//! 0), the place = +0x2C .. +0x34, and `sound::play_placed_sound`. Both are tail jumps.
//! One caller, `sound::car_sounds`.
//! A near `js` in its first five bytes: private copy (0x92 bytes, the six jumps out
//! relocated), manual patch.
//! Refuses (before anything) a null object or request, and a kind wanted but null.
//! Shadow mode runs the original alone (it sends a sound).
//! Self-test: a copy whose jumps out go to stand-ins, against the Rust version
//! (`turn_parts::test_car_sound`).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{base, x87_run};
use crate::sound::turn_parts::{PLACED, Part, STOP, dispatch_turn, test_car_sound};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::speed_sound", rva(0x40DE09), thunk, &ORIGINAL)
    .private_copy(LEN)
    .relocate_calls(&OUTS)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LEN: usize = 0x92;
/// The jumps to `sound::stop_sound` and the one to `sound::play_placed_sound`.
const OUTS: [usize; 6] = [0x19, 0x26, 0x30, 0x3D, 0x4A, 0x8C];
const ENTRY: [u8; 10] = [0x8B, 0x03, 0x85, 0xC0, 0x0F, 0x88, 0x87, 0x00, 0x00, 0x00];

// ---- Rust version ----

pub fn speed_sound_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    if c.ebx == 0 || c.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = c.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = c.esi as *mut u32;
    c.eax = w(0);
    if (c.eax as i32) < 0 {
        return Ok(());
    }
    if c.edi == 0 {
        return Err(BadInput("no sound kind").into());
    }
    c.ebp = c.edi + 0x60;
    c.eax = c
        .eax
        .wrapping_add(unsafe { (c.ebp as *const u32).read_unaligned() });
    unsafe { q.write_unaligned(c.eax) };
    if w(0x10C) == 0 || w(0x10C) == 1 || w(0x24) == 1 || w(0xF4) != 0 || w(0xAC) != 0 {
        return part(STOP, c);
    }
    x87_run!(
        "fld dword ptr [{bx} + 0x108]",
        "fmul dword ptr [{bp} + 0x18]",
        "fadd dword ptr [{bp} + 0x10]",
        "fstp dword ptr [{si} + 0x4]";
        bx = c.ebx, bp = c.ebp, si = c.esi
    );
    if w(0xC) == 0 {
        x87_run!(
            "fld dword ptr [{bx} + 0x108]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fmul dword ptr [{b} + 0x11637C]",
            "fstp dword ptr [{si} + 0x8]";
            bx = c.ebx, bp = c.ebp, si = c.esi, b = base()
        );
    } else {
        x87_run!(
            "fld dword ptr [{bx} + 0x108]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fstp dword ptr [{si} + 0x8]";
            bx = c.ebx, bp = c.ebp, si = c.esi
        );
    }
    // `mov eax, [ebx+..]; mov [esi+..], eax`: eax is left at the last.
    for k in 0..3 {
        c.eax = w(0x2C + 4 * k as u32);
        unsafe { q.add(3 + k).write_unaligned(c.eax) };
    }
    part(PLACED, c)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, speed_sound_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_car_sound(
        &mut t,
        0x40DE09,
        LEN,
        &ENTRY,
        &OUTS,
        speed_sound_in,
        &[(0x24, &[0, 1]), (0xF4, &[0, 2]), (0xAC, &[0, 1])],
    );
    t.finish()
}
