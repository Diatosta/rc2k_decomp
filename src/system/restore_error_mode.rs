//! `system::restore_error_mode` (0x402B95): at the game's end, if the error mode was
//! changed (`dword_4E059C` 1, `sub_402B7C`, agent A's), the switch 0 and the mode it had
//! before (`uMode`) put back (`SetErrorMode`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out (0x401116, also the short one). `SetErrorMode` through
//! `api` (the same function in the standalone). Shadow mode runs the original alone.
//! Self-test: a copy whose two variables are the test's and whose `SetErrorMode` is a
//! noting stand-in, against the Rust version, with the switch 0, 1, 2.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::system::api_stand_ins::{Api, copy_with};
use crate::system::exit_parts::{Part, SET_ERROR_MODE, compare_exit, dispatch_exit, exit_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "system::restore_error_mode",
    rva(0x402B95),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CHANGED: usize = 0x4E059C; // dword_4E059C: 1 after `sub_402B7C` set the mode
const MODE: usize = 0x4E05A0; // uMode: the mode before

/// The Rust version on the switch at `changed` and the mode at `mode`.
pub fn restore_error_mode_with(
    _r: &mut Regs,
    changed: u32,
    mode: u32,
    api: Api,
) -> Result<(), Refused> {
    unsafe {
        if (changed as *const u32).read_volatile() != 1 {
            return Ok(());
        }
        (changed as *mut u32).write_volatile(0);
        api(SET_ERROR_MODE, &[(mode as *const u32).read_volatile()]);
    }
    Ok(())
}

fn in_game(r: &mut Regs, _part: Part, api: Api) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    restore_error_mode_with(r, a(CHANGED), a(MODE), api)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_exit(&HOOK, r, call_original, in_game);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    exit_entry(r, in_game)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u32; 2];
    let (changed, mode) = (mem.as_mut_ptr() as u32, mem.as_mut_ptr() as u32 + 4);
    // pusha; cmp dword_4E059C, 1
    let entry = [0x60, 0x83, 0x3D, 0x9C, 0x05, 0x4E, 0x00, 0x01];
    let copy = match copy_with(
        0x402B95,
        0x21,
        &entry,
        &[0x1A],
        &[],
        &[(0x1A, SET_ERROR_MODE, 1)],
        &[(CHANGED, 4, changed), (MODE, 4, mode)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for switch in [0u32, 1, 2] {
        compare_exit(
            &mut t,
            &format!("switch {switch}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe { (changed as *mut [u32; 2]).write_volatile([switch, 0x8003]) },
            &[(changed, 8)],
            &|r, _, api| restore_error_mode_with(r, changed, mode, api),
            false,
        );
    }
    let _ = &mut mem;
    t.finish()
}
