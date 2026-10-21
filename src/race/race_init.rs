//! `race::load_stage`'s "race init" (`sub_4462B9`, IDA 2026-10-17): after the best line,
//! the camera's start (`sub_446463`), the face lights, the cars (`sub_43603C`), each car's
//! race state, the race slots, the cars' starts and randoms, then fifty settling steps
//! (`sub_4608BB`, `sub_456484`, the marks) and `sub_43FDFE`. Here its small steps.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::race::best_line::{CHECK_COUNT_RVA, CHECK_FACES_RVA, FACES_RVA};
use crate::race::objects::CAR_BLOCKS_RVA;
use crate::race::sounds::REPLAY_RVA;
use crate::service::harness::{Region, at};

pub const SLOTS_RVA: usize = rva(0x71BAC8); // g_raceSlots: 8 records of 0x78, +0 in use
pub const SLOT_SIZE: u32 = 0x78;
pub const SLOTS: u32 = 8;
/// The car blocks `g_carBlocks` lists (all eight, whatever the race's count).
pub const BLOCKS: u32 = 8;
pub const LAST_CHECK_RVA: usize = rva(0x623644); // g_lastCheckpoint
/// dword_6D1838, a64, dword_6D1844, a32: four dwords of spaces.
pub const BLANK_TEXT_RVAS: [usize; 4] =
    [rva(0x6D1838), rva(0x6D183C), rva(0x6D1844), rva(0x6D1848)];
/// dword_6D1850, a64_0, dword_6D185C, a32_0: four dwords of -1.
pub const UNSET_TEXT_RVAS: [usize; 4] =
    [rva(0x6D1850), rva(0x6D1854), rva(0x6D185C), rva(0x6D1860)];

/// Car block n (`g_carBlocks`[n]).
pub fn block(n: u32) -> u32 {
    unsafe { game_read(CAR_BLOCKS_RVA + 4 * n as usize) }
}

unsafe fn put(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) };
}

/// The race slots' first dwords.
pub fn slot_regions() -> Vec<Region> {
    (0..SLOTS)
        .map(|n| (game_addr(SLOTS_RVA) as u32 + n * SLOT_SIZE, 4))
        .collect()
}

/// Bytes `from .. from + len` of each car block.
pub fn block_regions(from: u32, len: usize) -> Vec<Region> {
    (0..BLOCKS).map(|n| (block(n) + from, len)).collect()
}

/// `race::clear_race_slots` (0x4621CD): the eight `g_raceSlots` records' first dword 0.
/// Leaves edi past the last record, ecx 0.
pub unsafe fn clear_race_slots(entry: &Regs) -> Regs {
    let first = game_addr(SLOTS_RVA) as u32;
    for n in 0..SLOTS {
        unsafe { put(first + n * SLOT_SIZE, 0) };
    }
    Regs {
        edi: first + SLOTS * SLOT_SIZE,
        ecx: 0,
        ..*entry
    }
}

