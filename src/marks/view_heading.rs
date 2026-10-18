//! `marks::view_heading` (0x4C4CEC): the view's heading: the angle between its forward
//! direction's level part (`flt_5EA3CC` +8, 0, +0x28, into `flt_91FF7E`) and
//! `flt_91FF8A` (`scene::angle_between_vectors`, both normalised in place), into
//! `flt_8F3975`; 2 pi less it when the level part's x (normalised) is not negative.
//!
//! In:  nothing
//! Out: esi 0x91FF7E, edi 0x91FF8A; the rest kept
//!
//! One caller, `sub_4C3EEE`. Self-test: the original and the Rust version on several
//! views and second vectors; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::rounded;
use crate::scene::vector3::{STATUS_RVA, angle_between_vectors};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::view_heading", rva(0x4C4CEC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view's matrix
const LEVEL_RVA: usize = rva(0x91FF7E); // flt_91FF7E: three floats
const OTHER_RVA: usize = rva(0x91FF8A); // flt_91FF8A: three floats
const HEADING_RVA: usize = rva(0x8F3975); // g_marksViewHeading

/// The Rust version (see the header).
pub unsafe fn view_heading(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let x = x87::loaded(game_read(VIEW_RVA + 8));
        let z = x87::loaded(game_read(VIEW_RVA + 0x28));
        game_write(LEVEL_RVA, [x, 0, z]);
        g.esi = game_addr(LEVEL_RVA) as u32;
        g.edi = game_addr(OTHER_RVA) as u32;
        let mut heading = rounded(angle_between_vectors(g.esi, g.edi));
        if game_read::<u32>(LEVEL_RVA) & 0x8000_0000 == 0 {
            heading = x87::two_pi_less(heading);
        }
        game_write(HEADING_RVA, heading);
    }
    g
}

pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(LEVEL_RVA) as u32, 0x18),
        at(HEADING_RVA),
        at(STATUS_RVA),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { view_heading(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut inputs = regions.clone();
    inputs.extend([at(VIEW_RVA + 8), at(VIEW_RVA + 0x28)]);
    let saved = capture(&inputs);
    let views: [(f32, f32); 6] = [
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.3, -0.9),
        (-0.7, 0.7),
        (0.0, 1.0),
        (-0.0, -2.5),
    ];
    let others: [[f32; 3]; 2] = [[0.0, 0.0, 1.0], [0.6, 0.0, -0.8]];
    for (x, z) in views {
        for other in others {
            check_state(
                &mut t,
                &format!("view x {x}, z {z}, other {other:?}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&inputs, &saved);
                    game_write(VIEW_RVA + 8, x.to_bits());
                    game_write(VIEW_RVA + 0x28, z.to_bits());
                    game_write(OTHER_RVA, other.map(f32::to_bits));
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&inputs, &saved);
    t.finish()
}
