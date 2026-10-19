//! `input::axis_found` (0x4083F4): DirectInput's `EnumObjects` callback for a joystick
//! being listed: the object's type GUID (`DIDEVICEOBJECTINSTANCEA` +4) named among the six
//! axes' (`GUID_XAxis`, `GUID_YAxis`, `GUID_ZAxis`, `GUID_RxAxis`, `GUID_RyAxis`,
//! `GUID_RzAxis`, at `unk_4F7E38` ..) sets the bit (1, 2, 4, 8, 0x10, 0x20) in the
//! controller's entry's +0x14 (kept: the bits are added to it). Always goes on (eax 1).
//!
//! In:  (stdcall) the `DIDEVICEOBJECTINSTANCEA`, the context (the controller's entry)
//! Out: eax = 1; ebx, esi, edi, ebp kept; `ret 8`
//!
//! Caller: DirectInput, from `input::joystick_found`. Shadow mode runs the original alone.
//! Self-test: the original and the Rust version called with each axis's GUID, a button's
//! and a slider's, on entries of the test's.

use crate::field;
use crate::framework::{RawHook, SelfTest, game_addr, rva};
use crate::input::devices::{at, put};
use crate::input::di_harness::{call_stdcall2, compare, dispatch_callback};
use crate::input::types::{Controller, DiDeviceObjectInstance};
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn thunk(object: u32, entry: u32) -> u32 {
    dispatch_callback(
        &HOOK,
        ORIGINAL.load(Ordering::SeqCst),
        object,
        entry,
        axis_found,
    )
}

type Thunk = unsafe extern "system" fn(u32, u32) -> u32;

pub static HOOK: RawHook = RawHook::new(
    "input::axis_found",
    rva(0x4083F4),
    unsafe { std::mem::transmute::<Thunk, unsafe extern "C" fn()>(thunk) },
    &ORIGINAL,
)
.self_test(self_test);

const AXES_RVA: usize = rva(0x4F7E38); // unk_4F7E38: the six axes' GUIDs, 16 bytes each

/// The Rust version: eax.
pub fn axis_found(object: u32, entry: u32) -> u32 {
    let mut bits = at(entry, field!(Controller, axis_bits));
    for n in 0..6u32 {
        let axis = game_addr(AXES_RVA) as u32 + 16 * n;
        if crate::system::same_guid::same_guid(
            object.wrapping_add(field!(DiDeviceObjectInstance, guid_type)),
            axis,
        ) {
            bits |= 1 << n;
        }
    }
    put(entry, field!(Controller, axis_bits), bits);
    1
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let world = crate::input::di_objects::TestWorld::begin(Vec::new(), Default::default());
    let original = ORIGINAL.load(Ordering::SeqCst);
    let mut guids: Vec<[u8; 16]> = (0..6)
        .map(|n| unsafe { crate::framework::game_read::<[u8; 16]>(AXES_RVA + 16 * n) })
        .collect();
    let rest = [0xBF, 0xC7, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00];
    // GUID_Button, GUID_Slider
    guids.push(crate::input::di_objects::guid(
        0xA36D_02F0,
        0xC9F3,
        0x11CF,
        rest,
    ));
    guids.push(crate::input::di_objects::guid(
        0xA36D_02E4,
        0xC9F3,
        0x11CF,
        rest,
    ));
    let mut entry = vec![0u8; size_of::<Controller>()];
    let e = entry.as_mut_ptr() as u32;
    for (n, g) in guids.iter().enumerate() {
        for before in [0u32, 0x15, 0xFFFF_FFC0] {
            let object = DiDeviceObjectInstance {
                size: size_of::<DiDeviceObjectInstance>() as u32,
                guid_type: *g,
                offset: 0,
                kind: 0,
                flags: 0,
                name: [0; 260],
                rest: [0; 24],
            };
            let o = &object as *const DiDeviceObjectInstance as u32;
            compare(
                &mut t,
                &format!("object {n}, bits before {before:#x}"),
                &world,
                &[(e, size_of::<Controller>())],
                &|| {
                    unsafe { std::ptr::write_bytes(e as *mut u8, 0xAA, size_of::<Controller>()) };
                    put(e, field!(Controller, axis_bits), before);
                },
                &mut || format!("{}", unsafe { call_stdcall2(original, o, e) }),
                &mut || format!("{}", axis_found(o, e)),
            );
        }
    }
    drop(world);
    drop(entry);
    t.finish()
}
