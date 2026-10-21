//! `saves::allocate_best_times` (0x47C4D8): takes the memory of the best times' tables,
//! once.
//!
//! In:  nothing
//! Out: every register preserved
//! Nothing if `dword_747610` is 1; else that becomes 1 and nine blocks are taken on the
//! list `stru_7475D0`, each block's size stored before it is asked for and its address
//! after, in the pairs from `dword_747618` on: 0x1C20, 0xBB8, stages * 0x618, 0x15E0,
//! 0xDB60, 0x4C2C4, 0x3A984, 0xDB60, stages * 0x618 bytes. The sixth and the seventh get
//! -1 in their last dword. "Stages" is the number of records of the championship's table
//! (`g_recordTable`) whose tag is 0 (`saves::count_stages`, 0x47C95D, a plain function
//! here). Six of the nine are what `saves::load_best_times` reads `var\hst.dat` into.
//! Callers: `saves::load_best_times` and `sub_47C63F`.
//! The original has a fatal error for each block that cannot be taken (0x807010F1,
//! 0x807010FB, 0x80701000 .. 0x80701005, 0x8070100A); none can come, the allocator
//! reporting its own first.
//! Shadow mode runs the original alone (the memory is taken once) and checks what it
//! left. Self-test: a redirected copy whose variables, list and record table are the
//! test's.
//! Refuses a record table without its end, and what the allocator refuses.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::memory::{MOST_HELD_RVA, MemoryList};
use crate::redirect::RedirectedCopy;
use rc2k_core::saves::tables::{MARKED, TABLES};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "saves::allocate_best_times",
    rva(0x47C4D8),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

/// `dword_747610`: 1 once the memory is taken; `dword_747614` (the file is loaded); then
/// nine pairs (address, size) from `dword_747618` on.
pub const VARS_RVA: usize = rva(0x747610);
pub const VARS_LEN: usize = 0x50;
const LIST_RVA: usize = rva(0x7475D0); // stru_7475D0
const RECORD_TABLE_RVA: usize = rva(0x735576); // g_recordTable
const RECORD_LEN: usize = 0xA4;

/// Where table `n`'s address is among the variables; its size follows.
pub fn pair(vars: *mut u32, n: usize) -> *mut u32 {
    vars.wrapping_add(2 + 2 * n)
}

// ---- Rust version ----

