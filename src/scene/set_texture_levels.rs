//! `scene::set_texture_levels` (0x422426): a run of texture slots given a level and a tag.
//!
//! In:  ecx = how many, ebx = the first slot, eax = the level (edx with `dword_5CDADC`
//!      set), ebp = the tag
//! Out: ebx past the run, ecx 0; the rest preserved
//!
//! See `scene/texture_levels.rs`. 41 calls (`scene::init_texture_levels`, `sub_422457`).
//! Refuses no slots or slots past 0x400. Self-test: the original and the Rust version on
//! the game's tables, put back afterwards.

use crate::errors::Refused;
use crate::framework::{EBX, ECX, RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::scene::texture_levels::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_texture_levels", rva(0x422426), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { set_texture_levels(r) }?;
    Ok(Left::nothing().with(EBX, g.ebx).with(ECX, g.ecx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &tables(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut regions = tables();
    regions.push(at(OTHER_LEVELS_RVA));
    let saved = capture(&regions);
    for (count, first, other) in [
        (1u32, 0u32, 0u32),
        (4, 0x100, 0),
        (8, 0x3F8, 1),
        (0x10, 0x200, 7),
    ] {
        check_state(
            &mut t,
            &format!("{count} from {first:#x}, other levels {other}"),
            call_original,
            Regs {
                ecx: count,
                ebx: first,
                eax: 3,
                edx: 5,
                ebp: 0x63,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&regions, &saved);
                game_write(OTHER_LEVELS_RVA, other);
            },
            &regions,
            &rust,
        );
    }
    let refused = unsafe {
        set_texture_levels(&Regs {
            ecx: 0,
            ebx: 5,
            ..Regs::SENTINEL
        })
    }
    .is_err()
        && unsafe {
            set_texture_levels(&Regs {
                ecx: 2,
                ebx: 0x3FF,
                ..Regs::SENTINEL
            })
        }
        .is_err();
    t.check(refused, || "no slots or past 0x400: not refused".into());
    restore(&regions, &saved);
    t.finish()
}
