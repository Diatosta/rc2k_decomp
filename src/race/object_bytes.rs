//! `race::object_bytes` (0x455263): four bytes for each of the 0x100 objects, read by
//! `sub_454C99`: `byte_63DC5C` record +0xE bit 0, `byte_63DD5C` 1, `byte_63DE5C` the kinds'
//! byte (`dword_5D3AE8`) of the record's kind (`dword_979F50`, 0x40 bytes each),
//! `byte_63DF5C` 0xFF, or with faces (`dword_632F70`) the kinds' byte of the first face's
//! (`g_faces[dword_632B70]`) +4 word.
//!
//! In:  nothing
//! Out: ebx 0x100, ecx 0x3FC0, eax the last kind value read (a `retn` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45487A` (0x4548A2). What the bytes mean is a
//! reading. The original reads where the tables point, unchecked; so does the Rust
//! version. Shadow mode: by state (the four 0x100-byte tables). Self-test: a copy of the
//! original whose tables are the test's (13 references), on patterns, against the Rust
//! core on the same.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::object_bytes", rva(0x455263), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { object_bytes() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let t = object_tables();
    let regions: Vec<Region> = t[5..].iter().map(|&a| (a, 0x100)).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The test's tables: records, kinds (0x400), face counts, first faces, face pointers,
/// faces (16 of 8 bytes), the four outputs.
const SIZES: [usize; 10] = [
    0x4000, 0x1000, 0x400, 0x400, 0x40, 0x80, 0x100, 0x100, 0x100, 0x100,
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut space = vec![0u8; SIZES.iter().sum::<usize>()];
    let mut starts = Vec::new();
    let mut at = space.as_mut_ptr() as u32;
    for s in SIZES {
        starts.push(at);
        at += s as u32;
    }
    // `OBJECT_TABLES`' order: the faces' own area (starts[5]) is not a table of the game's.
    let tables: Vec<u32> = [0, 1, 2, 3, 4, 6, 7, 8, 9]
        .iter()
        .map(|&k| starts[k])
        .collect();
    let mut patches: Vec<(usize, u32)> = OBJECT_TABLES
        .iter()
        .zip(&tables)
        .map(|(&a, &c)| (a, c))
        .collect();
    patches.push((0x979F5E, tables[0] + 0xE));
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x455263,
            len: 0x75,
            entry: &[0xBB, 0x00, 0x00, 0x00, 0x00, 0x8B, 0xCB, 0xC1],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &patches,
        13,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let total: usize = SIZES.iter().sum();
    let base = starts[0];
    for seed in [1u32, 0x9E37_79B9, 0x1234_5677] {
        let prepare = || unsafe {
            let mut x = seed;
            let mut next = || {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                x
            };
            let w = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
            std::ptr::write_bytes(base as *mut u8, 0xAA, total);
            for n in 0..0x100u32 {
                let r = tables[0] + 0x40 * n;
                w(r, next() & 0x3FF);
                ((r + 0xE) as *mut u8).write(next() as u8);
                w(tables[2] + 4 * n, u32::from(next() % 3 == 0) * next());
                w(tables[3] + 4 * n, next() & 0xF);
            }
            for k in 0..0x400u32 {
                w(tables[1] + 4 * k, next());
            }
            for f in 0..0x10u32 {
                let face = starts[5] + 8 * f;
                w(tables[4] + 4 * f, face);
                ((face + 4) as *mut u16).write_unaligned((next() & 0x3FF) as u16);
            }
        };
        let snapshot = || unsafe { std::slice::from_raw_parts(base as *const u8, total).to_vec() };
        prepare();
        let mut ro = Regs::SENTINEL;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = snapshot();
        prepare();
        let left = unsafe { object_bytes_at(&tables) };
        let mine = snapshot();
        let regs_ok = left.matches(&Regs::SENTINEL, &ro);
        t.check(theirs == mine && regs_ok, || {
            let diff = theirs.iter().zip(&mine).position(|(a, b)| a != b);
            format!("seed {seed:#x}: memory first differs at {diff:x?}, registers ok {regs_ok}")
        });
    }
    drop(space);
    t.finish()
}
