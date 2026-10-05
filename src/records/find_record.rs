use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, hook_asm, log};
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "records::find_record",
    0x47255D - 0x400000,
    thunk,
    &TRAMPOLINE,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = TRAMPOLINE
);

const RECORD_SIZE: u32 = 0xA4;
const SECTION_TAG: u32 = 4;

fn read_tag(addr: u32) -> u32 {
    unsafe { (addr as usize as *const u32).read_unaligned() }
}

/// Mirrors the original, including having no end-of-table check.
fn find_record(table: u32, index: i32, section: u32, tag_at: impl Fn(u32) -> u32) -> u32 {
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

// eax = index, ebx = section, edi = table -> pointer in edi; eax/ebx/ecx preserved
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let (index, section, table) = (r.eax as i32, r.ebx, r.edi);
    let mine = find_record(table, index, section, read_tag);

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        r.edi = mine;
        return;
    }

    let mut o = *r;
    unsafe { call_original(&mut o) };
    if o.edi != mine {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH find_record(index={index}, section={section}, table={table:#x}): original={:#x} rust={mine:#x}",
                o.edi
            ));
        }
    }
    r.edi = o.edi;
}

/// A synthetic table: first dword of each record is SECTION_TAG at `markers`, never 4 elsewhere
/// (including near misses). A 4 in another field must not count.
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

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 {
        log("find_record selftest SKIPPED: original not available");
        return (0, 0);
    }
    // The original never stops at the end of a table, so only ask for sections that exist.
    let tables = [
        (make_table(24, &[2, 5, 6, 11, 17]), 5u32),
        (make_table(8, &[0, 1, 2, 3]), 4),
    ];
    let (mut cases, mut failures) = (0, 0);

    for (table, markers) in &tables {
        let base = table.as_ptr() as u32;
        for section in 0..=*markers {
            for index in -3..=10i32 {
                cases += 1;
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

                // also checks the register contract: eax, ebx, ecx, edx preserved
                let ok = ro.edi == mine
                    && ro.eax == index as u32
                    && ro.ebx == section
                    && ro.ecx == 0x1234_5678
                    && ro.edx == 0x9ABC_DEF0;
                if !ok {
                    failures += 1;
                    log(&format!(
                        "find_record selftest FAILED: index={index} section={section} original edi={:#x} rust={mine:#x}",
                        ro.edi
                    ));
                }
            }
        }
    }
    (cases, failures)
}