/// `race::reset_car_starts` (0x444BBC): each car block's +0x14 and +8 0 (0x320 while
/// `dword_71C3E0` is set), +0x18 and +0x1C 0. Leaves eax that value, ecx 8, esi the last
/// block.
pub unsafe fn reset_car_starts(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::race::cars::start_value`.
    let value = rc2k_core::race::cars::start_value(unsafe { game_read::<u32>(REPLAY_RVA) } != 0);
    for n in 0..BLOCKS {
        let b = block(n);
        unsafe {
            put(b + 0x14, value);
            put(b + 8, value);
            put(b + 0x18, 0);
            put(b + 0x1C, 0);
        }
    }
    Regs {
        eax: value,
        ecx: BLOCKS,
        esi: block(BLOCKS - 1),
        ..*entry
    }
}

/// `race::nonzero_random` (0x444C14): `math::next_random`, 1 for 0.
pub unsafe fn nonzero_random() -> u32 {
    // Converted: `rc2k_core::race::cars::nonzero_random`, on the generator as
    // `math::next_random` reaches it (its wait and count of draws kept).
    use crate::math::next_random::{DRAWS, wait_for_random, with_random};
    wait_for_random();
    DRAWS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    with_random(rc2k_core::race::cars::nonzero_random)
}

/// `race::seed_car_randoms` (0x444BF6): each car block's +0x10 and +0xC one
/// `nonzero_random`. Leaves eax the last, ecx 8, esi the last block.
pub unsafe fn seed_car_randoms(entry: &Regs) -> Regs {
    // Converted: `rc2k_core::race::cars::car_seeds`.
    use crate::math::next_random::{DRAWS, wait_for_random, with_random};
    wait_for_random();
    DRAWS.fetch_add(BLOCKS, std::sync::atomic::Ordering::SeqCst);
    let seeds = with_random(rc2k_core::race::cars::car_seeds);
    for (n, &seed) in seeds.iter().enumerate() {
        let b = block(n as u32);
        unsafe {
            put(b + 0x10, seed);
            put(b + 0xC, seed);
        }
    }
    Regs {
        eax: seeds[BLOCKS as usize - 1],
        ecx: BLOCKS,
        esi: block(BLOCKS - 1),
        ..*entry
    }
}

/// `race::reset_car_race_state` (0x444C1D): each car block's +0x71C 0xCA, +0x72C 0x12C,
/// +0x720/+0x724/+0x728/+0x730 0, +0xA0C .. +0xA1C -1; the four text dwords at 0x6D1838 ..
/// spaces, the four at 0x6D1850 .. -1. Leaves ebx 8, edi the last block.
pub unsafe fn reset_car_race_state(entry: &Regs) -> Regs {
    for n in 0..BLOCKS {
        let b = block(n);
        unsafe {
            put(b + 0x730, 0);
            put(b + 0x72C, 0x12C);
            put(b + 0x724, 0);
            put(b + 0x71C, 0xCA);
            put(b + 0x720, 0);
            put(b + 0x728, 0);
            for k in 0..5 {
                put(b + 0xA0C + 4 * k, u32::MAX);
            }
        }
    }
    unsafe {
        for at in BLANK_TEXT_RVAS {
            game_write(at, 0x2020_2020u32);
        }
        for at in UNSET_TEXT_RVAS {
            game_write(at, u32::MAX);
        }
    }
    Regs {
        ebx: BLOCKS,
        edi: block(BLOCKS - 1),
        ..*entry
    }
}

/// What `reset_car_race_state` writes.
pub fn race_state_regions() -> Vec<Region> {
    let mut r = block_regions(0x71C, 0x18);
    r.extend(block_regions(0xA0C, 0x14));
    r.extend(
        BLANK_TEXT_RVAS
            .iter()
            .chain(UNSET_TEXT_RVAS.iter())
            .map(|&a| at(a)),
    );
    r
}

/// `race::first_check_checkpoint` (0x446280): the first checkpoint from 1 whose face's
/// +0x26 is not 0, looked for up to `g_lastCheckpoint` (not included); 0 for none or with
/// `g_checkpointCount` - 2 negative. Leaves eax that; while looking edx the checkpoint,
/// ebx and esi the face's number and address, al its byte. Refuses a look that would run
/// past the list (the original runs on until it finds one: `g_lastCheckpoint` 0 or 1).
pub unsafe fn first_check_checkpoint(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::checkpoints::first_check_checkpoint`; ebx, esi and al
    // the last face asked about.
    let mut g = *entry;
    let count: u32 = unsafe { game_read(CHECK_COUNT_RVA) };
    let last = || unsafe { game_read(LAST_CHECK_RVA) };
    let look = rc2k_core::race::checkpoints::first_check_checkpoint(count, last, |n| {
        let face: u32 = unsafe { game_read(CHECK_FACES_RVA + 4 * n as usize) };
        let address = (face << 6).wrapping_add(unsafe { game_read(FACES_RVA) });
        let byte = unsafe { *((address + 0x26) as *const u8) };
        g.ebx = face;
        g.esi = address;
        g.eax = g.eax & !0xFF | byte as u32;
        byte
    })?;
    match look {
        None => g.eax = 0,
        Some(l) => {
            g.edx = l.stopped;
            g.eax = l.found.unwrap_or(0);
        }
    }
    Ok(g)
}
