//! The controller page's binding items (`menu::learn_binding_1` 0x46AFC6 .. `learn_binding_11`
//! 0x46B110, 0x21 bytes apart): the item clicked starts its control's learning
//! (`screens::learn_axis` takes it each frame): `dword_7551F4` 1 (learning), `dword_7551F8`
//! the clock (`system::tick_count`, the 3 s start), `dword_7551FC` the control (1 .. 11).
//! The eleventh is in no menu.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the actions of the four controller pages' items (0x749C70 ..). Shadow mode runs
//! the original alone (the clock differs between two runs). Self-test: the original and the
//! Rust version from the same start: the flag and the control the same, the clocks within
//! 50 ms of each other.

use crate::framework::{Regs, SelfTest, game_read, game_write, rva};

const LEARNING_RVA: usize = rva(0x7551F4); // dword_7551F4: 1 while a control learns
const STARTED_RVA: usize = rva(0x7551F8); // dword_7551F8: when it began
const CONTROL_RVA: usize = rva(0x7551FC); // dword_7551FC: the control learning

/// The Rust version: control `control` learns from now.
pub fn learn_binding(control: u32) {
    unsafe {
        game_write(LEARNING_RVA, 1u32);
        game_write(STARTED_RVA, crate::system::tick_count::tick_count());
        game_write(CONTROL_RVA, control);
    }
}

fn read() -> [u32; 3] {
    unsafe {
        [
            game_read(LEARNING_RVA),
            game_read(STARTED_RVA),
            game_read(CONTROL_RVA),
        ]
    }
}

fn put(v: [u32; 3]) {
    unsafe {
        game_write(LEARNING_RVA, v[0]);
        game_write(STARTED_RVA, v[1]);
        game_write(CONTROL_RVA, v[2]);
    }
}

fn test(
    hook: &crate::framework::RawHook,
    call_original: crate::screens::harness::CallOriginal,
    control: u32,
) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    let kept = read();
    let start = [0, 0x1234, u32::MAX];
    put(start);
    let mut r = Regs::SENTINEL;
    unsafe { call_original(&mut r) };
    let theirs = (
        read(),
        Regs {
            esp: 0,
            eflags: 0,
            ..r
        },
    );
    put(start);
    learn_binding(control);
    let mine = read();
    put(kept);
    let regs_kept = theirs.1.preserved_except(&Regs::SENTINEL, 0);
    t.check(
        regs_kept
            && theirs.0[0] == mine[0]
            && theirs.0[2] == mine[2]
            && theirs.0[1].abs_diff(mine[1]) <= 50,
        || {
            format!(
                "original {:x?} (registers kept {regs_kept}), rust {mine:x?}",
                theirs.0
            )
        },
    );
    t.finish()
}

/// A hook for one binding item.
macro_rules! learn {
    ($module:ident, $name:literal, $address:expr, $control:expr) => {
        pub mod $module {
            use super::*;
            use crate::framework::{RawHook, hook_asm, shadow};
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
                learn_binding($control);
            }

            fn self_test() -> (usize, usize) {
                test(&HOOK, call_original, $control)
            }
        }
    };
}

learn!(learn_binding_1, "menu::learn_binding_1", 0x46AFC6, 1);
learn!(learn_binding_2, "menu::learn_binding_2", 0x46AFE7, 2);
learn!(learn_binding_3, "menu::learn_binding_3", 0x46B008, 3);
learn!(learn_binding_4, "menu::learn_binding_4", 0x46B029, 4);
learn!(learn_binding_5, "menu::learn_binding_5", 0x46B04A, 5);
learn!(learn_binding_6, "menu::learn_binding_6", 0x46B06B, 6);
learn!(learn_binding_7, "menu::learn_binding_7", 0x46B08C, 7);
learn!(learn_binding_8, "menu::learn_binding_8", 0x46B0AD, 8);
learn!(learn_binding_9, "menu::learn_binding_9", 0x46B0CE, 9);
learn!(learn_binding_10, "menu::learn_binding_10", 0x46B0EF, 10);
learn!(learn_binding_11, "menu::learn_binding_11", 0x46B110, 11);

/// The eleven, for the registry.
pub static HOOKS: [&dyn crate::framework::GameHook; 11] = [
    &learn_binding_1::HOOK,
    &learn_binding_2::HOOK,
    &learn_binding_3::HOOK,
    &learn_binding_4::HOOK,
    &learn_binding_5::HOOK,
    &learn_binding_6::HOOK,
    &learn_binding_7::HOOK,
    &learn_binding_8::HOOK,
    &learn_binding_9::HOOK,
    &learn_binding_10::HOOK,
    &learn_binding_11::HOOK,
];
