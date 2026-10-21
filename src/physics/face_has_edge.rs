//! `physics::face_has_edge` (0x45DA72): whether the edge (eax, ebx) is one of a face's,
//! by its corner words at esi +4 .. +0x10 (p1 .. p4): CF clear when (eax, ebx) is (p1,
//! p3), (p2, p1), (p4, p2) or (p3, p4), tested in that order; else CF.
//!
//! In:  eax, ebx the edge's two corners, esi the face
//! Out: CF; every register kept (ZF as the last compare leaves it, not read: callers
//!      `jnb`)
//!
//! Caller: `sub_45DA14` (four calls). It begins with a short jump in its first five
//! bytes: hooked by a private copy and a hand-made patch.
//! Self-test: a copy of the original on a test face, every pair of corners and some
//! that are not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use rc2k_core::physics::faces::has_edge;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("physics::face_has_edge", rva(0x45DA72), thunk, &ORIGINAL)
    .private_copy(0x2C)
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

unsafe fn corners(face: u32) -> [u32; 4] {
    let at = |o: u32| unsafe { ((face + o) as *const u32).read_unaligned() };
    [at(4), at(8), at(0xC), at(0x10)]
}

/// The Rust version.
// Converted: `rc2k_core::physics::faces::has_edge`.
pub unsafe fn face_has_edge(r: &Regs) -> FlagResult {
    let found = has_edge(r.eax, r.ebx, unsafe { corners(r.esi) });
    FlagResult {
        eax: r.eax,
        ecx: None,
        ebx: None,
        carry: !found,
        zero: None,
    }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { face_has_edge(r) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x45DA72,
            len: 0x2C,
            entry: &[0x3B, 0x46, 0x04, 0x75, 0x05],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let face = [0xAAAA_AAAAu32, 11, 22, 33, 44];
    let f = face.as_ptr() as u32;
    let values = [11u32, 22, 33, 44, 55];
    for &a in &values {
        for &b in &values {
            let before = Regs {
                eax: a,
                ebx: b,
                esi: f,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let mine = unsafe { face_has_edge(&before) };
            t.check(mine.matches(&before, &ro), || {
                format!(
                    "edge ({a}, {b}): original CF {}, rust {}",
                    ro.carry(),
                    mine.carry
                )
            });
        }
    }
    drop(copy);
    t.finish()
}
