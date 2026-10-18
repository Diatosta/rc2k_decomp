//! `marks::init_mark_step` (0x456EA7): the stage loading's "init mark" step: the race's cars
//! (`g_raceCars`) into `dword_5C2FE0`, `dword_5C5640` 0x50, `dword_5C563C` 1, the work car's
//! file number (`race::car_file_number` of `g_carBlockStore`'s kind) into `dword_5C562C`;
//! for each of the `dword_63DB94` cars its record's (`g_carRecords`) places +0x23C (0x50 +
//! 0x14 a car) and +0x240 (0xA more), seven values from its kind's row of `dword_5C90D8`
//! .. (the kind its block's +0x24 held to 0 .. 0x16, rows of 0x134) and the seventeen
//! fixed numbers +0x36C .. +0x3AC; then `marks::init_marks` with the settings
//! `asc_5C2F84`.
//!
//! In:  nothing
//! Out: ecx the cars, esi the settings; the rest as `init_marks` leaves them
//!
//! Caller: `race::loadlevel_init` (0x444670, with the log off round it). The file number
//! is its Rust version; the marks through `part` (1), in the game `init_marks`' Rust
//! version. Refuses a car count of 0 or past 8 (the original's loop would run on) and a
//! missing block or record, before anything. Shadow mode runs the original alone (the
//! marks' set-up takes memory). Self-test: a copy whose `init_marks` call goes to a
//! stand-in, against the Rust version, on test blocks and records.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::init_mark_step", rva(0x456EA7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const RACE_CARS_RVA: usize = rva(0x63DB90); // g_raceCars
const CARS_COPY_RVA: usize = rva(0x5C2FE0); // dword_5C2FE0
const SET_RVA: [usize; 2] = [rva(0x5C5640), rva(0x5C563C)];
const KIND_RVA: usize = rva(0x70F410); // g_carBlockStore.kind
const FILE_RVA: usize = rva(0x5C562C); // dword_5C562C
const COUNT_RVA: usize = rva(0x63DB94); // dword_63DB94
const BLOCKS_RVA: usize = rva(0x71B3B8); // g_carBlocks
const RECORDS_RVA: usize = rva(0x71B3D8); // g_carRecords
const ROWS_RVA: usize = rva(0x5C90D8); // dword_5C90D8 ..: rows of 0x134
const SETTINGS_RVA: usize = rva(0x5C2F84); // asc_5C2F84
const MOST_CARS: u32 = 8;
/// (the record's field, the row's offset from `dword_5C90D8`)
const COPIES: [(u32, usize); 7] = [
    (0x30C, 0),
    (0x310, 0x20),
    (0x314, 0x24),
    (0x318, 0x18),
    (0x31C, 0x1C),
    (0x320, 0x14),
    (0x324, 0x28),
];
const FIXED: [u32; 17] = [
    0xA, 0xB, 0x18, 0x19, 0xC, 0xD, 0x16, 0x17, 0x1A, 0x1B, 0xE, 0xF, 0x10, 0x11, 0x13, 0x14, 0x12,
];

/// A call by place: 1 `init_marks`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn init_mark_step(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if count == 0 || count > MOST_CARS {
            return Err(BadInput("an init mark of 0 or more than 8 cars").into());
        }
        let block = |n: u32| game_read::<u32>(BLOCKS_RVA + 4 * n as usize);
        let record = |n: u32| game_read::<u32>(RECORDS_RVA + 4 * n as usize);
        if (0..count).any(|n| block(n) == 0 || record(n) == 0) {
            return Err(BadInput("an init mark car without its block or record").into());
        }
        game_write(CARS_COPY_RVA, game_read::<u32>(RACE_CARS_RVA));
        game_write(SET_RVA[0], 0x50u32);
        game_write(SET_RVA[1], 1u32);
        g.eax = crate::race::mud::car_file_number(game_read(KIND_RVA));
        game_write(FILE_RVA, g.eax);
        for n in 0..count {
            let kind = ((block(n) + 0x24) as *const i32)
                .read_unaligned()
                .clamp(0, 0x16) as usize;
            let row = ROWS_RVA + kind * 0x134;
            let put = |o: u32, v: u32| ((record(n) + o) as *mut u32).write_unaligned(v);
            let place = n * 0x14 + 0x50;
            put(0x23C, place);
            put(0x240, place + 0xA);
            for (field, off) in COPIES {
                put(field, game_read(row + off));
            }
            for (k, v) in FIXED.into_iter().enumerate() {
                put(0x36C + 4 * k as u32, v);
            }
        }
        g.ecx = count;
        g.esi = game_addr(SETTINGS_RVA) as u32;
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `init_marks`' Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::init_marks as im;
    *r = unsafe { im::init_marks(r, &mut im::game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { init_mark_step(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::mud::{copy, run_both, test_part};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // mov eax, g_raceCars
    let entry: &[u8] = &[0xA1, 0x90, 0xDB, 0x63, 0x00];
    let copy = match copy(0x456EA7, 0x18F, entry, &[0x23, 0x189], &[(0x4D12F7, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let mut blocks = vec![0u32; 8 * 0x10];
    let mut records = vec![0u32; 8 * 0x100];
    let (bp, rp) = (blocks.as_mut_ptr() as u32, records.as_mut_ptr() as u32);
    let regions: Vec<Region> = vec![
        at(CARS_COPY_RVA),
        at(SET_RVA[0]),
        at(SET_RVA[1]),
        at(FILE_RVA),
        (rp, 8 * 0x400),
    ];
    let mut vars = regions.clone();
    vars.extend([COUNT_RVA, KIND_RVA].map(at));
    vars.push((game_addr(BLOCKS_RVA) as u32, 0x20));
    vars.push((game_addr(RECORDS_RVA) as u32, 0x20));
    let saved = capture(&vars);
    for case in 0..4u32 {
        run_both(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &|| unsafe {
                restore(&vars, &saved);
                let count = [1u32, 3, 6, 8][case as usize];
                game_write(COUNT_RVA, count);
                game_write(KIND_RVA, case * 5);
                for n in 0..8u32 {
                    game_write(BLOCKS_RVA + 4 * n as usize, bp + 0x40 * n);
                    game_write(RECORDS_RVA + 4 * n as usize, rp + 0x400 * n);
                    ((bp + 0x40 * n + 0x24) as *mut i32)
                        .write_unaligned([-3, 0, 7, 0x16, 0x30, 5, 1, 2][n as usize] + case as i32);
                }
                std::ptr::write_bytes(rp as *mut u8, 0xAA, 8 * 0x400);
            },
            &|r| unsafe { init_mark_step(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    drop((blocks, records));
    t.finish()
}
