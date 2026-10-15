//! `screens::car_caption_2` (0x469873): the car shown's name as the caption; the twin of
//! `screens::car_caption` (0x469A0E: the same bytes but for a `jmp $+2`).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `screens/car_caption.rs`. In three draw lists (0x758C36, 0x758C46, 0x758C56: the car
//! choice). Refuses what that refuses. Self-test: records of the test's, twelve cars.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::screens::car_caption::{TEST_CARS, TestRecords, car_caption};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::car_caption_2", rva(0x469873), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CAR_RVA: usize = rva(0x755990); // dword_755990
const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText
const CAPTION_LEN: usize = 0x100;

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_addr(CAPTION_RVA) as u32, CAPTION_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { car_caption() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let inputs = [at(CAR_RVA)];
    let saved = (capture(&regions()), capture(&inputs));
    let records = TestRecords::new(&TEST_CARS, &[0x249, 0x24A, 0x24B, 0x24C, 0x537]);
    for car in 0..records.len() {
        check_state(
            &mut t,
            &format!("entry={car}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(CAR_RVA, car);
                restore(&regions(), &[0xAAu8; CAPTION_LEN]);
            },
            &regions(),
            &rust,
        );
    }
    drop(records);
    restore(&regions(), &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
