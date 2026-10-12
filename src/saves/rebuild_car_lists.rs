//! `saves::rebuild_car_lists` (0x47088C): two lists of pointers from two lists of numbers.
//!
//! In:  nothing
//! Out: every register preserved
//! The list at `off_75A042` (0x28 entries) and the one at `off_75A12E` (0x14) are filled
//! with -1; then, for each number of `unk_755258` up to its -1, the entry of
//! `off_7564A0` with that number goes to the first list, and likewise from `unk_7552F8`
//! to the second. One caller, `saves::load_options`, whose file holds the two lists of
//! numbers. What they are lists of is a reading: cars that can be chosen.
//! Refuses a list of numbers without its -1 within its own length, and a number of
//! 0x400 or more (the original runs on, and reads wherever the number leads).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("saves::rebuild_car_lists", rva(0x47088C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TABLE_RVA: usize = rva(0x7564A0); // off_7564A0
/// (the numbers, the pointers, how many entries each has)
const LISTS: [(usize, usize, usize); 2] = [
    (rva(0x755258), rva(0x75A042), 0x28), // unk_755258, off_75A042
    (rva(0x7552F8), rva(0x75A12E), 0x14), // unk_7552F8, off_75A12E
];

// ---- Rust version ----

pub unsafe fn rebuild_car_lists() -> Result<(), BadInput> {
    unsafe {
        let mut found = Vec::new();
        for (numbers, _, len) in LISTS {
            let ids: Vec<u32> = (0..len)
                .map(|n| game_read::<u32>(numbers + n * 4))
                .take_while(|&id| id != u32::MAX)
                .collect();
            if ids.len() == len {
                return Err(BadInput("a list of car numbers without its end"));
            }
            if ids.iter().any(|&id| id >= 0x400) {
                return Err(BadInput("a car number far past the table"));
            }
            found.push(ids);
        }
        for (ids, (_, pointers, len)) in found.into_iter().zip(LISTS) {
            for n in 0..len {
                let entry = match ids.get(n) {
                    Some(&id) => game_read::<u32>(TABLE_RVA + id as usize * 4),
                    None => u32::MAX,
                };
                game_write(pointers + n * 4, entry);
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 2] {
    LISTS.map(|(_, pointers, len)| (game_addr(pointers) as u32, len * 4))
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { rebuild_car_lists() }?;
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
    let mut all = regions().to_vec();
    all.extend(LISTS.map(|(numbers, _, len)| (game_addr(numbers) as u32, len * 4)));
    let saved = capture(&all);
    let cases: [(&[u32], &[u32]); 5] = [
        (&[], &[]),
        (&[0], &[1]),
        (&[3, 1, 4, 1, 5, 9, 2, 6], &[0, 0, 0]),
        (
            &[
                10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
            ],
            &[2, 7],
        ),
        (&[1; 0x27], &[2; 0x13]),
    ];
    for (first, second) in cases {
        let prepare = || unsafe {
            restore(&all, &vec![0xAA; saved.len()]);
            for (ids, (numbers, _, _)) in [first, second].into_iter().zip(LISTS) {
                for (n, &id) in ids.iter().chain([&u32::MAX]).enumerate() {
                    game_write(numbers + n * 4, id);
                }
            }
        };
        check_state(
            &mut t,
            &format!("numbers {first:?} and {second:?}"),
            call_original,
            Regs::SENTINEL,
            &prepare,
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
