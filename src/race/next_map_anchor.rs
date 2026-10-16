//! `race::next_map_anchor` (0x445FEE): the next checkpoint whose face has a place on the
//! map.
//!
//! In:  ecx = the checkpoint to look from
//! Out: carry clear: ecx the anchor, ebx and esi its face's number and address; carry set:
//!      ecx `g_checkpointCount`, ebx and esi the last face looked at; the rest preserved
//!
//! See `race/best_line.rs`. Callers: `sub_445FB4` (twice, one of them inside itself: the
//! original loops by jumping back to its own start). Refuses more checkpoints than the
//! lists hold. Self-test: faces and checkpoints of the test's, the game's put back.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, ESI, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::next_map_anchor", rva(0x445FEE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (g, carry) = unsafe { next_map_anchor(r) }?;
    let mut left = Left::carry(carry).with(ECX, g.ecx);
    if g.ebx != r.ebx || g.esi != r.esi {
        left = left.with(EBX, g.ebx).with(ESI, g.esi);
    }
    Ok(left)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(ANCHORS_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let line = TestLine::new(24, 20, &|k| (k * 7 + 3) % 24, &|n, f| {
        f.fill(0x5A);
        let anchor = n % 5 == 1;
        f[0x2C..0x30].copy_from_slice(&(if anchor { 0x0102_0304u32 } else { 0 }).to_le_bytes());
    });
    let regions = [line.faces.region(), at(ANCHORS_RVA)];
    let saved = capture(&[at(ANCHORS_RVA)]);
    for from in [0u32, 1, 5, 19, 20, 0x8000_0000] {
        check_state(
            &mut t,
            &format!("from {from:#x}"),
            call_original,
            Regs {
                ecx: from,
                ..Regs::SENTINEL
            },
            &|| unsafe { game_write(ANCHORS_RVA, 3u32) },
            &regions,
            &rust,
        );
    }
    restore(&[at(ANCHORS_RVA)], &saved);
    drop(line);
    t.finish()
}
