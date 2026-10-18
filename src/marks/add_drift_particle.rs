//! `marks::add_drift_particle` (0x4CFF57): one entry of the marks' particle ring
//! (`g_marksRingFree`'s +0, while `g_marksRingUsed` is not 0xFFF) started from the six
//! values on the x87 stack: +8 eax (the kind), st(0) .. st(2) at +0x34, +0x2C, +0x28 (the
//! motion), st(3) at +0x14 and +0x20, st(4) at +0x10 and (less `flt_93C644`) +0x1C, st(5)
//! at +0xC and +0x18 (the place, now and last); +0x38 0x8000, +0x3C 0xFF000000, +0x44
//! 50000, +0x24 the view's +0x34 (`flt_5EA3CC`). With the ring full the six are dropped.
//!
//! In:  st(0) .. st(5) as above (all popped), eax the kind
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4CFB49` (every eighth marks step, a drift puff by the view). Refuses a null
//! ring (an entry to take), before anything. Thunk by `six_in_hook!`. Self-test: a ring
//! of the test's, the original and the Rust version at several fillings and values (some
//! unrounded).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::marks::blocks::{RING_ENTRY, RING_FREE_RVA, RING_USED_RVA};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::vector3::{Six80, six_in_hook};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static SIX: Six80 = Six80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::add_drift_particle", rva(0x4CFF57), thunk, &ORIGINAL).self_test(self_test);

six_in_hook!(
    thunk,
    call_original,
    original_with_six,
    dispatch = dispatch,
    original = ORIGINAL,
    six = SIX
);

const FULL: u32 = 0xFFF;
const LOWER_RVA: usize = rva(0x93C644); // flt_93C644
const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view's place at +0x30
const LIFE: u32 = 0xC350;

/// The Rust version (see the header).
pub unsafe fn add_drift_particle(kind: u32, v: [[u8; 10]; 6]) -> Result<(), Refused> {
    unsafe {
        let used: u32 = game_read(RING_USED_RVA);
        if used == FULL {
            return Ok(());
        }
        let free: u32 = game_read(RING_FREE_RVA);
        if free == 0 || (free as *const u32).read_unaligned() == 0 {
            return Err(BadInput("a null particle ring entry").into());
        }
        let e = (free as *const u32).read_unaligned();
        game_write(RING_FREE_RVA, e);
        game_write(RING_USED_RVA, used.wrapping_add(1));
        let put = |o: u32, x: u32| ((e + o) as *mut u32).write_unaligned(x);
        put(8, kind);
        put(0x34, rounded(v[0]));
        put(0x2C, rounded(v[1]));
        put(0x28, rounded(v[2]));
        put(0x14, rounded(v[3]));
        put(0x20, rounded(v[3]));
        put(0x10, rounded(v[4]));
        put(0x1C, x87::ext_sub(v[4], game_read(LOWER_RVA)));
        put(0xC, rounded(v[5]));
        put(0x18, rounded(v[5]));
        put(0x38, 0x8000);
        put(0x3C, 0xFF00_0000);
        put(0x44, LIFE);
        put(0x24, x87::loaded(game_read(VIEW_RVA + 0x34)));
    }
    Ok(())
}

/// What it may write: the ring's two globals and the entry it takes.
fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = [RING_FREE_RVA, RING_USED_RVA].map(at).to_vec();
    unsafe {
        let free: u32 = game_read(RING_FREE_RVA);
        if free != 0 {
            let e = (free as *const u32).read_unaligned();
            if e != 0 {
                r.push((e, RING_ENTRY as usize));
            }
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { add_drift_particle(r.eax, SIX.inputs()) }?;
    Ok(left_of(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, original_with_six, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const TEST_ENTRIES: u32 = 8;
    let mut ring = vec![0u8; (TEST_ENTRIES * RING_ENTRY) as usize];
    let base = ring.as_mut_ptr() as u32;
    let game: Vec<Region> = [RING_FREE_RVA, RING_USED_RVA, LOWER_RVA]
        .map(at)
        .into_iter()
        .chain([(game_addr(VIEW_RVA) as u32 + 0x34, 4)])
        .collect();
    let saved = capture(&game);
    let mut regions = game.clone();
    regions.push((base, ring.len()));
    // An unrounded value: 1/3 in 80 bits.
    let third = {
        let (mut out, three) = ([0u8; 10], 3.0f32.to_bits());
        x87::x87!("fld1", "fdiv dword ptr [{t}]", "fstp tbyte ptr [{o}]";
            t = &raw const three, o = out.as_mut_ptr());
        out
    };
    for (case, used) in [0u32, 0x10, 0xFFE, FULL, 0x1000].into_iter().enumerate() {
        let k = case as f32;
        let six = [
            extended((0.5 + k).to_bits()),
            third,
            extended((-3.25 * k).to_bits()),
            extended((100.0 + k).to_bits()),
            if case % 2 == 0 {
                third
            } else {
                extended(7.75f32.to_bits())
            },
            extended((-1e4f32 + k).to_bits()),
        ];
        check_state(
            &mut t,
            &format!("used {used:#x}, case {case}"),
            original_with_six,
            Regs {
                eax: case as u32 * 4,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&game, &saved);
                SIX.set_inputs(six);
                std::ptr::write_bytes(base as *mut u8, 0xAA, ring.len());
                for n in 0..TEST_ENTRIES {
                    let e = (base + n * RING_ENTRY) as *mut u32;
                    e.write_unaligned(base + ((n + 1) % TEST_ENTRIES) * RING_ENTRY);
                }
                game_write(RING_FREE_RVA, base + 2 * RING_ENTRY);
                game_write(RING_USED_RVA, used);
                game_write(LOWER_RVA, (0.375f32 + k).to_bits());
                game_write(VIEW_RVA + 0x34, (12.5f32 - k).to_bits());
            },
            &regions,
            &rust,
        );
    }
    restore(&game, &saved);
    drop(ring);
    t.finish()
}
