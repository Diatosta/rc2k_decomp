//! `marks::copy_block_points` (0x4D0A82): block ebp: its object's points copied to its +0x8C.
//!
//! In:  ebp the block
//! Out: eax the last dword copied (the count with none), esi and edi past the points
//!
//! See `marks/lines.rs`. One caller, `marks::set_up_blocks`. Refuses an object past 0xFF.
//! Self-test: the original and the Rust version on a scene of the test's (0, 1, 3 and 0x20
//! points of either object), the refusal; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::marks::lines::*;
use crate::race::stage_prefs::left_of;
use crate::scene::objects::POINT_COUNTS_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::copy_block_points", rva(0x4D0A82), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { copy_block_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = point_regions(r.ebp);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let scene = TestScene::new();
    let out = vec![0xAAu8; (12 * TEST_OBJECT_POINTS + 16) as usize];
    let block = scene.block(2);
    for (n, object) in TEST_OBJECTS.into_iter().enumerate() {
        for count in [0u32, 1, 3, TEST_OBJECT_POINTS] {
            let prepare = || unsafe {
                scene.fill(n as u32 * 0x10 + count, 8);
                std::ptr::write_bytes(out.as_ptr() as *mut u8, 0xAA, out.len());
                ((block + 4 * 0x23) as *mut u32).write_unaligned(out.as_ptr() as u32);
                (block as *mut u32).write_unaligned(object);
                game_write(POINT_COUNTS_RVA + 4 * object as usize, count);
            };
            check_state(
                &mut t,
                &format!("object {object:#x}, {count} points"),
                call_original,
                Regs {
                    ebp: block,
                    ..Regs::SENTINEL
                },
                &prepare,
                &[(out.as_ptr() as u32, out.len())],
                &rust,
            );
        }
    }
    unsafe { (block as *mut u32).write_unaligned(0x100) };
    t.check(
        unsafe {
            copy_block_points(&Regs {
                ebp: block,
                ..Regs::SENTINEL
            })
        }
        .is_err(),
        || "object 0x100: the Rust version went on".into(),
    );
    drop(scene);
    t.finish()
}
