//! `input::request_device_setup` (0x409351): the first input device's record
//! (`g_inputDevices`[0]) given a controller and its settings, and the input thread asked to
//! give the device back and acquire it again (`g_inputCommand` 1, which
//! `input::device_command` takes with device 0): +0 eax (the controller), +0x20 ebx, +8
//! ebx or 2 for 0, +0x18 ecx, +0xC 0, +0x10 edx, +0x14 ebp.
//!
//! In:  eax = the controller, ebx = its sensitivity, ecx = its mode, edx, ebp (0 from the
//!      menus)
//! Out: every register kept (`pusha` .. `popa`)
//! Not DirectInput itself: the request is the input thread's to act on. Caller:
//! `menu::apply_controller`. Refuses (before anything) no first record (null pointer).
//! Self-test: the original and the Rust version on a record of the test's in the pointer's
//! place, ebx 0 and not, put back.

use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::input::devices::put;
use crate::input::input_tick::COMMAND_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "input::request_device_setup",
    rva(0x409351),
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

pub const DEVICES_RVA: usize = rva(0x4F6EE8); // g_inputDevices
/// The bytes of the record this writes.
pub const WRITTEN: usize = 0x24;

/// What it writes: the record and the command.
pub fn regions() -> Vec<Region> {
    let record = unsafe { game_read::<u32>(DEVICES_RVA) };
    let mut r = vec![at(COMMAND_RVA)];
    if record != 0 {
        r.push((record, WRITTEN));
    }
    r
}

/// The Rust version (see the header).
pub unsafe fn request_device_setup(
    controller: u32,
    sensitivity: u32,
    mode: u32,
    edx: u32,
    ebp: u32,
) -> Result<(), BadInput> {
    let record = unsafe { game_read::<u32>(DEVICES_RVA) };
    if record == 0 {
        return Err(BadInput("no first input device record"));
    }
    put(record, 0, controller);
    put(record, 0x20, sensitivity);
    put(record, 8, if sensitivity == 0 { 2 } else { sensitivity });
    put(record, 0x18, mode);
    put(record, 0xC, 0);
    put(record, 0x10, edx);
    put(record, 0x14, ebp);
    unsafe { game_write(COMMAND_RVA, 1u32) };
    Ok(())
}

fn rust(r: &Regs) -> Result<Left, crate::errors::Refused> {
    unsafe { request_device_setup(r.eax, r.ebx, r.ecx, r.edx, r.ebp) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [at(DEVICES_RVA), at(COMMAND_RVA)];
    let saved = capture(&inputs);
    let mut record = vec![0xAAu8; WRITTEN + 8];
    let pointer = record.as_mut_ptr() as u32;
    unsafe { game_write(DEVICES_RVA, pointer) };
    for (n, ebx) in [0u32, 1, 3, 0xFFFF_FFFF].into_iter().enumerate() {
        let before = Regs {
            eax: 2 + n as u32,
            ebx,
            ecx: 0x10 + n as u32,
            edx: 0x5555,
            ebp: 0x3333,
            ..Regs::SENTINEL
        };
        check_state(
            &mut t,
            &format!("ebx {ebx:#x}"),
            call_original,
            before,
            &|| unsafe {
                std::ptr::write_bytes(pointer as *mut u8, 0xAA, WRITTEN + 8);
                // No DirectInput device and not acquired, should the input thread act on the
                // request before it is put back.
                put(pointer, 4, 0);
                put(pointer, 0x24, 0);
                game_write(COMMAND_RVA, 0u32);
            },
            &[at(COMMAND_RVA), (pointer, WRITTEN + 8)],
            &rust,
        );
    }
    restore(&inputs, &saved);
    drop(record);
    t.finish()
}
