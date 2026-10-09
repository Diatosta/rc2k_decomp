//! `records::find_record` (0x47255D): finds a record by index within a section of a record table.
//!
//! In:  eax = index, ebx = section (0 = from the table start), edi = table
//! Out: edi = record pointer; eax, ebx, ecx and edx preserved
//! Section N starts after the Nth record whose tag is 4. There is no end-of-table check.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::records::record::Record;
use std::ptr::addr_of;
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

const RECORD_SIZE: u32 = size_of::<Record>() as u32;
const SECTION_TAG: u32 = 4;

fn read_tag(addr: u32) -> u32 {
    let rec = addr as usize as *const Record;
    unsafe { addr_of!((*rec).tag).read_unaligned() }
}

/// Mirrors the original, including having no end-of-table check.
pub fn find_record(table: u32, index: i32, section: u32, tag_at: impl Fn(u32) -> u32) -> u32 {
    let mut p = table;
    if section != 0 {
        let mut seen = 0u32;
        loop {
            while tag_at(p) != SECTION_TAG {
                p = p.wrapping_add(RECORD_SIZE);
            }
            seen = seen.wrapping_add(1);
            if seen == section {
                break;
            }
            p = p.wrapping_add(RECORD_SIZE);
        }
        p = p.wrapping_add(RECORD_SIZE); // step past the matching record
    }
    p.wrapping_add(index.wrapping_mul(RECORD_SIZE as i32) as u32)
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let (index, section, table) = (r.eax as i32, r.ebx, r.edi);
    let mine = find_record(table, index, section, read_tag);

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

/// First dword of each record is SECTION_TAG at `markers`, never 4 elsewhere (including near
/// misses). A 4 in another field must not count.
fn make_table(records: usize, markers: &[usize]) -> Vec<u8> {
    const FILLER: [u32; 8] = [0, 1, 2, 3, 5, 0x0400_0000, 0xFFFF_FFFF, 0x104];
    let size = RECORD_SIZE as usize;
    let mut t = vec![0u8; records * size];
    for i in 0..records {
        let tag = if markers.contains(&i) {
            SECTION_TAG
        } else {
            FILLER[i % 8]
        };
        t[i * size..i * size + 4].copy_from_slice(&tag.to_le_bytes());
        t[i * size + 4..i * size + 8].copy_from_slice(&SECTION_TAG.to_le_bytes());
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
                let mine = find_record(base, index, section, read_tag);

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
