//! For the self-tests of the installation checks: stand-ins for what they call, each of
//! which hands the registers to a function of the test's, so that a test can say what
//! files exist, which drives are CD-ROMs and so on, to the original's copy and to the
//! Rust version alike. And a landing for a copy's jump to its fatal stub.

use crate::framework::Regs;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

/// What a stand-in does with the registers it was called with.
pub type Handler = fn(place: u32, regs: &mut Regs);

static HANDLER: AtomicUsize = AtomicUsize::new(0);
/// 1 after a copy went to its fatal stub.
pub static WENT_FATAL: AtomicU32 = AtomicU32::new(0);

pub fn set_handler(handler: Handler) {
    HANDLER.store(handler as usize, Ordering::SeqCst);
    WENT_FATAL.store(0, Ordering::SeqCst);
}

/// Calls the test's function for a place: for the Rust versions' side.
pub fn act(place: u32, regs: &mut Regs) {
    let handler = HANDLER.load(Ordering::SeqCst);
    if handler != 0 {
        let handler: Handler = unsafe { std::mem::transmute(handler) };
        handler(place, regs);
    }
}

extern "C" fn act_from_copy(regs: *mut Regs, place: u32) {
    act(place, unsafe { &mut *regs });
}

macro_rules! acting_stand_in {
    ($name:ident, $place:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "pushfd",
                "pushad",
                "mov eax, esp",
                "push {place}",
                "push eax",
                "call {act}",
                "add esp, 8",
                "popad",
                "popfd",
                "ret",
                place = const $place,
                act = sym act_from_copy,
            )
        }
    };
}
acting_stand_in!(stand_in_0, 0);
acting_stand_in!(stand_in_1, 1);
acting_stand_in!(stand_in_2, 2);
acting_stand_in!(stand_in_3, 3);
acting_stand_in!(stand_in_4, 4);
acting_stand_in!(stand_in_5, 5);
acting_stand_in!(stand_in_6, 6);
acting_stand_in!(stand_in_7, 7);
acting_stand_in!(stand_in_8, 8);
acting_stand_in!(stand_in_9, 9);
acting_stand_in!(stand_in_10, 10);
acting_stand_in!(stand_in_11, 11);
acting_stand_in!(stand_in_12, 12);
acting_stand_in!(stand_in_13, 13);
acting_stand_in!(stand_in_14, 14);
acting_stand_in!(stand_in_15, 15);
acting_stand_in!(stand_in_16, 16);
acting_stand_in!(stand_in_17, 17);
acting_stand_in!(stand_in_18, 18);
acting_stand_in!(stand_in_19, 19);
acting_stand_in!(stand_in_20, 20);
acting_stand_in!(stand_in_21, 21);
acting_stand_in!(stand_in_22, 22);
acting_stand_in!(stand_in_23, 23);
acting_stand_in!(stand_in_24, 24);
acting_stand_in!(stand_in_25, 25);
acting_stand_in!(stand_in_26, 26);
acting_stand_in!(stand_in_27, 27);
acting_stand_in!(stand_in_28, 28);
acting_stand_in!(stand_in_29, 29);
acting_stand_in!(stand_in_30, 30);
acting_stand_in!(stand_in_31, 31);
acting_stand_in!(stand_in_32, 32);
acting_stand_in!(stand_in_33, 33);
acting_stand_in!(stand_in_34, 34);
acting_stand_in!(stand_in_35, 35);
acting_stand_in!(stand_in_36, 36);
acting_stand_in!(stand_in_37, 37);
acting_stand_in!(stand_in_38, 38);
acting_stand_in!(stand_in_39, 39);

const STAND_INS: [unsafe extern "C" fn(); 40] = [
    stand_in_0,
    stand_in_1,
    stand_in_2,
    stand_in_3,
    stand_in_4,
    stand_in_5,
    stand_in_6,
    stand_in_7,
    stand_in_8,
    stand_in_9,
    stand_in_10,
    stand_in_11,
    stand_in_12,
    stand_in_13,
    stand_in_14,
    stand_in_15,
    stand_in_16,
    stand_in_17,
    stand_in_18,
    stand_in_19,
    stand_in_20,
    stand_in_21,
    stand_in_22,
    stand_in_23,
    stand_in_24,
    stand_in_25,
    stand_in_26,
    stand_in_27,
    stand_in_28,
    stand_in_29,
    stand_in_30,
    stand_in_31,
    stand_in_32,
    stand_in_33,
    stand_in_34,
    stand_in_35,
    stand_in_36,
    stand_in_37,
    stand_in_38,
    stand_in_39,
];

pub fn stand_in(place: u32) -> usize {
    STAND_INS[place as usize] as *const () as usize
}

/// Where a copy's jump to its fatal stub is pointed: the copy began with `pusha` and has
/// nothing else on the stack there, so this ends it as its own `popa; retn` would.
#[unsafe(naked)]
pub unsafe extern "C" fn fatal_landing() {
    core::arch::naked_asm!(
        "mov dword ptr [{flag}], 1",
        "popad",
        "ret",
        flag = sym WENT_FATAL,
    )
}
