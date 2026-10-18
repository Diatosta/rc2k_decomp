//! `race::view_light` (0x455896): the light at the work car (`g_car` +0x134 .. into
//! `g_litPlace` ..), `race::light_colour_at`, then `dword_6415E4` = `dword_62DAD4` plus that
//! light byte by byte (through `dword_62400C`; three bytes without MMX, four with).
//!
//! In:  nothing
//! Out: eax the sum; the rest as `light_colour_at` leaves them (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_4557AB` (0x4557F4), each view drawn. Refuses as
//! `light_colour_at` (before anything). Shadow mode: by state. Self-test: as
//! `light_colour_at`'s, the work car placed and `dword_62DAD4` set; all put back (skipped
//! with a null block).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::race::light_colour_at::{LIGHT_CASES, LightStage, colour_regions};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::view_light", rva(0x455896), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut v = colour_regions();
    v.extend([(game_addr(LIT_PLACE_RVA) as u32, 0xC), at(rva(0x6415E4))]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { view_light(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let extra = vec![
        (game_addr(rva(0x71A10C)) as u32, 0xC),
        at(rva(0x62DAD4)),
        at(rva(0x6415E4)),
    ];
    let Some(stage) = LightStage::new(&extra) else {
        return t.skip("a car block is null (no race loaded)");
    };
    let regions = regions();
    for (k, c) in LIGHT_CASES.iter().enumerate() {
        check_state(
            &mut t,
            &format!("case {k}: {c:?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                stage.prepare(c);
                let car = game_addr(rva(0x71A10C)) as u32;
                (car as *mut u32).write_unaligned(0x10_1000 + k as u32 * 0x3000);
                ((car + 4) as *mut u32).write_unaligned(0x1_0000);
                ((car + 8) as *mut u32).write_unaligned(0x17_F000);
                game_write(rva(0x62DAD4), 0xFF20_8010u32 ^ c.seed);
            },
            &regions,
            &rust,
        );
    }
    stage.end();
    t.finish()
}