/// `saves::count_stages` (0x47C95D): the records with tag 0, up to the table's -1 (the
/// first record is not looked at for the end).
pub unsafe fn count_stages(table: *const u8) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::saves::tables::count_stages`.
    Ok(rc2k_core::saves::tables::count_stages(|n| unsafe {
        (table.add(n as usize * RECORD_LEN) as *const u32).read_unaligned()
    })?)
}

/// The nine sizes for a table with that many stages. Converted:
/// `rc2k_core::saves::tables::sizes`.
pub fn sizes(stages: u32) -> [u32; TABLES] {
    rc2k_core::saves::tables::sizes(stages)
}

pub unsafe fn allocate_best_times(
    vars: *mut u32,
    list: *mut MemoryList,
    records: *const u8,
) -> Result<(), Refused> {
    unsafe {
        if vars.read_unaligned() == 1 {
            return Ok(());
        }
        let sizes = sizes(count_stages(records)?);
        vars.write_unaligned(1);
        for (n, size) in sizes.into_iter().enumerate() {
            let pair = pair(vars, n);
            pair.add(1).write_unaligned(size);
            let memory = allocate(list, size)?;
            pair.write_unaligned(memory as u32);
            if MARKED.contains(&n) {
                (memory.add(size as usize - 4) as *mut u32).write_unaligned(u32::MAX);
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn game_vars() -> *mut u32 {
    game_addr(VARS_RVA) as *mut u32
}

/// What is wrong with what a run left, if anything: `before` is the variables as they
/// were.
unsafe fn wrong(
    before: &[u32],
    vars: *const u32,
    records: *const u8,
    total_before: u32,
    list: *const MemoryList,
) -> Option<String> {
    unsafe {
        let after = std::slice::from_raw_parts(vars, VARS_LEN / 4).to_vec();
        if before[0] == 1 {
            return (after != before).then(|| "taken before, and yet something changed".into());
        }
        let Ok(stages) = count_stages(records) else {
            return None;
        };
        let sizes = sizes(stages);
        if after[0] != 1 || after[1] != before[1] {
            return Some(format!("the switches are {:#x} {:#x}", after[0], after[1]));
        }
        for (n, size) in sizes.into_iter().enumerate() {
            let (memory, stored) = (after[2 + 2 * n], after[3 + 2 * n]);
            if stored != size || memory == 0 {
                return Some(format!(
                    "table {n}: at {memory:#x}, size {stored:#x}, expected {size:#x}"
                ));
            }
            if MARKED.contains(&n)
                && ((memory + size - 4) as *const u32).read_unaligned() != u32::MAX
            {
                return Some(format!("table {n} has no end mark"));
            }
        }
        let taken = (*list).total.wrapping_sub(total_before);
        let sum = sizes.iter().fold(0u32, |sum, &size| sum.wrapping_add(size));
        (taken != sum).then(|| format!("the list grew by {taken:#x}, expected {sum:#x}"))
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let list = game_addr(LIST_RVA) as *mut MemoryList;
    let records = game_addr(RECORD_TABLE_RVA) as *const u8;
    if !shadow() {
        if let Err(e) = unsafe { allocate_best_times(game_vars(), list, records) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let before = unsafe { std::slice::from_raw_parts(game_vars(), VARS_LEN / 4) }.to_vec();
    let total = unsafe { (*list).total };
    unsafe { call_original(r) };
    if let Some(what) = unsafe { wrong(&before, game_vars(), records, total, list) } {
        HOOK.mismatch(|| format!("taken before: {}; the original left: {what}", before[0]));
    }
}

// ---- self-test ----

const LEN: usize = 0x14F;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x83, 0x3D, 0x10, 0x76, 0x74, 0x00, 0x01];
/// `memory::allocate` nine times and `saves::count_stages` twice.
const CALLS: [usize; 11] = [
    0x28, 0x42, 0x59, 0x6A, 0x84, 0xAD, 0xC7, 0xEE, 0x115, 0x12C, 0x13D,
];
/// How often the copy names its variables, its list and the record table.
const NAMED: (usize, usize, usize) = (22, 1, 2);

/// A record table with these tags, and its end.
fn table(tags: &[u32]) -> Vec<u8> {
    let mut table = vec![0x55u8; tags.len() * RECORD_LEN + 4];
    for (n, tag) in tags.iter().enumerate() {
        table[n * RECORD_LEN..][..4].copy_from_slice(&tag.to_le_bytes());
    }
    let end = tags.len() * RECORD_LEN;
    table[end..].copy_from_slice(&u32::MAX.to_le_bytes());
    table
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x47C4D8)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut vars = [0u32; VARS_LEN / 4];
    let mut list = MemoryList {
        head: 0,
        total: 0,
        cursor: 0,
        remaining: 0,
    };
    // Room for the largest table of the cases, at an address that does not move.
    let mut records = vec![0u8; 8 * RECORD_LEN + 4];
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let (vars_at, list_at, records_at) = (
            game_addr(VARS_RVA) as u32,
            game_addr(LIST_RVA) as u32,
            game_addr(RECORD_TABLE_RVA) as u32,
        );
        let mut patches = Vec::new();
        let mut seen = (0, 0, 0);
        let mut at = 0;
        while at + 4 <= LEN {
            let named = u32::from_le_bytes(code[at..at + 4].try_into().unwrap());
            let new = if (vars_at..vars_at + VARS_LEN as u32).contains(&named) {
                seen.0 += 1;
                Some(vars.as_mut_ptr() as u32 + (named - vars_at))
            } else if named == list_at {
                seen.1 += 1;
                Some(&mut list as *mut MemoryList as u32)
            } else if named == records_at {
                seen.2 += 1;
                Some(records.as_mut_ptr() as u32)
            } else {
                None
            };
            match new {
                Some(new) => {
                    patches.push((at, new));
                    at += 4;
                }
                None => at += 1,
            }
        }
        if seen != NAMED {
            return t.skip("the original is not what the test was made from");
        }
        for (at, new) in patches {
            copy.patch_u32(at, new);
        }
    }

    if unsafe { game_read::<u32>(crate::memory::OWN_HEAP_RVA) } == 1 {
        return t.skip("the game's own heap is in use (altmemenable)");
    }
    let figures: [u32; 3] = unsafe { game_read(MOST_HELD_RVA) };
    let cases: [(&str, &[u32], u32); 6] = [
        ("one rally", &[5, 1, 0, 0, 1, 0], 0),
        ("no stage", &[5, 1], 0),
        ("a stage first", &[0, 4, 0, 3], 0),
        ("only a stage", &[0], 0),
        ("eight records", &[5, 0, 0, 0, 0, 0, 0, 0], 0),
        ("taken before", &[5, 0, 0], 1),
    ];
    for (what, tags, taken) in cases {
        let made = table(tags);
        let mut run = |rust: bool| unsafe {
            records.fill(0x55);
            records[..made.len()].copy_from_slice(&made);
            vars.fill(0xAAAA_AAAA);
            vars[0] = taken;
            list = MemoryList {
                head: 0,
                total: 0x10,
                cursor: 0xAAAA_AAAA,
                remaining: 0xAAAA_AAAA,
            };
            let before = vars.to_vec();
            let ended = if rust {
                match allocate_best_times(vars.as_mut_ptr(), &mut list, records.as_ptr()) {
                    Ok(()) => 0,
                    Err(_) => 2,
                }
            } else {
                let start = Regs::SENTINEL;
                let mut ro = start;
                call_game(&mut ro, copy.addr());
                !ro.preserved_except(&start, 0) as u32
            };
            let wrong = wrong(&before, vars.as_ptr(), records.as_ptr(), 0x10, &list);
            // The variables without the addresses, which are each side's own.
            let mut left = vars.to_vec();
            if taken != 1 {
                for n in 0..TABLES {
                    left[2 + 2 * n] = (left[2 + 2 * n] != 0) as u32;
                }
            }
            let total = list.total;
            let _ = free_list(&mut list);
            (ended, wrong, left, total)
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(
            theirs == mine && theirs.0 == 0 && theirs.1.is_none(),
            || {
                format!(
                    "{what}: original ended {} ({:?}) total {:#x} left {:x?}; rust ended {} ({:?}) total {:#x} left {:x?}",
                    theirs.0, theirs.1, theirs.3, theirs.2, mine.0, mine.1, mine.3, mine.2
                )
            },
        );
    }
    unsafe { game_write(MOST_HELD_RVA, figures) };
    t.finish()
}
