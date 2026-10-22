//! `hud::tenths` (0x4B1D2B): six times of the HUD's record in tenths (signed `idiv` by
//! 10): +0x24, +0x2C, +0x20, +0x54, +0x58, +0x50 into `dword_75C074` .. `dword_75C088`.
//!
//! In:  ebp the HUD's record
//! Out: eax the last quotient, edx its remainder, ebx 10; the rest kept
//!
//! Caller: `hud::draw_hud`. Self-test: the original and the Rust version on records of the
//! test's, negative times among them.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::tenths", rva(0x4B1D2B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const OUT_RVA: usize = rva(0x75C074); // dword_75C074 .. dword_75C088
const FIELDS: [u32; 6] = [0x24, 0x2C, 0x20, 0x54, 0x58, 0x50];

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn tenths(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::hud::tenths::tenths`.
    let mut g = *entry;
    let times = FIELDS.map(|f| unsafe { ((g.ebp + f) as *const i32).read_unaligned() });
    let (tenths, remainder) = rc2k_core::hud::tenths::tenths(times);
    unsafe { game_write(OUT_RVA, tenths) };
    [g.eax, g.edx, g.ebx] = [tenths[5] as u32, remainder as u32, 10];
    g
}

fn regions() -> Vec<Region> {
    vec![(game_addr(OUT_RVA) as u32, 0x18)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { tenths(r) }))
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
    use crate::service::harness::{capture, restore};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    for case in 0..4u32 {
        let mut record = [0u32; 0x18];
        for (n, f) in FIELDS.into_iter().enumerate() {
            let v = (12_345 * (case + 1) + 77 * n as u32) as i32;
            record[f as usize / 4] = if (case + n as u32) % 2 == 1 { -v } else { v } as u32;
        }
        record[0x20 / 4] = if case == 3 {
            0x8000_0000
        } else {
            record[0x20 / 4]
        };
        let rp = record.as_ptr() as u32;
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs {
                ebp: rp,
                ..Regs::SENTINEL
            },
            &|| (),
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
