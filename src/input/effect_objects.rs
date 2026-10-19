//! A force-feedback effect's DirectInput object (`IDirectInputEffect`, an effect's +0x30;
//! the effects are a device record's six from +0x3C, 0xA4 bytes each), asked by the game's
//! four small functions, each doing nothing when the effect has no object:
//! - `input::release_effect` (0x409ACC): `Stop`, `Release`, +0x30 0;
//! - `input::start_effect` (0x409AEE): `Download`, `Start` (one iteration, no flags);
//! - `input::set_effect_parameters` (0x409B1F): `SetParameters` with the effect's
//!   `DIEFFECT` (+0x34) and eax the flags;
//! - `input::stop_effect` (0x409B0D): `Stop`.
//!
//! In:  edi = the effect, eax = the flags (`set_effect_parameters`)
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the effects' functions (`input::effect_N_change`, through
//! `effect_changes::game_part`), `input::make_effect` and `input::end_force`
//! (`release_effect`). The methods are the object's own (DirectInput's in the game, Rust's
//! in the standalone program, `input::di_objects`). Shadow mode runs the original alone (it
//! calls DirectInput). Self-test: the original and the Rust version on DirectInput objects
//! made in Rust (`input::di_harness`): an effect with no object and one with a constant
//! force's, `set_effect_parameters` with several flags.

use crate::field;
use crate::framework::{Regs, SelfTest};
use crate::input::devices::{at, put};
use crate::input::di_harness::{EFFECT_LEN, compare, method, regs_text, slot};
use crate::input::types::ForceEffect;

/// `release_effect`'s Rust version.
pub fn release_effect(effect: u32) {
    let object = at(effect, field!(ForceEffect, object));
    if object != 0 {
        method(object, slot::STOP, &[]);
        method(object, slot::RELEASE, &[]);
        put(effect, field!(ForceEffect, object), 0);
    }
}

/// `start_effect`'s Rust version.
pub fn start_effect(effect: u32) {
    let object = at(effect, field!(ForceEffect, object));
    if object != 0 {
        method(object, slot::DOWNLOAD, &[]);
        method(object, slot::START, &[1, 0]);
    }
}

/// `set_effect_parameters`'s Rust version.
pub fn set_effect_parameters(effect: u32, flags: u32) {
    let object = at(effect, field!(ForceEffect, object));
    if object != 0 {
        method(
            object,
            slot::SET_PARAMETERS,
            &[effect.wrapping_add(field!(ForceEffect, effect)), flags],
        );
    }
}

/// `stop_effect`'s Rust version.
pub fn stop_effect(effect: u32) {
    let object = at(effect, field!(ForceEffect, object));
    if object != 0 {
        method(object, slot::STOP, &[]);
    }
}

/// The four (0 release, 1 start, 2 set, 3 stop): the Rust version on the registers.
fn rust_of(which: u32, r: &Regs) {
    match which {
        0 => release_effect(r.edi),
        1 => start_effect(r.edi),
        2 => set_effect_parameters(r.edi, r.eax),
        _ => stop_effect(r.edi),
    }
}

/// An effect of the test's at `entry` (0xA4 bytes): a constant force's parameters
/// (`input::make_effect`'s), and an object made for it on the wheel (`object`).
pub fn test_effect(entry: u32, object: bool) {
    unsafe { std::ptr::write_bytes(entry as *mut u8, 0x5A, EFFECT_LEN as usize) };
    put(entry, field!(ForceEffect, kind), 1);
    put(entry, field!(ForceEffect, found), 1);
    put(entry, field!(ForceEffect, object), 0);
    let constant = crate::input::di_objects::guid(
        0x1354_1C20,
        0x8E33,
        0x11D0,
        [0x9A, 0xD0, 0x00, 0xA0, 0xC9, 0xA0, 0x6E, 0x35],
    );
    unsafe {
        std::ptr::copy_nonoverlapping(
            constant.as_ptr(),
            (entry + field!(ForceEffect, guid)) as *mut u8,
            16,
        )
    };
    crate::input::make_effect::fill_parameters(entry);
    if object {
        let device = crate::input::di_objects::test_device(1);
        method(
            device,
            slot::CREATE_EFFECT,
            &[
                entry + field!(ForceEffect, guid),
                entry + field!(ForceEffect, effect),
                entry + field!(ForceEffect, object),
                0,
            ],
        );
    }
}

fn test(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    which: u32,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let world = crate::input::di_objects::TestWorld::begin(
        crate::input::di_objects::test_devices(),
        Default::default(),
    );
    let mut entry = vec![0u8; EFFECT_LEN as usize];
    let e = entry.as_mut_ptr() as u32;
    let flags: &[u32] = if which == 2 {
        &[0x1, 0x4, 0x100, 0x2000_0100, 0x1FF]
    } else {
        &[0]
    };
    for object in [false, true] {
        for &eax in flags {
            let entry_regs = Regs {
                eax,
                edi: e,
                ..Regs::SENTINEL
            };
            compare(
                &mut t,
                &format!("object {object}, flags {eax:#x}"),
                &world,
                &[(e, EFFECT_LEN as usize)],
                &|| test_effect(e, object),
                &mut || {
                    let mut r = entry_regs;
                    unsafe { call_original(&mut r) };
                    regs_text(&Regs { esp: 0, ..r })
                },
                &mut || {
                    rust_of(which, &entry_regs);
                    regs_text(&Regs {
                        esp: 0,
                        ..entry_regs
                    })
                },
            );
        }
    }
    drop(world);
    drop(entry);
    t.finish()
}

/// A hook for one of the four.
macro_rules! effect_object {
    ($module:ident, $name:literal, $address:expr, $which:expr) => {
        pub mod $module {
            use super::*;
            use crate::framework::{RawHook, hook_asm, rva, shadow};
            use std::sync::atomic::AtomicUsize;

            static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

            pub static HOOK: RawHook =
                RawHook::new($name, rva($address), thunk, &ORIGINAL).self_test(self_test);

            hook_asm!(
                thunk,
                call_original,
                dispatch = dispatch,
                original = ORIGINAL
            );

            extern "C" fn dispatch(r: &mut Regs) {
                HOOK.count_call();
                if shadow() {
                    unsafe { call_original(r) };
                    return;
                }
                rust_of($which, r);
            }

            fn self_test() -> (usize, usize) {
                test(&HOOK, call_original, $which)
            }
        }
    };
}

effect_object!(release_effect_hook, "input::release_effect", 0x409ACC, 0);
effect_object!(start_effect_hook, "input::start_effect", 0x409AEE, 1);
effect_object!(
    set_effect_parameters_hook,
    "input::set_effect_parameters",
    0x409B1F,
    2
);
effect_object!(stop_effect_hook, "input::stop_effect", 0x409B0D, 3);

/// The four, for `input/mod.rs`.
pub static HOOKS: [&dyn crate::framework::GameHook; 4] = [
    &release_effect_hook::HOOK,
    &start_effect_hook::HOOK,
    &set_effect_parameters_hook::HOOK,
    &stop_effect_hook::HOOK,
];
