//! `records::find_record` (0x47255D): finds a record by index within a section of a record table.
//!
//! In:  eax = index, ebx = section (0 = from the table start), edi = table
//! Out: edi = record pointer; eax, ebx, ecx and edx preserved
//! Section N starts after the Nth record whose tag is 4. There is no end-of-table check.

use crate::bridge::{Raw, read_through};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("records::find_record", rva(0x47255D), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// The record, as the original finds it (converted: `rc2k_core::files::records::find_record`, on
/// the table in game memory: no end test, as the original).
pub fn find_record_in(table: u32, index: i32, section: u32) -> u32 {
    let records = unsafe { Raw::new(table as usize as *const u8) };
    let place = read_through(rc2k_core::files::records::find_record(
        &records, index, section,
    ));
    table.wrapping_add(place as u32)
}

/// True if `table` has at least `section` section headers among its first `limit` records:
/// a bounded check that `find_record_in` will not run off the table (converted:
/// `rc2k_core::files::records::has_section`).
pub fn has_section(table: u32, section: u32, limit: u32) -> bool {
    let records = unsafe { Raw::new(table as usize as *const u8) };
    rc2k_core::files::records::has_section(&records, section, limit as usize)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (index, section, table) = (r.eax as i32, r.ebx, r.edi);
    let mine = find_record_in(table, index, section);

    if !shadow() {
        r.edi = mine;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.edi != mine {
        HOOK.mismatch(|| {
            format!(
                "index={index} section={section} table={table:#x}: original={:#x} rust={mine:#x}",
                o.edi
            )
        });
    }
    r.edi = o.edi;
}

// ---- self-test: synthetic tables ----

/// First dword of each record is rc2k_core::files::records::SECTION at `markers`, never 4 elsewhere (including near
/// misses). A 4 in another field must not count.
fn make_table(records: usize, markers: &[usize]) -> Vec<u8> {
    const FILLER: [u32; 8] = [0, 1, 2, 3, 5, 0x0400_0000, 0xFFFF_FFFF, 0x104];
    let size = rc2k_core::files::records::RECORD_LEN;
    let mut t = vec![0u8; records * size];
    for i in 0..records {
        let tag = if markers.contains(&i) {
            rc2k_core::files::records::SECTION
        } else {
            FILLER[i % 8]
        };
        t[i * size..i * size + 4].copy_from_slice(&tag.to_le_bytes());
        t[i * size + 4..i * size + 8]
            .copy_from_slice(&rc2k_core::files::records::SECTION.to_le_bytes());
    }
    t
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The original never stops at the end of a table, so only ask for sections that exist.
    let tables = [
        (make_table(24, &[2, 5, 6, 11, 17]), 5u32),
        (make_table(8, &[0, 1, 2, 3]), 4),
    ];

    for (table, markers) in &tables {
        let base = table.as_ptr() as u32;
        for section in 0..=*markers {
            for index in -3..=10i32 {
                let mut ro = Regs {
                    eax: index as u32,
                    ebx: section,
                    ecx: 0x1234_5678,
                    edx: 0x9ABC_DEF0,
                    edi: base,
                    ..Regs::default()
                };
                unsafe { call_original(&mut ro) };
                let mine = find_record_in(base, index, section);

                let preserved = ro.eax == index as u32
                    && ro.ebx == section
                    && ro.ecx == 0x1234_5678
                    && ro.edx == 0x9ABC_DEF0;
                t.check(ro.edi == mine && preserved, || {
                    format!(
                        "index={index} section={section} original edi={:#x} rust={mine:#x} preserved={preserved}",
                        ro.edi
                    )
                });
            }
        }
    }
    t.finish()
}
