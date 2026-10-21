//! `scene::project_polygon` (0x44E1F7): finds or makes the record of each of the
//! polygon's points and counts those outside the depths that can be drawn.
//!
//! In:  nothing
//! Out: eax, ecx, edx, esi as the last point's work leaves them (eax and edx as
//!      `scene::project_point`, ecx = the number of vertices, esi = the last entry);
//!      everything else preserved
//!
//! For each entry of the polygon: its point (+0) through `scene::project_point`, the
//! record's address to its +4; `dword_6240F4`, 0 at the start, counts the points whose
//! depth is under 4096 (or is no number) or is `flt_6241C8` or more. The callers
//! compare the count with the number of vertices (all outside: nothing to draw) and
//! with 0 (none: no clipping by depth).
//! Eight callers. A polygon of no vertices is refused (the original goes through 2^32).
//! Self-test: polygons of the test's on points, records and views of the test's.
//!
//! Converted 2026-10-21: the hook's Rust version is `project_polygon_plain`, on
//! `scene::project_point_plain` and `rc2k_core::scene::projection::outside_depths` (the x87
//! version stays for the Rust callers in unconverted code). Shadow mode and the self-test
//! compare the records' floats and `dword_62400C` within `service::floats`' tolerances
//! (the self-test at both test precisions, exact at double); the count decides on the
//! stored depths, as the original.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, ECX, EDX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::{ENTRIES, ENTRIES_RVA, ENTRY_LEN, LIST_RVA, dword, listed, put};
use crate::scene::project_point::{
    NEAR, POINTS_RVA, RECORD_LEN, RECORDS_RVA, World, inputs, project_point, project_point_plain,
    record_offset, same_world,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::project_polygon", rva(0x44E1F7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const OUTSIDE_RVA: usize = rva(0x6240F4); // dword_6240F4
const FAR_RVA: usize = rva(0x6241C8); // flt_6241C8

// ---- Rust version ----

pub unsafe fn project_polygon() -> Result<Left, Refused> {
    unsafe {
        let entries = listed()?;
        let Some(&last) = entries.last() else {
            return Err(BadInput("a polygon of no vertices").into());
        };
        // Nothing may be written before a refusal: every point is looked at first.
        // The scene's points are only needed for a scene's point: the menus have none,
        // and their objects' points (negative) only need the records.
        let scene_point = entries.iter().any(|&entry| dword(entry) as i32 >= 0);
        if game_read::<u32>(RECORDS_RVA) == 0 || (scene_point && game_read::<u32>(POINTS_RVA) == 0)
        {
            return Err(BadInput("no points or no records for them").into());
        }
        for &entry in &entries {
            record_offset(dword(entry))?;
        }
        let far = f32::from_bits(game_read(FAR_RVA));
        let mut outside = 0u32;
        let mut done = None;
        for &entry in &entries {
            let point = project_point(dword(entry))?;
            put(entry + 4, point.record);
            let depth = f32::from_bits(dword(point.record as usize + 0x18));
            // Two compares, each taking "no number" for "below".
            let near = depth.partial_cmp(&NEAR).is_none_or(|order| order.is_lt());
            let in_reach = depth.partial_cmp(&far).is_none_or(|order| order.is_lt());
            if near || !in_reach {
                outside += 1;
            }
            done = Some(point);
        }
        game_write(OUTSIDE_RVA, outside);
        let done = done.expect("the polygon has a vertex");
        Ok(Left::nothing()
            .with(EAX, done.eax)
            .with(ECX, entries.len() as u32)
            .with(EDX, done.record)
            .with(ESI, last as u32))
    }
}

/// The converted `project_polygon` (plain floats): the hook's. The x87 version above stays
/// for the Rust callers in unconverted code.
// Converted: `rc2k_core::scene::projection::outside_depths`
pub unsafe fn project_polygon_plain() -> Result<Left, Refused> {
    unsafe {
        let entries = listed()?;
        let Some(&last) = entries.last() else {
            return Err(BadInput("a polygon of no vertices").into());
        };
        let scene_point = entries.iter().any(|&entry| dword(entry) as i32 >= 0);
        if game_read::<u32>(RECORDS_RVA) == 0 || (scene_point && game_read::<u32>(POINTS_RVA) == 0)
        {
            return Err(BadInput("no points or no records for them").into());
        }
        for &entry in &entries {
            record_offset(dword(entry))?;
        }
        let far = f32::from_bits(game_read(FAR_RVA));
        let mut outside = 0u32;
        let mut done = None;
        for &entry in &entries {
            let point = project_point_plain(dword(entry))?;
            put(entry + 4, point.record);
            let depth = f32::from_bits(dword(point.record as usize + 0x18));
            if rc2k_core::scene::projection::outside_depths(depth, far) {
                outside += 1;
            }
            done = Some(point);
        }
        game_write(OUTSIDE_RVA, outside);
        let done = done.expect("the polygon has a vertex");
        Ok(Left::nothing()
            .with(EAX, done.eax)
            .with(ECX, entries.len() as u32)
            .with(EDX, done.record)
            .with(ESI, last as u32))
    }
}

// ---- hook ----

/// What the call may write: the count, each entry's record pointer and each record.
fn regions() -> Vec<Region> {
    let mut regions = vec![at(OUTSIDE_RVA), at(CHECKED_RVA)];
    let records: u32 = unsafe { game_read(RECORDS_RVA) };
    for entry in unsafe { listed() }.unwrap_or_default() {
        regions.push((entry as u32 + 4, 4));
        if let (Ok(offset), true) = (unsafe { record_offset(dword(entry)) }, records != 0) {
            regions.push((records.wrapping_add(offset), RECORD_LEN));
        }
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { project_polygon_plain() }
}

/// The float places of `regions()`: `dword_62400C` and each record's five floats.
fn float_places() -> Vec<(Region, crate::service::floats::Tolerance)> {
    use crate::service::floats::FLOAT;
    let mut places = vec![(at(CHECKED_RVA), FLOAT)];
    let records: u32 = unsafe { game_read(RECORDS_RVA) };
    for entry in unsafe { listed() }.unwrap_or_default() {
        if let (Ok(offset), true) = (unsafe { record_offset(dword(entry)) }, records != 0) {
            places.push(((records.wrapping_add(offset + 0xC), 0x14), FLOAT));
        }
    }
    places
}

extern "C" fn dispatch(r: &mut Regs) {
    use crate::service::floats::{Floats, dispatch_state_floats};
    let places = float_places();
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions(), &floats, &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut inputs = inputs();
    inputs.extend([at(COUNT_RVA), at(FAR_RVA), at(OUTSIDE_RVA)]);
    let table = [(game_addr(ENTRIES_RVA) as u32, ENTRIES as usize * ENTRY_LEN)];
    let list = [(game_addr(LIST_RVA) as u32, 8 * 4)];
    let saved = (capture(&inputs), capture(&table), capture(&list));
    let mut world = World::new();
    let polygons: [&[i32]; 6] = [
        &[0, 1, 3, 7],
        &[5, 6, 71],
        &[2, 2, 0],
        &[-1, -2, -3, -4, -5],
        &[4, -2, 2, 6, 1, -5],
        &[3],
    ];
    let places = [(at(CHECKED_RVA), crate::service::floats::FLOAT)];
    let tolerant = crate::service::floats::Floats {
        memory: &places,
        regs: &[],
    };
    crate::service::floats::at_test_precisions(&tolerant, &mut |word, held| {
        let tolerance = held.memory[0].1;
        for (n, points) in polygons.into_iter().enumerate() {
            for view in 0..4 {
                for far in [0.0f32, 9000.0, 1e9, f32::NAN] {
                    let (flat, lowered) = (n % 2 == 1, view % 2 == 1);
                    let before = Regs::SENTINEL;
                    let set = |world: &mut World| unsafe {
                        world.set(view, flat, lowered);
                        restore(&table, &vec![0xAA; saved.1.len()]);
                        for (k, &point) in points.iter().enumerate() {
                            // The list picks the entries out of order.
                            let number = [5u32, 2, 9, 0, 0x7F, 3][k];
                            game_write(LIST_RVA + k * 4, number);
                            put(
                                game_addr(ENTRIES_RVA) + number as usize * ENTRY_LEN,
                                point as u32,
                            );
                        }
                        game_write(COUNT_RVA, points.len() as u32);
                        game_write(FAR_RVA, far.to_bits());
                        game_write(OUTSIDE_RVA, 0xAAAA_AAAAu32);
                    };
                    let state = |world: &World| {
                        (world.state(), capture(&table), unsafe {
                            game_read::<u32>(OUTSIDE_RVA)
                        })
                    };
                    set(&mut world);
                    let mut ro = before;
                    unsafe { call_original(&mut ro) };
                    let theirs = state(&world);
                    set(&mut world);
                    let mine = rust(&before);
                    let left = state(&world);
                    let regs_ok = mine.as_ref().is_ok_and(|left| left.matches(&before, &ro));
                    t.check(
                        regs_ok
                            && same_world(&theirs.0, &left.0, tolerance)
                            && theirs.1 == left.1
                            && theirs.2 == left.2,
                        || {
                            format!(
                                "control word {word:#x}, points {points:?}, view {view}, far {far}: regs_ok={regs_ok} original eax {:#x} ecx {:#x} edx {:#x} esi {:#x}, rust {mine:x?}; outside {:#x} {:#x}; records same {}, entries same {}",
                                ro.eax,
                                ro.ecx,
                                ro.edx,
                                ro.esi,
                                theirs.2,
                                left.2,
                                theirs.0 == left.0,
                                theirs.1 == left.1,
                            )
                        },
                    );
                }
            }
        }
    });
    restore(&inputs, &saved.0);
    restore(&table, &saved.1);
    restore(&list, &saved.2);
    t.finish()
}
