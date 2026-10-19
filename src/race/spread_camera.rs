//! `race::spread_camera` (0x41E6BD): a block's camera byte changed (ebx, the old value in
//! dh): the new value (`[ebx]`) also into the next blocks' same byte (8 bytes on each)
//! while they held the old one, for the blocks after the one shown
//! (`dword_5BECD0 - dword_5C2530` less one at most).
//!
//! In:  ebx the byte, dh its old value
//! Out: dl the new value, eax the blocks gone over (from 1), ecx what was left of the
//!      count; no flags read
//!
//! Callers: `race::viewer_action` (four), `race::next_camera`, `race::previous_camera`,
//! with the block shown not the last. The count is not checked: 0 (or less) is limited
//! only by the bytes met; refused when the walk would pass the 0x708 blocks of the
//! table, before anything is written. Shadow mode compares. Self-test: a table of the
//! test's, runs of the old value of every length, the count reached or not.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{
    BLOCK, BLOCKS, MOST_BLOCKS, Saved, check_original, dispatch_regs, dword, get, put,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::spread_camera", rva(0x41E6BD), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: the registers it leaves.
pub unsafe fn spread_camera(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    let at = |n: u32| (r.ebx.wrapping_add(n.wrapping_mul(8))) as *mut u8;
    let dl = unsafe { (r.ebx as *const u8).read() };
    let old = (r.edx >> 8) as u8;
    let count = get(BLOCKS).wrapping_sub(get(BLOCK));
    // How far it goes, first: (eax, ecx) at the end and the bytes written.
    let (mut eax, mut ecx, mut writes) = (0u32, count, 0u32);
    loop {
        eax = eax.wrapping_add(1);
        ecx = ecx.wrapping_sub(1);
        if ecx == 0 {
            break;
        }
        if eax > MOST_BLOCKS {
            return Err(BadInput("the camera's run goes past the replay's table").into());
        }
        if unsafe { at(eax).read() } != old {
            break;
        }
        writes = eax;
    }
    for n in 1..=writes {
        unsafe { at(n).write(dl) };
    }
    r.eax = eax;
    r.ecx = ecx;
    r.edx = r.edx & !0xFF | dl as u32;
    Ok(r)
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    unsafe { spread_camera(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    // The bytes it may write: the 0x708 blocks' from ebx.
    let regions = [(r.ebx, 8 * MOST_BLOCKS as usize + 1)];
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = Saved::new(&[dword(BLOCK), dword(BLOCKS)]);
    let mut table = vec![0u8; 8 * 40];
    let at = table.as_mut_ptr() as u32;
    for (blocks, run, old) in [
        (10u32, 3usize, 5u8),
        (10, 0, 5),
        (4, 9, 5),
        (1, 4, 5),
        (12, 12, 0x2E),
    ] {
        let prepare = || {
            let table = unsafe { std::slice::from_raw_parts_mut(at as *mut u8, 8 * 40) };
            for (n, b) in table.iter_mut().enumerate() {
                *b = if n % 8 == 3 && (1..=run).contains(&(n / 8)) {
                    old
                } else {
                    n as u8
                };
            }
            table[3] = 9; // the new value
            put(BLOCK, 7);
            put(BLOCKS, 7 + blocks);
        };
        check_original(
            &mut t,
            &format!("count {blocks}, run {run}"),
            call_original,
            Regs {
                ebx: at + 3,
                edx: 0x1234_0000 | (old as u32) << 8 | 0x77,
                ..Regs::SENTINEL
            },
            &[(at, table.len())],
            0,
            &prepare,
            &rust,
        );
    }
    drop(saved);
    t.finish()
}
