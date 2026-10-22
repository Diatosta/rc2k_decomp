//! `drive::finish_camera` (0x45941D): the finish routine's camera from the eased time
//! (`flt_64BB68`) and the share (`flt_64BB6C`): `dword_606748`, `606730`, `606734`,
//! `606744` (each a line in them, `flt_64BB4C` .. `flt_64BB7C`), `dword_60672C` 0.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::finish_run`.
//! Self-test: the original and the converted version on the figures by lot, exact at
//! double precision.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::{a, base, g, p, x87_run};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::floats::{
    FLOAT, Floats, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
    floats_entry,
};
use crate::service::harness::{capture, restore};
use rc2k_core::drive::finish_camera::{FinishLines, finish_camera as finish_camera_core};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::finish_camera", rva(0x45941D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (the exact one, for `drive::finish_run`).
pub fn finish_camera() {
    x87_run!(
        "fld dword ptr [{b} + 0x24BB68]",
        "fmul dword ptr [{b} + 0x24BB4C]",
        "fstp dword ptr [{b} + 0x206748]",
        "fld dword ptr [{b} + 0x24BB68]",
        "fmul dword ptr [{b} + 0x24BB5C]",
        "fadd dword ptr [{b} + 0x24BB54]",
        "fld dword ptr [{b} + 0x24BB6C]",
        "fmul dword ptr [{b} + 0x24BB7C]",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        "fstp dword ptr [{b} + 0x206730]",
        "fld dword ptr [{b} + 0x24BB68]",
        "fmul dword ptr [{b} + 0x24BB58]",
        "fadd dword ptr [{b} + 0x24BB50]",
        "fstp dword ptr [{b} + 0x206734]";
        b = base()
    );
    p(0x60672C, 0);
    x87_run!(
        "fld dword ptr [{b} + 0x24BB6C]",
        "fmul dword ptr [{b} + 0x24BB78]",
        "fstp dword ptr [{b} + 0x206744]";
        b = base()
    );
}

/// The converted version (`rc2k_core::drive::finish_camera`, plain floats): the hook's.
// Converted: `rc2k_core::drive::finish_camera::finish_camera`.
pub fn finish_camera_plain() {
    let f = |x: usize| f32::from_bits(g(x));
    let [a748, a730, a734, a744] = finish_camera_core(&FinishLines {
        k4c: f(0x64BB4C),
        k50: f(0x64BB50),
        k54: f(0x64BB54),
        k58: f(0x64BB58),
        k5c: f(0x64BB5C),
        time: f(0x64BB68),
        share: f(0x64BB6C),
        k78: f(0x64BB78),
        k7c: f(0x64BB7C),
    });
    p(0x606748, a748.to_bits());
    p(0x606730, a730.to_bits());
    p(0x606734, a734.to_bits());
    p(0x60672C, 0);
    p(0x606744, a744.to_bits());
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    finish_camera_plain();
    Ok(Left::nothing())
}

/// The float places, held to `FLOAT` at extended precision.
fn floats() -> [(crate::service::harness::Region, Tolerance); 2] {
    [((a(0x606730), 8), FLOAT), ((a(0x606744), 8), FLOAT)]
}

fn regions() -> [crate::service::harness::Region; 3] {
    [(a(0x60672C), 0xC), (a(0x606744), 8), (a(0x62400C), 4)]
}

extern "C" fn dispatch(r: &mut Regs) {
    let memory = floats();
    let floats = Floats {
        memory: &memory,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions().to_vec();
    all.push((a(0x64BB4C), 0x34));
    let saved = capture(&all);
    let memory = floats();
    let tolerant = Floats {
        memory: &memory,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, fl| {
        for seed in 0..8u32 {
            let set = || {
                let mut f = [0u32; 13];
                test_floats(seed * 7 + 3, &mut f);
                for (k, v) in f.iter().enumerate() {
                    p(0x64BB4C + 4 * k, *v);
                }
            };
            check_state_floats(
                &mut t,
                "drive::finish_camera",
                &format!("control word {word:#x}, seed {seed}"),
                call_original,
                Regs::SENTINEL,
                &set,
                &regions(),
                fl,
                &rust,
            );
        }
    });
    restore(&all, &saved);
    t.finish()
}
