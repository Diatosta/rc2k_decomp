//! `scene::face_sort_key` (0x450564): the depth a face is sorted by.
//!
//! In:  ebp = the face, ecx = the number of its vertices
//! Out: `dword_62407C` = the key; eax = the last point's record as an offset, ebx = the
//!      number of vertices as `g_polygonVertexCount` has it, ecx = 0, edx = the last
//!      point's record, esi = the face's end; everything else preserved
//!
//! The depths of the face's points (each point's record 0x1000 + (number & 0xFFF), its
//! +0x18) are added one by one, each sum rounded to a float; the sum times the entry
//! `g_polygonVertexCount` of the table `flt_5C6950` (0x100 floats the game fills: one
//! over the number, by the use) is rounded to a whole number (`fistp`).
//! One caller, `scene::list_object_face`.
//! Refuses a count of 0 (the original goes through 2^32), a vertex count past the
//! table, no face and no records. Self-test: the original and the Rust version on records
//! and factors of the test's, at the test precisions (the key within one at extended).

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, EBX, ECX, EDX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm,
    rva,
};
use crate::math::x87;
use crate::scene::polygon::dword;
use crate::scene::project_point::RECORDS_RVA;
use crate::screens::harness::Left;
use crate::service::floats::{
    Floats, ROUNDED, Tolerance, at_test_precisions, check_state_floats, dispatch_state_floats,
    floats_entry,
};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::face_sort_key", rva(0x450564), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const KEY_RVA: usize = rva(0x62407C); // dword_62407C
const FACTORS_RVA: usize = rva(0x5C6950); // flt_5C6950
const FACTORS: u32 = 0x100;

// ---- Rust version ----

pub unsafe fn face_sort_key(face: usize, count: u32) -> Result<Left, Refused> {
    unsafe {
        let records: u32 = game_read(RECORDS_RVA);
        let vertices: u32 = game_read(COUNT_RVA);
        if face == 0 || records == 0 || count == 0 || count > 0x1000 || vertices >= FACTORS {
            return Err(
                BadInput("no face, no records, or a count the key cannot be made for").into(),
            );
        }
        let mut sum = 0f32.to_bits();
        let (mut offset, mut record) = (0, 0);
        for n in 0..count as usize {
            offset = ((dword(face + 0x10 + n * 0x10) & 0xFFF) + 0x1000) << 5;
            record = records.wrapping_add(offset);
            sum = x87::add(dword(record as usize + 0x18), sum);
        }
        let factor: u32 = game_read(FACTORS_RVA + vertices as usize * 4);
        game_write(KEY_RVA, x87::mul_to_int(sum, factor));
        Ok(Left::nothing()
            .with(EAX, offset)
            .with(EBX, vertices)
            .with(ECX, 0)
            .with(EDX, record)
            .with(ESI, (face + 0x10 + count as usize * 0x10) as u32))
    }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [at(KEY_RVA)]
}

/// The converted `face_sort_key` (plain floats): the hook's. The x87 version above stays
/// for `scene::list_object_face`.
// Converted: `rc2k_core::scene::sort_keys`
pub unsafe fn face_sort_key_plain(face: usize, count: u32) -> Result<Left, Refused> {
    use rc2k_core::scene::sort_keys as keys;
    unsafe {
        let records: u32 = game_read(RECORDS_RVA);
        let vertices: u32 = game_read(COUNT_RVA);
        keys::check(face != 0, records != 0, count, vertices)?;
        let (mut offset, mut record) = (0, 0);
        let mut depths = Vec::with_capacity(count as usize);
        for n in 0..count as usize {
            offset = keys::record_offset(dword(face + 0x10 + n * 0x10));
            record = records.wrapping_add(offset);
            depths.push(f32::from_bits(dword(record as usize + 0x18)));
        }
        let factor = f32::from_bits(game_read(FACTORS_RVA + vertices as usize * 4));
        game_write(KEY_RVA, keys::key(depths, factor));
        Ok(Left::nothing()
            .with(EAX, offset)
            .with(EBX, vertices)
            .with(ECX, 0)
            .with(EDX, record)
            .with(ESI, (face + 0x10 + count as usize * 0x10) as u32))
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { face_sort_key_plain(r.ebp as usize, r.ecx) }
}

/// The key, from floats (`fistp`): one off at a rounding edge at extended precision.
fn float_places() -> [(Region, Tolerance); 1] {
    [(at(KEY_RVA), ROUNDED)]
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let inputs = [
        at(RECORDS_RVA),
        at(COUNT_RVA),
        (game_addr(FACTORS_RVA) as u32, 8 * 4),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let depths: [f32; 8] = [
        9000.0, 4096.5, 123456.79, 0.001, -500.25, 16777216.0, 1.0, 3.3333333,
    ];
    let mut records = vec![[0u32; 8]; 0x2000];
    for (n, depth) in depths.into_iter().enumerate() {
        records[0x1000 + 0x30 + n][6] = depth.to_bits();
    }
    let factors: [f32; 8] = [0.0, 1.0, 0.5, 0.33333334, 0.25, 0.2, 0.16666667, 0.14285715];
    for count in 1..=7u32 {
        for (turn, vertices) in [(0u32, count), (3, 7 - count), (5, 1)] {
            // The face's points, from `turn` on round the eight records.
            let mut face = [0u32; 4 + 7 * 4];
            for n in 0..count {
                face[4 + n as usize * 4] = 0x7030 + (n + turn) % 8;
            }
            let before = Regs {
                ebp: face.as_ptr() as u32,
                ecx: count,
                ..Regs::SENTINEL
            };
            let places = float_places();
            let tolerant = Floats {
                memory: &places,
                regs: &[],
            };
            at_test_precisions(&tolerant, &mut |word, held| {
                check_state_floats(
                    &mut t,
                    "scene::face_sort_key",
                    &format!(
                        "control word {word:#x}, {count} points from {turn}, {vertices} vertices"
                    ),
                    call_original,
                    before,
                    &|| unsafe {
                        game_write(KEY_RVA, 0xAAAA_AAAAu32);
                        game_write(RECORDS_RVA, records.as_ptr() as u32);
                        game_write(COUNT_RVA, vertices);
                        game_write(FACTORS_RVA, factors.map(f32::to_bits));
                    },
                    &regions,
                    held,
                    &rust,
                );
            });
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
