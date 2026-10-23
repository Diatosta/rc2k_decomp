//! `physics::ease_view_turn` (0x438B4F): the camera's three turns (`unk_71A688`,
//! `71A690`, `71A68C`) each moved toward its target (`dword_608B50`, `608B58`,
//! `608B54`) by its share (`dword_608BE8`, `608BE4`, `608BEC`; all 1 first when
//! `dword_6EF7CC` is set): the difference `fild`ed, times the share, `fistp`ed, added.
//!
//! In:  nothing
//! Out: eax the last step; the rest kept; no flags read
//!
//! Caller: `sub_43907F`.
//! Self-test: `physics/camera_view.rs` (`view_test_floats`), figures by lot, exact at
//! double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, Regs};
use crate::physics::flow::{base, g, p, x87_run};
use crate::screens::harness::Left;

/// The Rust version (the exact one, for `physics::camera_avoid_ground`).
pub unsafe fn ease_view_turn(_: &Regs) -> Left {
    if g(0x6EF7CC) != 0 {
        for x in [0x608BE4, 0x608BE8, 0x608BEC] {
            p(x, 0x3F80_0000);
        }
    }
    let mut eax = 0;
    for (target, turn, share) in [
        (0x608B50, 0x71A688, 0x608BE8),
        (0x608B58, 0x71A690, 0x608BE4),
        (0x608B54, 0x71A68C, 0x608BEC),
    ] {
        p(0x62400C, g(target).wrapping_sub(g(turn)));
        x87_run!(
            "fild dword ptr [{b} + 0x22400C]",
            "fmul dword ptr [{s}]",
            "fistp dword ptr [{b} + 0x22400C]";
            b = base(), s = crate::physics::flow::a(share)
        );
        eax = g(0x62400C);
        p(turn, g(turn).wrapping_add(eax));
    }
    Left::nothing().with(EAX, eax)
}
