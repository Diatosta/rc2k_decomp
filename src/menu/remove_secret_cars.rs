//! `menu::remove_secret_cars` (0x471BC7): takes the secret cars out of the two lists of
//! cars that can be chosen, and clears the cheats.
//!
//! In:  nothing
//! Out: every register preserved
//! For the list at `off_75A042` and then the one at `off_759D16` (`sub_471BEC`): the
//! entries that are not in the list of secret cars `off_759F4A` (`sub_471BAC`) are copied
//! to the scratch list `unk_759F7A` and from there back to the list's start, a -1 put
//! after them, and the cheats' flags `g_cheats` (0x75870C) set to 0. The lists hold the
//! addresses of car descriptions; each ends with -1.
//! One caller, `system::load_game_data`, when the setting `allsecretcars` is 0.
//! Refuses a list without its end within its room, and a list that would be left with no
//! car at all: the original copies one entry back and then counts through all 32 bits.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use rc2k_core::menu::car_lists;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::remove_secret_cars", rva(0x471BC7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// A list of dwords ended by -1: where it is and how many dwords it has room for.
pub type List = (usize, usize);
/// off_75A042: the cars of one kind of event (0x28 entries are kept in `var\ral.dat`).
pub const FIRST: List = (rva(0x75A042), 0x28);
/// off_759D16: the next variable is 0xB4 bytes on.
pub const SECOND: List = (rva(0x759D16), 0x2D);
/// off_759F4A: the secret cars.
const SECRET: List = (rva(0x759F4A), 0x0C);
/// unk_759F7A: where a list is copied to and back from.
const SCRATCH: List = (rva(0x759F7A), 0x32);
pub const CHEATS_RVA: usize = rva(0x75870C); // g_cheats

pub fn region(list: List) -> Region {
    (game_addr(list.0) as u32, list.1 * 4)
}

/// The list's entries before its -1.
pub unsafe fn read_list(list: List) -> Result<Vec<u32>, BadInput> {
    Ok(car_lists::entries(list.1, |n| unsafe {
        game_read::<u32>(list.0 + n * 4)
    })?)
}

/// The entries and a -1 after them. The caller has made sure of the room.
pub unsafe fn write_list(list: List, entries: &[u32]) {
    unsafe {
        for (n, &entry) in entries.iter().enumerate() {
            game_write(list.0 + n * 4, entry);
        }
        game_write(list.0 + entries.len() * 4, car_lists::END);
    }
}

// ---- Rust version ----

pub unsafe fn remove_secret_cars() -> Result<(), BadInput> {
    // Converted: `rc2k_core::menu::car_lists` (`entries`, `without_secret`).
    unsafe {
        let secret = read_list(SECRET)?;
        // Each list read and checked in turn, as the original goes.
        let mut kept = Vec::new();
        for list in [FIRST, SECOND] {
            kept.extend(car_lists::without_secret(
                &[read_list(list)?],
                &secret,
                SCRATCH.1,
            )?);
        }
        for (list, cars) in [FIRST, SECOND].into_iter().zip(&kept) {
            // The scratch list gets no end mark.
            for (n, &car) in cars.iter().enumerate() {
                game_write(SCRATCH.0 + n * 4, car);
            }
            write_list(list, cars);
            game_write(CHEATS_RVA, 0u32);
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 4] {
    [
        region(FIRST),
        region(SECOND),
        region(SCRATCH),
        at(CHEATS_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { remove_secret_cars() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: lists of the test's in the game's places, which are put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let Ok(secret) = (unsafe { read_list(SECRET) }) else {
        return t.skip("the list of secret cars has no end");
    };
    if secret.len() < 3 {
        return t.skip("the list of secret cars is not what the test was made for");
    }
    let regions = regions();
    let saved = capture(&regions);
    let (a, b, c) = (0x1111_0000u32, 0x2222_0000u32, 0x3333_0000u32);
    let (s0, s1, last) = (secret[0], secret[1], secret[secret.len() - 1]);
    let all: Vec<u32> = [&[a][..], &secret[..], &[b][..]].concat();
    let cases: [(&str, Vec<u32>, Vec<u32>); 6] = [
        ("no secret car", vec![a, b, c], vec![c]),
        (
            "secret cars first and last",
            vec![s0, a, b, last],
            vec![s1, c, s0],
        ),
        (
            "secret cars in the middle",
            vec![a, s1, s0, b, last, c],
            vec![a, last, b],
        ),
        ("every secret car", all, vec![last, s0, a]),
        ("one car twice", vec![a, a, s0, a], vec![b, s1, b]),
        (
            "long lists",
            (0..0x27).map(|n| a + n).collect(),
            (0..0x2C).map(|n| b + n).collect(),
        ),
    ];
    for (what, first, second) in cases {
        check_state(
            &mut t,
            what,
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.len()]);
                write_list(FIRST, &first);
                write_list(SECOND, &second);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
