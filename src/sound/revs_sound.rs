//! `sound::revs_sound` (0x40DD19): a car's engine sound by its revs, placed ahead of the
//! car.
//!
//! In:  ebx the car's sound object, edi its sound kind, esi the request
//! Out: eax the sound, ebp its set (kind +0x10, or `unk_516230` for a computer's car);
//!      everything else preserved
//! The sound: the set's +0 + the object's +0 (a computer's car, +0 negative: + its
//! bit 0, from `unk_516230`), into the request. With the state (+0x10C) 0 or 3:
//! `sound::stop_sound`. Else the request's +4 (pitch) = +0x100 x set +0x18 + set +0x10,
//! +8 (volume) = +0x100 x set +0x24 + set +0x1C (x `flt_516380` when +0xC is 0), the
//! place = +0x2C .. +0x34 plus +0x38 .. +0x40, and `sound::play_placed_sound`. Both
//! are tail jumps. One caller, `sound::car_sounds`.
//! A short jump in its first five bytes: private copy (0x7D bytes, the three jumps out
//! relocated), manual patch.
//! Refuses (before anything) a null object or request, and a kind wanted but null.
//! Shadow mode runs the original alone (it sends a sound).
//! Self-test: a copy whose jumps out go to stand-ins, against the Rust version
//! (`turn_parts::test_car_sound`).

use crate::errors::{BadInput, Refused};
use crate::framework::game_addr;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{base, x87_run};
use crate::sound::turn_parts::{PLACED, Part, STOP, dispatch_turn, test_car_sound};
use rc2k_core::sound::engine::{computers, revs_sound_number, revs_stopped};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("sound::revs_sound", rva(0x40DD19), thunk, &ORIGINAL)
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

const LEN: usize = 0x7D;
/// The jumps to `sound::stop_sound` and the one to `sound::play_placed_sound`.
const OUTS: [usize; 3] = [0x15, 0x22, 0x6D];
const ENTRY: [u8; 6] = [0x8B, 0x03, 0x85, 0xC0, 0x78, 0x6C];
/// unk_516230: the computer's cars' set.
const COMPUTERS: usize = 0x516230;

// ---- Rust version ----

pub fn revs_sound_in(c: &mut Regs, part: Part) -> Result<(), Refused> {
    if c.ebx == 0 || c.esi == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let o = c.ebx;
    let w = |at: u32| unsafe { ((o + at) as *const u32).read_unaligned() };
    let q = c.esi as *mut u32;
    // Converted: `rc2k_core::sound::engine` (`computers`, `revs_sound_number`,
    // `revs_stopped`).
    let object = w(0);
    if computers(object) {
        c.ebp = game_addr(rva(COMPUTERS)) as u32;
    } else {
        if c.edi == 0 {
            return Err(BadInput("no sound kind").into());
        }
        c.ebp = c.edi + 0x10;
    }
    c.eax = revs_sound_number(object, unsafe { (c.ebp as *const u32).read_unaligned() });
    unsafe { q.write_unaligned(c.eax) };
    if revs_stopped(w(0x10C)) {
        return part(STOP, c);
    }
    x87_run!(
        "fld dword ptr [{bx} + 0x100]",
        "fmul dword ptr [{bp} + 0x18]",
        "fadd dword ptr [{bp} + 0x10]",
        "fstp dword ptr [{si} + 0x4]";
        bx = c.ebx, bp = c.ebp, si = c.esi
    );
    if w(0xC) == 0 {
        x87_run!(
            "fld dword ptr [{bx} + 0x100]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fmul dword ptr [{b} + 0x116380]",
            "fstp dword ptr [{si} + 0x8]";
            bx = c.ebx, bp = c.ebp, si = c.esi, b = base()
        );
    } else {
        x87_run!(
            "fld dword ptr [{bx} + 0x100]",
            "fmul dword ptr [{bp} + 0x24]",
            "fadd dword ptr [{bp} + 0x1C]",
            "fstp dword ptr [{si} + 0x8]";
            bx = c.ebx, bp = c.ebp, si = c.esi
        );
    }
    x87_run!(
        "fld dword ptr [{bx} + 0x2C]",
        "fadd dword ptr [{bx} + 0x38]",
        "fstp dword ptr [{si} + 0xC]",
        "fld dword ptr [{bx} + 0x30]",
        "fadd dword ptr [{bx} + 0x3C]",
        "fstp dword ptr [{si} + 0x10]",
        "fld dword ptr [{bx} + 0x34]",
        "fadd dword ptr [{bx} + 0x40]",
        "fstp dword ptr [{si} + 0x14]";
        bx = c.ebx, si = c.esi
    );
    part(PLACED, c)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_turn(&HOOK, r, call_original, revs_sound_in, None);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_car_sound(&mut t, 0x40DD19, LEN, &ENTRY, &OUTS, revs_sound_in, &[]);
    t.finish()
}
