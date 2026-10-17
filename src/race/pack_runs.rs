//! `race::pack_runs` (0x41EFEC): ecx bytes at esi packed to edi by runs: 2 .. 0x80 equal
//! bytes as (length - 1, the byte), other stretches of up to 0x80 as (-length, the
//! bytes); fewer than three bytes or one left as one literal; then a dword 0 of which
//! one byte counts.
//!
//! In:  esi = the bytes, ecx = their count, edi = where the packed bytes go
//! Out: ecx = the bytes written; the rest preserved
//!
//! See `race/save_seal.rs`. One caller, `sub_41EEE4` (the results file; `add ecx, 3`
//! follows, no flags read). Shadow mode: the Rust version into a buffer of its own, the
//! original into the caller's, the two compared (the caller's buffer's size is not
//! known, so it is not captured). Self-test: sources of the test's (runs of 1 .. 300,
//! literal stretches past 0x80, mixes, 0 .. 3 bytes), buffers compared whole.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::race::save_seal::pack_runs;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::pack_runs", rva(0x41EFEC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Room for a packing of `count` bytes: one header per 0x80, the dword after.
fn room(count: u32) -> usize {
    count as usize + count as usize / 0x40 + 16
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        let mut mine = vec![0xAAu8; room(r.ecx)];
        let n = unsafe { pack_runs(r.esi, r.ecx, mine.as_mut_ptr() as u32) };
        let before = *r;
        unsafe { call_original(r) };
        let theirs = unsafe { std::slice::from_raw_parts(before.edi as *const u8, r.ecx as usize) };
        if r.ecx != n
            || theirs != &mine[..n as usize]
            || !r.preserved_except(&before, crate::framework::ECX)
        {
            HOOK.mismatch(|| format!("{} bytes: original {} written, rust {n}", before.ecx, r.ecx));
        }
        return;
    }
    r.ecx = unsafe { pack_runs(r.esi, r.ecx, r.edi) };
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut sources: Vec<Vec<u8>> = vec![vec![], vec![7], vec![7, 7], vec![1, 2], vec![5, 5, 5]];
    sources.push(vec![9; 300]);
    sources.push((0..300u32).map(|n| (n * 7 + n / 3) as u8).collect());
    sources.push(
        (0..400u32)
            .map(|n| if n % 50 < 20 { 0 } else { (n % 7) as u8 })
            .collect(),
    );
    sources.push((0..0x81u32).map(|n| n as u8).collect());
    sources.push([vec![1, 2, 3], vec![4; 0x80], vec![4; 0x81], vec![5, 6]].concat());
    sources.push(
        (0..64u32)
            .flat_map(|n| vec![n as u8; (n % 5 + 1) as usize])
            .collect(),
    );
    for (k, src) in sources.iter().enumerate() {
        let mut theirs = vec![0xAAu8; room(src.len() as u32)];
        let mut mine = theirs.clone();
        let before = Regs {
            esi: src.as_ptr() as u32,
            ecx: src.len() as u32,
            edi: theirs.as_mut_ptr() as u32,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_original(&mut ro) };
        let n = unsafe { pack_runs(before.esi, before.ecx, mine.as_mut_ptr() as u32) };
        t.check(
            ro.ecx == n && theirs == mine && ro.preserved_except(&before, crate::framework::ECX),
            || {
                format!(
                    "source {k} ({} bytes): original {} {:x?}; rust {n} {:x?}",
                    src.len(),
                    ro.ecx,
                    &theirs[..16.min(theirs.len())],
                    &mine[..16.min(mine.len())]
                )
            },
        );
    }
    t.finish()
}
