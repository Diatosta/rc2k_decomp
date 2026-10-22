//! `race::prepare_sound_tables` (0x40D636): once (`dword_516358`) the 0x22 sound records
//! (`off_514F90`) made ready (spans, the reciprocal at +0xC, three rows each) and the 0x40
//! floats at 0x516258 1 + 0.05 sin(x), x from 0 by pi / 32.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/sound_tables.rs`. One caller, `race::set_up_race_sound_objects` (0x415C93).
//! Shadow mode: the Rust version, the records and tables put back, the original's,
//! compared (the floats within `FLOAT`: the sines go through `fsin`, which no precision
//! rounds). The Rust version is `prepare_sound_tables_plain`. Self-test: a copy whose 20
//! references to the four variables are the test's (records of its own), against the
//! Rust version on the same: not done (records filled two ways) and done.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::sound_tables::*;
use crate::screens::harness::Left;
use crate::service::floats::{
    Agreement, Floats, compare_memory, dispatch_state_floats, floats_entry,
};
use crate::service::harness::capture;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::prepare_sound_tables",
    rva(0x40D636),
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { prepare_sound_tables_plain(&TableVars::game()) };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let v = TableVars::game();
    let places = table_floats(&v);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &table_regions(&v), &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    use crate::audio::test_copy::{Copy, make};
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut records = vec![0u32; RECORDS as usize * RECORD_LEN / 4];
    let table: Vec<u32> = (0..RECORDS)
        .map(|n| records.as_mut_ptr() as u32 + n * RECORD_LEN as u32)
        .collect();
    let mut words = vec![0u32; 2 + SINES as usize];
    let base = words.as_mut_ptr() as u32;
    let v = TableVars {
        done: base,
        records: table.as_ptr() as u32,
        sines: base + 8,
        scratch: base + 4,
    };
    let copy = match make(
        &HOOK,
        &Copy {
            address: 0x40D636,
            len: 0xD0,
            entry: &[0x60, 0x83, 0x3D, 0x58, 0x63, 0x51, 0x00, 0x01],
            calls: &[0x24, 0x3A],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x514F90, v.records),
            (0x516358, v.done),
            (0x516258, v.sines),
            (0x5183CC, v.scratch),
        ],
        20,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let regions = table_regions(&v);
    let places = table_floats(&v);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    let values = [
        0u32,
        7,
        0xFFFF_FFF6,
        0x3F80_0000,
        0x4000_0000,
        1000,
        0x0100_0000,
    ];
    for (done, k) in [(0u32, 0usize), (0, 3), (1, 1)] {
        let prepare = || unsafe {
            for (n, w) in std::slice::from_raw_parts_mut(table[0] as *mut u32, records.len())
                .iter_mut()
                .enumerate()
            {
                *w = values[(n + k) % values.len()];
            }
            (v.done as *mut u32).write(done);
            (v.scratch as *mut u32).write(0x5A5A_5A5A);
            for n in 0..SINES {
                ((v.sines + 4 * n) as *mut u32).write(0x5A5A_5A5A);
            }
        };
        prepare();
        let before = Regs::SENTINEL;
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (capture(&regions), ro.preserved_except(&before, 0));
        prepare();
        unsafe { prepare_sound_tables_plain(&v) };
        let mine = capture(&regions);
        let same = compare_memory(&regions, &theirs.0, &mine, &floats) != Agreement::Differ;
        t.check(same && theirs.1, || {
            format!(
                "done {done}, values from {k}: same {same}, registers kept {}",
                theirs.1
            )
        });
    }
    drop((records, words));
    t.finish()
}
