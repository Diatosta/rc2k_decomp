//! `physics::face_shares` (0x442323): for the face corners named by `dword_639890` ..
//! `98` (points of `g_scenePoints`, x y z whole numbers) and the point (`dword_639824`,
//! `dword_63982C`): the corners copied to 0x639708 .. 0x63972B, the point to 0x6396FC /
//! 0x639704, the differences from the first corner (0x639738 .. 0x63975B), two cross
//! products over a third (0x63975C, 0x639760, 0x639764), their quotients (shares,
//! `flt_639810`, `flt_639814`) and those times `flt_5C5D98` (65536) as whole numbers
//! (0x639800, 0x639804). With `g_useSse` the quotient is by `rcpss` (0x639768).
//!
//! In:  nothing
//! Out: eax = 0x63974C's difference, ebx, ecx the third corner's y, z; the rest kept
//!      (flags not read: callers go on with `mov` and calls)
//!
//! See `physics/ground.rs`. Callers: `physics::face_under_point` (when inside), and
//! `sub_457B86`.
//! Self-test: a copy of the original on the test's ground (`TestGround`), every corner
//! triple of a grid's faces against points in and out, both ways (SSE switch off and
//! on), under three precisions.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, ECX, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::physics::ground::*;
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::face_shares", rva(0x442323), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let mut g = *r;
    unsafe { face_shares_at(&Ground::game(), &mut g) };
    Ok(Left::nothing()
        .with(EAX, g.eax)
        .with(EBX, g.ebx)
        .with(ECX, g.ecx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &Ground::game().regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut test = TestGround::new(16, 25, 1);
    let copy = match test.copy(FACE_SHARES, &|_| None) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    test.lay_out(3);
    let g = test.ground();
    for word in TEST_CONTROL_WORDS {
        with_control_word(word, &mut || {
            for sse in [false, true] {
                for (n, &(x, z)) in test_points().iter().enumerate() {
                    let face = (n as u32 * 5) % 16;
                    let f = &test.faces[(face * FACE_LEN) as usize..];
                    let corner =
                        |k: usize| u32::from_le_bytes(f[4 + 4 * k..8 + 4 * k].try_into().unwrap());
                    let prepare = || unsafe {
                        test.clear_work();
                        test.set_point(x, z, 0, sse);
                        for k in 0..3 {
                            g.put(0x639890 + 4 * k as u32, corner(k));
                        }
                    };
                    check_case(
                        &mut t,
                        &format!(
                            "control word {word:#x}, sse {sse}, face {face}, point ({x:#x}, {z:#x})"
                        ),
                        copy.addr(),
                        &test,
                        Regs::SENTINEL,
                        &prepare,
                        &|g, r| {
                            unsafe { face_shares_at(g, r) };
                            Ok(false)
                        },
                        false,
                    );
                }
            }
        });
    }
    drop(copy);
    t.finish()
}
