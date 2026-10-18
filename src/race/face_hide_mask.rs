//! `race::face_hide_mask` (0x456056): `physics::probe_ground`, then, unless the face under
//! the car (`g_car` +0xE00) is -1, `dword_64167C` from its +2 word: bits 1 and 3 flipped
//! when either is set, or'd with bits 4 and 5 likewise.
//!
//! In:  nothing
//! Out: as `probe_ground` leaves them, then ebx (the face, or its bits 4/5), esi (the face
//!      record), eax (the mask) when the face is there (a `call` follows)
//!
//! It begins with its call: private copy (0x3B, the call relocated), manual patch. See
//! `race/frame_draw.rs`. Caller: `race::prepare_course_view` (0x4548B2). Refuses what
//! `probe_ground` refuses (before anything). Shadow mode: by state (`probe_ground`'s
//! regions and the mask). Self-test: on the game's state (the ground as it is); all put
//! back (skipped without a stage). Its branches show in `stats` under play only.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::ground::Ground;
use crate::race::frame_draw::*;
use crate::race::gather_record_faces::stage_there;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::face_hide_mask", rva(0x456056), thunk, &ORIGINAL)
    .private_copy(0x3B)
    .relocate_calls(&[0x00])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn mask_regions() -> Vec<Region> {
    let mut v = crate::physics::probe_ground::regions(&Ground::game());
    v.push(at(rva(0x64167C)));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { face_hide_mask(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &mask_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if !stage_there() {
        return t.skip("no stage loaded");
    }
    let regions = mask_regions();
    let saved = capture(&regions);
    check_state(
        &mut t,
        "the game's ground",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions, &saved),
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
