//! `drive::countdown_beeps` (0x440E6C): `drive::countdown_beep` for the two cars of a
//! two-car race (`g_currentCar` 0 and 1, the blocks `g_carBlockStore` + 0 and + 0xFE0, the
//! seconds into `unk_6D1482` and `unk_6D1492`).
//!
//! In:  nothing
//! Out: esi the second block, edi `unk_6D1492`, the rest as the beeps leave them; no flags
//!      read
//!
//! Caller: `drive::replay_step`.
//! Refuses what `countdown_beep` refuses.
//! Self-test: the original and the Rust version on the two blocks' figures (saved and put
//! back), neither car with a sound slot.

use crate::errors::Refused;
use crate::framework::{EDI, ESI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, p};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::countdown_beeps", rva(0x440E6C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const B: usize = 0x70F3EC;

/// The Rust version.
pub unsafe fn countdown_beeps(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    for (car, block, to) in [(0u32, B, 0x6D1482usize), (1, B + 0xFE0, 0x6D1492)] {
        p(0x71B3F8, car);
        r.esi = a(block);
        r.edi = a(to);
        unsafe { crate::drive::countdown_beep::countdown_beep(r.esi, r.edi) }?.apply(&mut r);
    }
    Ok(crate::race::stage_prefs::left_of(&r)
        .with(ESI, r.esi)
        .with(EDI, r.edi))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { countdown_beeps(r) }
}

fn regions() -> [crate::service::harness::Region; 4] {
    [
        (a(0x6D1482), 4),
        (a(0x6D1492), 4),
        (a(0x614114), 4),
        (a(0x71B3F8), 4),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let all = [
        (a(B + 0x14), 12),
        (a(B + 0x25C), 4),
        (a(B + 0xFE0 + 0x14), 12),
        (a(B + 0xFE0 + 0x25C), 4),
        (a(0x6D1482), 4),
        (a(0x6D1492), 4),
        (a(0x614114), 4),
        (a(0x71B3F8), 4),
        (a(0x641680), 8),
        (a(0x6D1814), 4),
    ];
    let saved = capture(&all);
    for case in 0..6u32 {
        let set = || {
            p(0x641680, u32::MAX);
            p(0x641684, u32::MAX);
            p(0x6D1814, 500);
            for (k, block) in [B, B + 0xFE0].iter().enumerate() {
                p(block + 0x1C, 0);
                p(block + 0x25C, 1);
                p(block + 0x14, 500 + 100 * case + 37 * k as u32);
            }
        };
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &set,
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
