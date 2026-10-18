//! `race::face_lights` (0x453715): the light at each of the face's corners
//! (`dword_63DA54` .., by `race::light_colour_at`; `sub_453786` covered here), when lamps
//! are wanted and the face is not a plain mode-4 one; else all 0 (see
//! `race/course_faces.rs`).
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `call` follows)
//!
//! Caller: `sub_452C6F`. Refuses (before anything) what `light_colour_at` refuses.
//! Shadow mode: by state. Self-test: the light test's scene (cars, lamps), corners of the
//! test's (points pointed at), the switches; all put back (skipped with a null block).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::course_faces::face_lights;
use crate::race::frame_draw::all_regs;
use crate::race::light_colour_at::{LIGHT_CASES, LightStage, colour_regions};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::face_lights", rva(0x453715), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { face_lights(r)? }))
}

fn regions() -> Vec<Region> {
    let mut v = colour_regions();
    v.extend([
        (game_addr(rva(0x63DA54)) as u32, 0x10),
        (game_addr(rva(0x636588)) as u32, 0xC),
    ]);
    v
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let extra = vec![
        at(rva(0x63DB58)),
        at(rva(0x63DA2C)),
        at(rva(0x63DAB4)),
        (game_addr(rva(0x63D9EC)) as u32, 0x10),
        (game_addr(rva(0x63DA54)) as u32, 0x10),
    ];
    let Some(stage) = LightStage::new(&extra) else {
        return t.skip("a car block is null (no race loaded)");
    };
    let regions = regions();
    for (k, (wanted, animated, mode, fourth)) in [
        (1u32, 0u32, 3u32, 3u32),
        (1, 1, 4, u32::MAX),
        (1, 0, 4, 3),
        (0, 1, 3, 3),
    ]
    .into_iter()
    .enumerate()
    {
        check_state(
            &mut t,
            &format!("wanted {wanted} animated {animated} mode {mode} fourth {fourth:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                stage.prepare(&LIGHT_CASES[k % LIGHT_CASES.len()]);
                game_write(rva(0x63DB58), wanted);
                game_write(rva(0x63DA2C), animated);
                game_write(rva(0x63DAB4), mode);
                for (n, v) in [0u32, 5, 9, fourth].iter().enumerate() {
                    game_write(rva(0x63D9EC + 4 * n), *v);
                }
            },
            &regions,
            &rust,
        );
    }
    stage.end();
    t.finish()
}
