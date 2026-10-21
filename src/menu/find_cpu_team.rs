//! `menu::find_cpu_team` (0x4A49FF): the driver (`g_drivers`, 0x94 each) whose first dword
//! is eax (a reading: the computer team's number); none: the fatal error 0x8070A098
//! "CpuTeam Not Found (oh sh_t!)".
//!
//! In:  eax the number
//! Out: esi the driver; the rest kept
//!
//! Callers: `sub_4A47EA`, `sub_4A4933`. Refuses a table without its end entry, before
//! anything; raises the fatal error as the original. Self-test: the original and the Rust
//! version on the game's drivers, numbers there and (the fatal error checked in Rust
//! only) not.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::find_cpu_team", rva(0x4A49FF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn find_cpu_team(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let first = crate::framework::game_addr(DRIVERS_RVA) as u32;
    let all = entries(first, DRIVER)?;
    // Converted: `rc2k_core::menu::standings::find_team`.
    let drivers = unsafe { table(first, DRIVER, all.len() as u32) };
    if let Some(n) = rc2k_core::menu::standings::find_team(&drivers, g.eax) {
        g.esi = all[n];
        return Ok(g);
    }
    Err(crate::errors::FatalError {
        code: 0x8070A098,
        message: "CpuTeam Not Found (oh sh_t!)",
    }
    .into())
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { find_cpu_team(r) }?))
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![]
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    use crate::framework::SelfTest;
    use crate::service::harness::check_state;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let first = crate::framework::game_addr(DRIVERS_RVA) as u32;
    let Ok(all) = entries(first, DRIVER) else {
        return t.skip("the drivers' table without its end");
    };
    let mut numbers: Vec<u32> = all.iter().step_by(7).map(|&e| get(e)).collect();
    numbers.push(0xDEAD_0001);
    for n in numbers {
        let before = Regs {
            eax: n,
            ..Regs::SENTINEL
        };
        if !all.iter().any(|&e| get(e) == n) {
            let r = unsafe { find_cpu_team(&before) };
            t.check(
                matches!(r, Err(Refused::Fatal(e)) if e.code == 0x8070A098),
                || format!("number {n:#x}: {:?}", r.as_ref().err()),
            );
            continue;
        }
        check_state(
            &mut t,
            &format!("number {n:#x}"),
            call_original,
            before,
            &|| (),
            &[],
            &rust,
        );
    }
    t.finish()
}
