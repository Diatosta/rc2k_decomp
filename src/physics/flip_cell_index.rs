//! `physics::flip_cell_index` (0x43DB13): a 2 x 2 cell (ebx, edx, each 0 or 1) turned by
//! one of eight ways (eax & 7: 1 flips ebx, 2 flips edx, 3 both; 4 .. 7 swap the two
//! first, then 4 flips edx, 5 both, 6 none, 7 ebx), then edx = 2 edx + ebx.
//!
//! In:  eax the way, ebx and edx the cell
//! Out: eax & 7, ebx and edx as worked out; the rest kept; no flags read (callers store
//!      edx)
//!
//! Caller: `sub_43D59A` (twice).
//! Self-test: a copy of the original against the core, every way with cells 0 .. 3 and
//! odd values.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, EDX, RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::flip_cell_index", rva(0x43DB13), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: (eax, ebx, edx) out.
// Converted: `rc2k_core::physics::ground::flip_cell_index`.
pub use rc2k_core::physics::ground::flip_cell_index;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (a, b, d) = flip_cell_index(r.eax, r.ebx, r.edx);
    Ok(Left::nothing().with(EAX, a).with(EBX, b).with(EDX, d))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x43DB13,
            len: 0x5E,
            entry: &[0x83, 0xE0, 0x07, 0x83, 0xF8, 0x00],
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
    for way in [0u32, 1, 2, 3, 4, 5, 6, 7, 0xF, 0xFFFF_FFFC] {
        for (b, d) in [(0u32, 0u32), (0, 1), (1, 0), (1, 1), (2, 0x8000_0001)] {
            let before = Regs {
                eax: way,
                ebx: b,
                edx: d,
                ..Regs::SENTINEL
            };
            let mut ro = before;
            unsafe { call_game(&mut ro, copy.addr()) };
            let (a2, b2, d2) = flip_cell_index(way, b, d);
            let ok = Left::nothing()
                .with(EAX, a2)
                .with(EBX, b2)
                .with(EDX, d2)
                .matches(&before, &ro);
            t.check(ok, || {
                format!("way {way:#x}, cell ({b:#x}, {d:#x}): original ({:#x}, {:#x}, {:#x}); rust ({a2:#x}, {b2:#x}, {d2:#x})", ro.eax, ro.ebx, ro.edx)
            });
        }
    }
    drop(copy);
    t.finish()
}
