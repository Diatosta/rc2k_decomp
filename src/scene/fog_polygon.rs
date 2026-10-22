//! `scene::fog_polygon` (0x44E286): each of the polygon's points gets its fog by its
//! depth.
//!
//! In:  nothing
//! Out: eax = the last byte stored, ecx = the number of vertices, edx = the last
//!      point's record, esi = the last entry; everything else preserved
//!
//! For each entry: the depth of its point's record (+0x18) as a whole number (`fistp`,
//! kept in `dword_62400C`), under 0 taken as 0, over 0x2000 is the place in the table
//! `byte_63661C` (`sub_44E260`; its length `dword_63761C`, a place past it is the last
//! one); the entry's byte +0x20 is that table's byte inverted. With `dword_6365C8` at 4
//! the byte is the low byte of `dword_636610` inverted instead.
//! One caller, the six-byte `sub_44E25A`, which six functions call.
//! Refuses a polygon of no vertices (the original goes through 2^32), an entry without
//! its record and a table's length above 0x1000. A length of 0, as the menus have it,
//! makes every place the byte before the table (the original's `length - 1`): kept.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, ECX, EDX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::{ENTRIES, ENTRIES_RVA, ENTRY_LEN, LIST_RVA, dword, listed, put};
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::fog_polygon", rva(0x44E286), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const TABLE_RVA: usize = rva(0x63661C); // byte_63661C
const TABLE_LEN: u32 = 0x1000;
const LENGTH_RVA: usize = rva(0x63761C); // dword_63761C
const KIND_RVA: usize = rva(0x6365C8); // dword_6365C8
const FIXED_RVA: usize = rva(0x636610); // dword_636610

// ---- Rust version ----

pub unsafe fn fog_polygon() -> Result<Left, Refused> {
    unsafe {
        let entries = listed()?;
        let Some(&last) = entries.last() else {
            return Err(BadInput("a polygon of no vertices").into());
        };
        if entries.iter().any(|&entry| dword(entry + 4) == 0) {
            return Err(BadInput("an entry without its record").into());
        }
        let length: u32 = game_read(LENGTH_RVA);
        if length > TABLE_LEN {
            return Err(BadInput("a fog table longer than its room").into());
        }
        let fixed = (game_read::<u32>(KIND_RVA) == 4).then(|| game_read::<u32>(FIXED_RVA));
        let (mut eax, mut record) = (0, 0);
        for &entry in &entries {
            record = dword(entry + 4);
            let depth = x87::to_int(dword(record as usize + 0x18));
            game_write(CHECKED_RVA, depth);
            // A place past the length is the last one; with no length (the menus)
            // that is the byte before the table, as in the original.
            let place = depth.max(0) as u32 >> 13;
            let place = if (place.wrapping_sub(length) as i32) < 0 {
                place
            } else {
                length.wrapping_sub(1)
            };
            let fog =
                ((game_addr(TABLE_RVA) as u32).wrapping_add(place) as *const u8).read() as u32;
            eax = fixed.unwrap_or(fog) ^ 0xFF;
            ((entry + 0x20) as *mut u8).write(eax as u8);
        }
        Ok(Left::nothing()
            .with(EAX, eax)
            .with(ECX, entries.len() as u32)
            .with(EDX, record)
            .with(ESI, last as u32))
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![at(CHECKED_RVA)];
    for entry in unsafe { listed() }.unwrap_or_default() {
        regions.push((entry as u32 + 0x20, 1));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { fog_polygon() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [
        (game_addr(ENTRIES_RVA) as u32, ENTRIES as usize * ENTRY_LEN),
        at(CHECKED_RVA),
    ];
    let inputs = [
        (game_addr(LIST_RVA) as u32, 4 * 4),
        (game_addr(TABLE_RVA) as u32, TABLE_LEN as usize),
        at(COUNT_RVA),
        at(LENGTH_RVA),
        at(KIND_RVA),
        at(FIXED_RVA),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let table: Vec<u8> = (0..TABLE_LEN).map(|n| (n * 7 + n / 256) as u8).collect();
    let depths: [[f32; 4]; 5] = [
        [0.0, 8191.9, 8192.5, 40000.0],
        [-5.0, 123456.0, 1e12, f32::NAN],
        [16384.0, 24575.5, 24576.5, 0.49],
        [f32::INFINITY, f32::NEG_INFINITY, 2e9, 3e9],
        [4096.0, 262144.0, 33554430.0, 1.0],
    ];
    for depths in depths {
        let mut records = [[0xAAAA_AAAAu32; 8]; 4];
        for (record, depth) in records.iter_mut().zip(depths) {
            record[6] = depth.to_bits();
        }
        for count in 1..=4u32 {
            for (kind, fixed, length) in [
                (0u32, 0u32, 0x1000u32),
                (4, 0x1234_5678, 0x1000),
                (3, 0xFF, 5),
                (5, 0, 1),
                (4, 0xFFFF_FF00, 2),
                (0, 0, 0),
                (4, 0x55, 0),
            ] {
                check_state(
                    &mut t,
                    &format!(
                        "depths {depths:?}, {count} vertices, kind {kind}, fixed {fixed:#x}, length {length:#x}"
                    ),
                    call_original,
                    Regs::SENTINEL,
                    &|| unsafe {
                        restore(&regions, &vec![0xAA; saved.0.len()]);
                        for (n, record) in records.iter().enumerate() {
                            let number = [5u32, 2, 0x7F, 0][n];
                            game_write(LIST_RVA + n * 4, number);
                            let entry = game_addr(ENTRIES_RVA) + number as usize * ENTRY_LEN;
                            put(entry + 4, record.as_ptr() as u32);
                        }
                        std::ptr::copy_nonoverlapping(
                            table.as_ptr(),
                            game_addr(TABLE_RVA) as *mut u8,
                            table.len(),
                        );
                        game_write(COUNT_RVA, count);
                        game_write(LENGTH_RVA, length);
                        game_write(KIND_RVA, kind);
                        game_write(FIXED_RVA, fixed);
                    },
                    &regions,
                    &rust,
                );
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
