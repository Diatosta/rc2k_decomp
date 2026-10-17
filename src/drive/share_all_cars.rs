//! `drive::share_all_cars` (0x456484): `drive::keep_lead_car_values`,
//! `drive::set_listener`, then for each of the eight cars (`g_currentCar` = n)
//! `drive::share_car_state` with its block, its record (`off_71B3D8[n]`) and its sound
//! object (`g_raceSoundObjects[n]`) inside `pusha` / `popa`; then `dword_5C5688` = 1
//! when `dword_70F410` is not 0x21 and the first record's +0x258 is 10 or more (else 0);
//! then `drive::put_back_lead_car_values`.
//!
//! In:  nothing
//! Out: ecx 0, esi the last block, edi the first record, ebp the last sound object, and
//!      eax, ebx, edx as `set_listener` left them (eax the record's +0x258 when
//!      `dword_70F410` is not 0x21); no flags read (its callers go on with `mov`)
//!
//! Callers: race init's loop (`sub_4462B9`, 0x4463F4) and `sub_456336` (0x4563C8).
//! Refuses (before anything) what `set_listener` refuses, a car whose file number is
//! above 0x21, and a null record or block.
//! Self-test: a copy of the original (its four calls to the game's, hooked) against the
//! Rust version on the game's own state, only in a race (skipped otherwise): everything
//! it writes is saved first and put back after each run. Its pieces have their own
//! tests on test memory.

use crate::drive::keep_lead_car_values::keep_lead_car_values;
use crate::drive::put_back_lead_car_values::put_back_lead_car_values;
use crate::drive::set_listener::set_listener;
use crate::drive::share_car_state::{BLOCK_LEN, RECORD_LEN, share_car_state};
use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBP, ECX, EDI, ESI, RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write,
    hook_asm, rva,
};
use crate::race::mud::car_file_number;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("drive::share_all_cars", rva(0x456484), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const RECORDS_RVA: usize = rva(0x71B3D8); // g_carRecords (was off_71B3D8)
const SOUNDS_RVA: usize = rva(0x512A10); // g_raceSoundObjects
const CURRENT_RVA: usize = rva(0x71B3F8); // g_currentCar
const SHOWN_RVA: usize = rva(0x5C5688); // dword_5C5688
const LEAD_FILE_RVA: usize = rva(0x70F410); // dword_70F410

fn cars() -> [(u32, u32, u32); 8] {
    std::array::from_fn(|n| unsafe {
        (
            game_read(BLOCKS_RVA + 4 * n),
            game_read(RECORDS_RVA + 4 * n),
            game_read(SOUNDS_RVA + 4 * n),
        )
    })
}

/// The refusals, worked out before anything is written.
unsafe fn refusal() -> Option<&'static str> {
    for (block, record, sound) in cars() {
        if block == 0 || record == 0 || sound == 0 {
            return Some("a null car block, record or sound object");
        }
        if car_file_number(unsafe { ((block + 0x24) as *const u32).read_unaligned() }) > 0x21 {
            return Some("a car file number above 0x21 (past g_lightRecords)");
        }
    }
    None
}

/// The Rust version.
pub unsafe fn share_all_cars() -> Result<Left, Refused> {
    unsafe {
        if let Some(why) = refusal() {
            return Err(BadInput(why).into());
        }
        // `set_listener` refuses before writing; nothing is written before it but the
        // lead car's values, which are kept again by the original.
        let listener = set_listener()?;
        keep_lead_car_values();
        let mut last = (0, 0, 0);
        for (n, (block, record, sound)) in cars().into_iter().enumerate() {
            game_write(CURRENT_RVA, n as u32);
            let r = Regs {
                esi: block,
                edi: record,
                ebp: sound,
                ..Regs::default()
            };
            share_car_state(&r)?;
            last = (block, record, sound);
        }
        let first_record: u32 = game_read(RECORDS_RVA);
        game_write(SHOWN_RVA, 0u32);
        let mut left = listener
            .with(ECX, 0)
            .with(ESI, last.0)
            .with(EDI, first_record)
            .with(EBP, last.2);
        if game_read::<u32>(LEAD_FILE_RVA) != 0x21 {
            let eax = ((first_record + 0x258) as *const u32).read_unaligned();
            if eax >= 10 {
                game_write(SHOWN_RVA, 1u32);
            }
            left = left.with(EAX, eax);
        }
        put_back_lead_car_values();
        Ok(left)
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { share_all_cars() }
}

/// What it writes: its pieces' regions, the blocks, records and sound objects.
fn regions() -> Vec<Region> {
    let mut r = crate::drive::set_listener::regions(&crate::physics::mirror::Mirror::game());
    let a = |x: usize| game_addr(rva(x)) as u32;
    r.push((a(0x64165C), 0x20));
    r.push((a(0x70FA70), 0x70));
    r.push((game_addr(CURRENT_RVA) as u32, 4));
    r.push((game_addr(SHOWN_RVA) as u32, 4));
    for (block, record, sound) in cars() {
        if block != 0 && record != 0 && sound != 0 {
            r.push((block, BLOCK_LEN));
            r.push((record, RECORD_LEN));
            r.push((sound + 0xF0, 4));
        }
    }
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = unsafe { refusal() } {
        return t.skip(&format!("not in a race ({why})"));
    }
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x456484,
            len: 0x71,
            entry: &[0xE8, 0x6C, 0x00, 0x00, 0x00],
            calls: &[0, 5, 0x2B, 0x6B],
            jumps_out: &[],
        },
        &|_| None,
        &[],
        0,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let all = regions();
    let saved = capture(&all);
    let mut ro = Regs::SENTINEL;
    unsafe { call_game(&mut ro, copy.addr()) };
    let theirs = capture(&all);
    restore(&all, &saved);
    let mine = unsafe { share_all_cars() };
    let state = capture(&all);
    restore(&all, &saved);
    let regs_ok = mine.as_ref().is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
    t.check(regs_ok && theirs == state, || {
        let first = theirs.iter().zip(&state).position(|(a, b)| a != b);
        format!("regs_ok={regs_ok} {mine:?}, first difference at byte {first:?}")
    });
    drop(copy);
    t.finish()
}
