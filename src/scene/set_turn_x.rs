//! `scene::set_turn_x` (0x426FE1): the turning matrix about the x axis for an angle.
//!
//! In:  edx = the angle, sixteen bits a turn
//! Out: eax = the sine, ebx = the cosine, ecx = the sine negated, edi = the matrix;
//!      everything else preserved
//!
//! The sine and cosine come from the game's two tables of 0x1000 floats (the angle's
//! low four bits dropped), are kept in `dword_624148` and `dword_62414C`, and go to the matrix
//! `TURN_X_RVA`: +0x14 the cosine, +0x18 the sine, +0x24 the sine negated, +0x28 the cosine.
//! Its other floats stay as they are.
//! 12 callers.
//! Self-test: 41 angles on the game's matrix, put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `set_turn_x_plain`, through
//! `rc2k_core::scene::matrices::set_table_turn`; compared exactly as before (the earlier
//! version stays for the Rust callers in unconverted code).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::scene::matrix::{self, matrix, test_floats};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_turn_x", rva(0x426FE1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn regions() -> [Region; 2] {
    [
        matrix(matrix::TURN_X_RVA),
        (game_addr(matrix::TURN_X_WORK_RVA) as u32, 8),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { matrix::set_turn_x_plain(r.edx) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let saved = capture(&regions);
    let mut seed = 0x1357_9BDFu32;
    for n in 0..41u32 {
        let angle = match n {
            0 => 0,
            1 => 0xF,
            2 => 0x10,
            3 => 0xFFFF,
            4 => 0x1_0000,
            5 => 0xFFFF_FFF0,
            6 => 0x8000_4000,
            n => n.wrapping_mul(0x0612_3457),
        };
        let floats = test_floats(&mut seed);
        check_state(
            &mut t,
            &format!("angle {angle:#x}"),
            call_original,
            Regs {
                edx: angle,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.len()]);
                (regions[0].0 as *mut [u32; 16]).write_unaligned(floats);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
