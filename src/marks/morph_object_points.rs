//! `marks::morph_object_points` (0x4D08DD, reached through the jumps at 0x4D074D and
//! 0x4D0815): for each car (the settings' +0x5C, the last first; its block at +0x29C of
//! its 0x4C8 bytes) the block's +0xC its +8; when its eight factors (+0x90 ..) differ from
//! the kept ones (+0xB0 ..): +0xC -1, the factors kept (also in `dword_93C6D4` ..), and
//! for each of its first object's (+0) points the matching point of the second object
//! (+4) blended toward from the base list (+0x8C) by `marks::blend_corner`, where the
//! base and the second object's point differ. Then `marks::measure_lines`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4D14B4`. Refuses (before anything) null settings, more than 8 cars, and
//! for a car whose factors changed: an object past 0xFF, a null base list, points past
//! the object points' room, a point count below 0 (the loop runs on); and
//! `measure_lines`' refusals (all it may write is put back first). Self-test: the game's
//! settings (skipped without them or without object points), factors changed or not by
//! lot, two objects' points of the game's; compared on all it may write.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::marks::blend_corner::blend_corner;
use crate::marks::lines::{SETTINGS_RVA, measure_lines};
use crate::marks::store::{Slot, game_read, game_write};
use crate::scene::objects::{
    FIRST_POINT_RVA, POINT_COUNTS_RVA, POINTS_ROOM_RVA, POINTS_RVA, SLOTS,
};

const CAR_LEN: u32 = 0x4C8;
const BLOCK: u32 = 0x29C;
const MOST_CARS: i32 = 8;
const KEPT_RVA: Slot = Slot::at(0x93C6D4); // dword_93C6D4 .. dword_93C6F0

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The cars' blocks, the last first (none for a count of 0 or less).
unsafe fn blocks(settings: u32) -> Vec<u32> {
    let count = unsafe { get(settings + 0x5C) } as i32;
    (0..count.clamp(0, MOST_CARS) as u32)
        .rev()
        .map(|n| settings + n * CAR_LEN + BLOCK)
        .collect()
}

/// Whether a block's factors differ from its kept ones.
unsafe fn changed(b: u32) -> bool {
    (0..8).any(|k| unsafe { get(b + 0x90 + 4 * k) != get(b + 0xB0 + 4 * k) })
}

/// A block's objects' first points (in `g_objectPoints`) and their count.
unsafe fn points(b: u32) -> (u32, u32, i32) {
    unsafe {
        let base: u32 = game_read(POINTS_RVA);
        let first =
            |id: u32| base.wrapping_add(12 * game_read::<u32>(FIRST_POINT_RVA + 4 * id as usize));
        let (a, c) = (get(b), get(b + 4));
        (
            first(a),
            first(c),
            game_read(POINT_COUNTS_RVA + 4 * a as usize),
        )
    }
}

/// Why the Rust version refuses, if it does (its own checks), worked out without writing.
unsafe fn refusal(settings: u32) -> Option<&'static str> {
    unsafe {
        if settings == 0 {
            return Some("no marks settings");
        }
        if get(settings + 0x5C) as i32 > MOST_CARS {
            return Some("more than 8 cars");
        }
        let base: u32 = game_read(POINTS_RVA);
        let room: u32 = game_read(POINTS_ROOM_RVA);
        for b in blocks(settings) {
            if !changed(b) {
                continue;
            }
            if get(b) >= SLOTS || get(b + 4) >= SLOTS {
                return Some("an object past 0xFF");
            }
            if get(b + 0x8C) == 0 {
                return Some("a null base point list");
            }
            let (a, c, n) = points(b);
            if n < 0 {
                return Some("a negative point count: the loop runs on");
            }
            let end = base.wrapping_add(room);
            for p in [a, c] {
                if p < base || p.wrapping_add(12 * n as u32) > end {
                    return Some("points past the object points' room");
                }
            }
        }
        None
    }
}

/// The Rust version (see the header).
pub unsafe fn morph_object_points() -> Result<(), Refused> {
    let settings: u32 = unsafe { game_read(SETTINGS_RVA) };
    if let Some(why) = unsafe { refusal(settings) } {
        return Err(BadInput(why).into());
    }
    unsafe {
        for b in blocks(settings) {
            put(b + 0xC, get(b + 8));
            if (0..8).all(|k| get(b + 0x90 + 4 * k) == get(b + 0xB0 + 4 * k)) {
                continue;
            }
            put(b + 0xC, 0xFFFF_FFFF);
            for k in 0..8 {
                let v = get(b + 0x90 + 4 * k);
                put(b + 0xB0 + 4 * k, v);
                game_write(KEPT_RVA + 4 * k as usize, v);
            }
            let (a, c, n) = points(b);
            let base = get(b + 0x8C);
            for k in 0..n.max(0) as u32 {
                let (to, from, base) = (a + 12 * k, c + 12 * k, base + 12 * k);
                if (0..3).any(|i| get(base + 4 * i) != get(from + 4 * i)) {
                    blend_corner(b, to, from, base);
                }
            }
        }
        measure_lines()?;
    }
    Ok(())
}
