//! `physics::edge_side` (0x4425EF): which side of the edge from the point at ebp to the
//! point at edi (whole-number x at +0, z at +8) the point (`dword_639824`,
//! `dword_63982C`) lies: (X - x0)(z1 - z0) - (Z - z0)(x1 - x0) in 64 bits (`imul`, then
//! `sub`/`sbb`), the second product kept in `qword_620460`.
//!
//! In:  ebp, edi the edge's ends
//! Out: eax 1 with ZF and CF clear when the ends have the same x and z (nothing stored);
//!      else eax 0, ZF set, CF set when the difference is negative. The rest kept (ebx,
//!      edx pushed and popped). The callers read ZF (`jnz`) then CF (`jb`).
//!
//! Callers (8): `sub_442006`, the face under a point (each edge of a triangle or a quad).
//! Shadow mode: `physics/flag_result.rs`; both versions store the same `qword_620460` (a
//! scratch value), which only the self-test compares.
//! Self-test: a copy of the original on a test point and qword, edges by lot (some with
//! the same ends, some large enough to overflow 32 bits).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::physics::flag_result::{FlagResult, dispatch_flags};
use rc2k_core::physics::faces;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::edge_side", rva(0x4425EF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The point's and the scratch value's addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct EdgePlaces {
    pub x: u32,
    pub z: u32,
    pub product: u32,
}

impl EdgePlaces {
    pub fn game() -> Self {
        let p = |x: usize| game_addr(rva(x)) as u32;
        Self {
            x: p(0x639824),
            z: p(0x63982C),
            product: p(0x620460),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core; `eax` what eax held at entry is not needed (it is always set).
// Converted: `rc2k_core::physics::faces::edge_side`.
pub unsafe fn edge_side_at(p: &EdgePlaces, from: u32, to: u32) -> FlagResult {
    let result = |eax, carry, zero| FlagResult {
        eax,
        ecx: None,
        ebx: None,
        carry,
        zero: Some(zero),
    };
    unsafe {
        let point = (at(p.x), at(p.z));
        match faces::edge_side(point, (at(from), at(from + 8)), (at(to), at(to + 8))) {
            None => result(1, false, false),
            Some((kept, negative)) => {
                (p.product as *mut i64).write_unaligned(kept);
                result(0, negative, true)
            }
        }
    }
}

/// The Rust version.
pub unsafe fn edge_side(from: u32, to: u32) -> FlagResult {
    unsafe { edge_side_at(&EdgePlaces::game(), from, to) }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { edge_side(r.ebp, r.edi) })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_flags(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // x, z, the product (2), the two ends (3 dwords each).
    let mut cells = vec![0u32; 4 + 6];
    let base = cells.as_mut_ptr() as u32;
    let p = EdgePlaces {
        x: base,
        z: base + 4,
        product: base + 8,
    };
    let (from, to) = (base + 16, base + 28);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x4425EF,
            len: 0x5C,
            entry: &[0x53, 0x52, 0x8B, 0x45, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[
            (0x639824, p.x),
            (0x63982C, p.z),
            (0x620460, p.product),
            (0x620464, p.product + 4),
        ],
        6,
    ) {
        Ok(copy) => copy,
        Err(why) => return t.skip(&why),
    };
    let fill = |seed: u32| {
        let mut s = seed.wrapping_mul(0x2545_F491);
        let mut v = [0u32; 10];
        for x in v.iter_mut() {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            *x = match seed % 4 {
                0 => s,
                1 => (s as i32 >> 12) as u32,
                _ => (s as i32 >> 4) as u32,
            };
        }
        v[2] = 0xAAAA_AAAA;
        v[3] = 0xAAAA_AAAA;
        if seed.is_multiple_of(7) {
            v[7] = v[4];
            v[9] = v[6];
        }
        if seed.is_multiple_of(11) {
            v[7] = v[4]; // same x only
        }
        unsafe { (base as *mut [u32; 10]).write(v) };
    };
    for seed in 1..=200u32 {
        fill(seed);
        let before = Regs {
            ebp: from,
            edi: to,
            ..Regs::SENTINEL
        };
        let mut ro = before;
        unsafe { call_game(&mut ro, copy.addr()) };
        let theirs = unsafe { (base as *const [u32; 10]).read() };
        fill(seed);
        let mine = unsafe { edge_side_at(&p, from, to) };
        let state = unsafe { (base as *const [u32; 10]).read() };
        t.check(mine.matches(&before, &ro) && theirs == state, || {
            format!(
                "seed {seed}: original eax {:#x} carry {} zero {} {theirs:x?}; rust {mine:?} {state:x?}",
                ro.eax,
                ro.carry(),
                crate::audio::buffer_play::zero(&ro)
            )
        });
    }
    drop((copy, cells));
    t.finish()
}
