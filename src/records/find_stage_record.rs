//! `records::find_stage_record` (0x468289): in a record table (0xA4-byte records, +0 the
//! tag), past section ebx's start (the ebx-th record of tag 4, none for 0), the eax-th record
//! of tag 0 (a stage; counted from 1). (Stream A, part F1: the time trial results page.)
//!
//! In:  edi = the table, ebx = the section, eax = the stage's number
//! Out: edi = the record; eax, ebx, ecx kept (the rest untouched)
//!
//! Caller: `screens::draw_stage_results` (0x48EB91). The original has no end test: a number
//! of 0 (or one past the stages) runs on through memory. Refused, as is a search that finds
//! nothing within 0x1000 records. Shadow mode: by state. Self-test: a test table, sections and
//! numbers.

use crate::errors::{BadInput, Refused};
use crate::framework::{EDI, RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "records::find_stage_record",
    rva(0x468289),
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

const RECORD: u32 = 0xA4;
const LIMIT: u32 = 0x1000;

/// The record, as the original finds it (refused where it would run away).
pub unsafe fn find_stage_record(table: u32, section: u32, number: u32) -> Result<u32, Refused> {
    unsafe {
        let tag = |p: u32| (p as *const u32).read_unaligned();
        let mut p = table;
        let mut steps = 0u32;
        let mut step = |p: &mut u32| -> Result<(), Refused> {
            *p = p.wrapping_add(RECORD);
            steps += 1;
            if steps > LIMIT {
                return Err(BadInput("no such stage record within 0x1000 records").into());
            }
            Ok(())
        };
        if number == 0 {
            return Err(BadInput("stage number 0: the original runs on").into());
        }
        if section != 0 {
            let mut seen = 0u32;
            loop {
                while tag(p) != 4 {
                    step(&mut p)?;
                }
                seen += 1;
                if seen == section {
                    break;
                }
                step(&mut p)?;
            }
        }
        let mut seen = 0u32;
        loop {
            while tag(p) != 0 {
                step(&mut p)?;
            }
            seen += 1;
            if seen == number {
                return Ok(p);
            }
            step(&mut p)?;
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EDI, unsafe { find_stage_record(r.edi, r.ebx, r.eax) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let tags = [1u32, 0, 0, 3, 4, 1, 0, 0, 0, 4, 0, 4, 0, 0];
    let mut table = vec![0u8; RECORD as usize * tags.len()];
    for (n, &tag) in tags.iter().enumerate() {
        table[n * RECORD as usize..n * RECORD as usize + 4].copy_from_slice(&tag.to_le_bytes());
    }
    let p = table.as_ptr() as u32;
    for (section, number) in [(0u32, 1u32), (0, 2), (1, 1), (1, 3), (2, 1), (3, 2)] {
        check_state(
            &mut t,
            &format!("section {section} number {number}"),
            call_original,
            Regs {
                edi: p,
                ebx: section,
                eax: number,
                ..Regs::SENTINEL
            },
            &|| {},
            &[],
            &rust,
        );
    }
    drop(table);
    t.finish()
}
