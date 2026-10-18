//! `drive::finish_camera` (0x45941D): the finish routine's camera from the eased time
//! (`flt_64BB68`) and the share (`flt_64BB6C`): `dword_606748`, `606730`, `606734`,
//! `606744` (each a line in them, `flt_64BB4C` .. `flt_64BB7C`), `dword_60672C` 0.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `drive::finish_run`.
//! Self-test: the original and the Rust version on the figures by lot, under three
//! precisions.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::flow::{a, base, p, x87_run};
use crate::scene::matrix3::test_floats;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
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

/// The Rust version.
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    finish_camera();
    Ok(Left::nothing())
}

fn regions() -> [crate::service::harness::Region; 3] {
    [(a(0x60672C), 0xC), (a(0x606744), 8), (a(0x62400C), 4)]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions().to_vec();
    all.push((a(0x64BB4C), 0x34));
    let saved = capture(&all);
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for seed in 0..8u32 {
                let set = || {
                    let mut f = [0u32; 13];
                    test_floats(seed * 7 + 3, &mut f);
                    for (k, v) in f.iter().enumerate() {
                        p(0x64BB4C + 4 * k, *v);
                    }
                };
                check_state(
                    &mut t,
                    &format!("control word {word:#x}, seed {seed}"),
                    call_original,
                    Regs::SENTINEL,
                    &set,
                    &regions(),
                    &rust,
                );
            }
        });
    }
    restore(&all, &saved);
    t.finish()
}
