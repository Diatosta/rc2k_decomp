//! `records::default_stage_records` (0x47C7E0): gives every stage of a record table a
//! made-up best time and top speed.
//!
//! In:  edi = the record table
//! Out: every register preserved
//! Nothing if the first record's byte +0x54 is not 0 (it has a name there already).
//! Else for every record with tag 0, up to the table's -1 (`sub_47CE05`, a plain function
//! here), from four random numbers under 0xAA: a driver, whose +0x14 goes to the
//! record's +0x74, +0x18 to +0x78 and name (without a NUL) to +0x54; the second + 1000
//! to +0x30; another driver to +0x9C, +0xA0 and +0x7C; the fourth + 1000 to +0x2C.
//! One caller, `menu::set_up_front_end`, on each of thirteen record tables when
//! `var\ral.dat` was not loaded.
//! Refuses a null table, a table without its end, and a name that does not fit.

use crate::bridge::Raw;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::math::next_random::STATE_RVA;
use crate::math::random_below::random_below;
use crate::saves::drivers::{Driver, put, put_bytes};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "records::default_stage_records",
    rva(0x47C7E0),
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

const RECORD_LEN: usize = 0xA4;
/// More records than any table has.
const MOST_RECORDS: usize = 0x1000;

// ---- Rust version ----

/// How many records the table has before its -1 (the first is not looked at for the
/// end, as in the original).
unsafe fn records(table: u32) -> Result<usize, BadInput> {
    // Converted: `rc2k_core::records::record_count`.
    let records = unsafe { Raw::new(table as usize as *const u8) };
    Ok(rc2k_core::records::record_count(&records, MOST_RECORDS)?)
}

pub unsafe fn default_stage_records(table: u32) -> Result<(), Refused> {
    unsafe {
        if table == 0 {
            return Err(BadInput("no record table").into());
        }
        if ((table + 0x54) as *const u8).read() != 0 {
            return Ok(());
        }
        // (the record, the dwords by offset, the two names by offset)
        type Made = (u32, [(u32, u32); 6], [(u32, Vec<u8>); 2]);
        let mut made: Vec<Made> = Vec::new();
        for n in 0..records(table)? {
            let record = table + (n * RECORD_LEN) as u32;
            if (record as *const u32).read_unaligned() != 0 {
                continue;
            }
            let first = Driver::number(random_below(0xAA)?)?;
            let time = random_below(0xAA)?.wrapping_add(1000);
            let second = Driver::number(random_below(0xAA)?)?;
            let speed = random_below(0xAA)?.wrapping_add(1000);
            made.push((
                record,
                [
                    (0x74, first.dword(0x14)),
                    (0x78, first.dword(0x18)),
                    (0x30, time),
                    (0x9C, second.dword(0x14)),
                    (0xA0, second.dword(0x18)),
                    (0x2C, speed),
                ],
                [(0x54, first.name()?.1), (0x7C, second.name()?.1)],
            ));
        }
        for (record, dwords, names) in made {
            for (offset, value) in dwords {
                put(record, offset, value);
            }
            for (offset, name) in names {
                put_bytes(record, offset, &name);
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { default_stage_records(r.edi) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let Ok(count) = (unsafe { records(r.edi) }) else {
        HOOK.count_call();
        let e = BadInput("a record table without its end");
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        return;
    };
    let regions = [(r.edi, count * RECORD_LEN), at(STATE_RVA)];
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test: record tables of the test's, filled by both from the same random
// numbers ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let seeds = [at(STATE_RVA)];
    let saved = capture(&seeds);
    let cases: [(&str, &[u32], u8); 5] = [
        ("a rally", &[5, 1, 0, 0, 1, 0, 4, 1, 0], 0),
        ("only stages", &[0, 0, 0], 0),
        ("no stage", &[5, 1, 3], 0),
        ("named already", &[0, 0, 0], b'x'),
        ("a stage first", &[0, 4, 0], 0),
    ];
    for (what, tags, named) in cases {
        let mut table = vec![0xAAu8; tags.len() * RECORD_LEN + 4];
        let fresh = |table: &mut Vec<u8>| {
            table.fill(0xAA);
            for (n, tag) in tags.iter().enumerate() {
                table[n * RECORD_LEN..][..4].copy_from_slice(&tag.to_le_bytes());
            }
            let end = tags.len() * RECORD_LEN;
            table[end..].copy_from_slice(&u32::MAX.to_le_bytes());
            table[0x54] = named;
        };
        fresh(&mut table);
        let address = table.as_mut_ptr();
        let template = table.clone();
        let regions = [(address as u32, table.len()), at(STATE_RVA)];
        for seed in [7u32, 0xCAFE_F00D] {
            check_state(
                &mut t,
                &format!("{what}, seed {seed:#x}"),
                call_original,
                Regs {
                    edi: address as u32,
                    ..Regs::SENTINEL
                },
                &|| unsafe {
                    std::ptr::copy(template.as_ptr(), address, template.len());
                    game_write(STATE_RVA, seed);
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&seeds, &saved);
    t.finish()
}
