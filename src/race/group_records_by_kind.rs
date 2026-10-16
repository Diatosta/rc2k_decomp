//! `race::group_records_by_kind` (0x421E42): the course's records put in order of kind,
//! each kind's first and last noted.
//!
//! In:  nothing
//! Out: ecx 0x100, eax, ebx, edx, esi, edi, ebp as `group_records_by_kind` says
//!
//! See `race/course_data.rs`. One caller, `sub_4394CB` (locacurrcourse). Refuses more than
//! 0x78000 records. Self-test: records of the test's in the game's pointer and count, the
//! two tables put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::race::course_data::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::group_records_by_kind",
    rva(0x421E42),
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

fn tables() -> Region {
    (game_addr(KIND_TABLES_RVA) as u32, KIND_TABLES_LEN)
}

fn regions() -> Vec<Region> {
    let count: u32 = unsafe { game_read(RECORD_COUNT_RVA) };
    let records: u32 = unsafe { game_read(RECORDS_RVA) };
    let len = if count <= MOST_RECORDS {
        count as usize * 0x20
    } else {
        0
    };
    vec![(records, len), tables()]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        group_records_by_kind(r)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = [tables()];
    let saved = capture(&kept);
    for (count, spread) in [(0u32, 1u32), (1, 1), (5, 3), (40, 0x30), (64, 0x100)] {
        let records = TestTable::new(RECORDS_RVA, RECORD_COUNT_RVA, 0x20, count, 65, &|n, r| {
            let mut s = (n as u32).wrapping_mul(0x9E37_79B9) ^ count;
            for d in r.chunks_mut(4) {
                s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
                d.copy_from_slice(&s.to_le_bytes());
            }
            // The kind: low four bits and bits 11 .. 14, among `spread` kinds.
            let kind = (s >> 7) % spread;
            let d = u32::from_le_bytes(r[..4].try_into().unwrap()) & !0x780F;
            r[..4].copy_from_slice(&(d | (kind & 0xF) | ((kind >> 4) << 11)).to_le_bytes());
        });
        let regions = [records.region(), tables()];
        check_state(
            &mut t,
            &format!("{count} records over {spread:#x} kinds"),
            call_original,
            Regs::SENTINEL,
            &|| records.reset(),
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
