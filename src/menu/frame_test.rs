//! For the self-tests of `menu::frame_callback` and `menu::draw_frame`: stand-ins for what
//! the two call, which note each call and the registers it came with, and an `Env` that
//! does the same for the Rust versions. One call, counted from the first, can be made to
//! come back with the carry flag set.

use crate::framework::{Regs, game_addr};
use crate::menu::screen_loop::Env;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

/// (place, eax, ebx, ecx, edx, esi, edi, ebp) of every call.
static NOTES: Mutex<Vec<[u32; 8]>> = Mutex::new(Vec::new());
/// The number of the call, from 0, that fails; u32::MAX for none.
static FAILING: AtomicU32 = AtomicU32::new(u32::MAX);
/// What the quit switch says.
pub static QUITTING: AtomicU32 = AtomicU32::new(0);

/// The list function that changes registers, to show how they are handed on.
pub const CHANGING: u32 = 10;
pub const STAND_IN_COUNT: usize = 12;

fn note(regs: &mut Regs, place: u32) {
    let mut notes = NOTES.lock().unwrap();
    let fails = notes.len() as u32 == FAILING.load(Ordering::SeqCst);
    notes.push([
        place, regs.eax, regs.ebx, regs.ecx, regs.edx, regs.esi, regs.edi, regs.ebp,
    ]);
    if place == CHANGING {
        regs.ebx = regs.ebx.wrapping_add(0x101);
        regs.edi ^= 0x40;
        regs.eax = 0x7777;
    }
    regs.set_carry(fails);
}

extern "C" fn note_from_copy(regs: *mut Regs, place: u32) {
    note(unsafe { &mut *regs }, place);
}

macro_rules! noting_stand_in {
    ($name:ident, $place:expr) => {
        #[unsafe(naked)]
        unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                "pushfd",
                "pushad",
                "mov eax, esp",
                "push {place}",
                "push eax",
                "call {note}",
                "add esp, 8",
                "popad",
                "popfd",
                "ret",
                place = const $place,
                note = sym note_from_copy,
            )
        }
    };
}
noting_stand_in!(stand_in_0, 0);
noting_stand_in!(stand_in_1, 1);
noting_stand_in!(stand_in_2, 2);
noting_stand_in!(stand_in_3, 3);
noting_stand_in!(stand_in_4, 4);
noting_stand_in!(stand_in_5, 5);
noting_stand_in!(stand_in_6, 6);
noting_stand_in!(stand_in_7, 7);
noting_stand_in!(stand_in_8, 8);
noting_stand_in!(stand_in_9, 9);
noting_stand_in!(stand_in_10, 10);
noting_stand_in!(stand_in_11, 11);

const STAND_INS: [unsafe extern "C" fn(); STAND_IN_COUNT] = [
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
];

/// The address of the stand-in for a place.
pub fn stand_in(place: u32) -> usize {
    STAND_INS[place as usize] as *const () as usize
}

/// Starts a run: no notes, and the call that fails.
pub fn begin(failing: u32, quitting: u32) {
    NOTES.lock().unwrap().clear();
    FAILING.store(failing, Ordering::SeqCst);
    QUITTING.store(quitting, Ordering::SeqCst);
}

pub fn notes() -> Vec<[u32; 8]> {
    NOTES.lock().unwrap().clone()
}

/// The Rust versions' side: `places` says which place each game function has; a function
/// that is none of them is one of the stand-ins itself (a list of the test's holds them).
pub struct Noting<'a> {
    pub places: &'a [(usize, u32)],
}

impl Env for Noting<'_> {
    fn call(&mut self, function: usize, regs: &mut Regs) {
        let place = self
            .places
            .iter()
            .find(|&&(known, _)| known == function)
            .map(|&(_, place)| place)
            .or_else(|| {
                (0..STAND_IN_COUNT as u32)
                    .find(|&place| stand_in(place).wrapping_sub(game_addr(0)) == function)
            })
            .expect("a call the test has no place for");
        note(regs, place);
    }
    fn quitting(&self) -> bool {
        QUITTING.load(Ordering::SeqCst) == 1
    }
}
