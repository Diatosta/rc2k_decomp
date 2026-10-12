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

const STAND_INS: [unsafe extern "C" fn(); 6] = [
    stand_in_0, stand_in_1, stand_in_2, stand_in_3, stand_in_4, stand_in_5,
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
