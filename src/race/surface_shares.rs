//! `race::surface_shares` (0x445E30): how much of the best line runs on each surface.
//!
//! In:  nothing
//! Out: with two checkpoints or more, ecx 16, edi past the shares, edx the count, eax, ebx,
//!      esi as the last face leaves them; the rest preserved
//!
//! See `race/best_line.rs`. One caller, `sub_445772` (after "replaycam2"). Refuses more
//! checkpoints than the lists hold. Self-test: faces and checkpoints of the test's (the
//! game's texture infos read as they are), the game's put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::surface_shares", rva(0x445E30), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    vec![(game_addr(SHARES_RVA) as u32, 0x40), at(WEIGHT_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        surface_shares(r)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = regions();
    let saved = capture(&kept);
    for checks in [16u32, 2, 1] {
        let line = TestLine::new(40, checks, &|k| (k * 5 + 3) % 40, &|n, f| {
            for (k, b) in f.iter_mut().enumerate() {
                *b = (n * 31 + k * 13) as u8;
            }
            // Links: most to a face, some past the faces.
            let link = if n % 6 == 5 {
                0x9999
            } else {
                ((n * 7) % 40) as u32
            };
            f[0x3C..0x40].copy_from_slice(&link.to_le_bytes());
            if n % 4 == 1 {
                f[0x21] = 0;
            }
        });
        let mut regions = kept.clone();
        regions.push(line.faces.region());
        check_state(
            &mut t,
            &format!("{checks} checkpoints"),
            call_original,
            Regs::SENTINEL,
            &|| restore(&kept, &saved),
            &regions,
            &rust,
        );
        drop(line);
    }
    restore(&kept, &saved);
    t.finish()
}
