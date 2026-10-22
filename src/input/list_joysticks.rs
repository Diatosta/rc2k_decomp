//! `input::list_joysticks` (0x408216): unless the configuration turned them off
//! (`dword_4F68DC` not 0), the attached joysticks listed into `g_controllers`
//! (`EnumDevices` of `dword_4F6904`, DirectInput, for joysticks (4), attached only (1), with
//! `input::joystick_found`, context 0; its result not looked at).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `input::input_thread` (its start). The method DirectInput's own (in the
//! standalone program Rust's, `input::di_objects`). Shadow mode runs the original alone.
//! Self-test: the original and the Rust version on DirectInput objects made in Rust
//! (`input::di_harness`) with the joystick and the wheel attached: listed from none, from
//! 15 (one room left) and turned off.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::input::di_harness::{
    CONTROLLER_COUNT_RVA, CONTROLLER_LEN, CONTROLLERS, CONTROLLERS_RVA, DIRECT_INPUT_RVA, a,
    compare, g, method, p, regs_text, shared_regions, slot,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::list_joysticks", rva(0x408216), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const OFF_RVA: usize = rva(0x4F68DC); // dword_4F68DC: joysticks turned off

/// The Rust version.
pub fn list_joysticks() {
    if g(OFF_RVA) != 0 {
        return;
    }
    let found = crate::input::joystick_found::callback as unsafe extern "system" fn(u32, u32) -> u32
        as usize as u32;
    method(g(DIRECT_INPUT_RVA), slot::ENUM_DEVICES, &[4, found, 0, 1]);
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    list_joysticks();
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let world = crate::input::di_objects::TestWorld::begin(
        crate::input::di_objects::test_devices(),
        Default::default(),
    );
    let mut regions = shared_regions();
    regions.push((a(OFF_RVA), 4));
    regions.push((a(crate::input::joystick_found::NO_FORCE_RVA), 4));
    for (count, off) in [(0u32, 0u32), (15, 0), (0, 1)] {
        let entry = Regs::SENTINEL;
        compare(
            &mut t,
            &format!("{count} before, turned off {off}"),
            &world,
            &regions,
            &|| {
                p(DIRECT_INPUT_RVA, crate::input::di_objects::create_input());
                p(CONTROLLER_COUNT_RVA, count);
                p(OFF_RVA, off);
                p(crate::input::joystick_found::NO_FORCE_RVA, 0);
                let table = a(CONTROLLERS_RVA) as *mut u8;
                unsafe { std::ptr::write_bytes(table, 0, (CONTROLLERS * CONTROLLER_LEN) as usize) };
            },
            &mut || {
                let mut r = entry;
                unsafe { call_original(&mut r) };
                regs_text(&r)
            },
            &mut || {
                list_joysticks();
                regs_text(&entry)
            },
        );
    }
    drop(world);
    t.finish()
}
