//! `scene::set_object_fade` (0x451609): how see-through an object is drawn by how far
//! off it is, and whether another, simpler object is drawn in its place.
//!
//! In:  nothing
//! Out: the original leaves eax, ebx, ecx, edx and esi changed; its caller goes on
//!      without them, and the Rust version keeps them
//!
//! From the record of the object's kind (0x80 bytes at `unk_97DF30`) two words: +0x1D
//! to `qword_6214E0` (another kind to draw instead) and +0x22 to its second half (a
//! distance). `dword_635004` and `dword_63500C` = 0, `dword_63996C` = 0 (no fading),
//! `dword_639974` = 0x100 (not see-through). Nothing more if both words are 0 or
//! the object's place `dword_634FFC` (a point of the scene) is -1. Else that point's
//! record (made if it is not there: `sub_44CF33`, the same as
//! `scene::project_point` makes one) gives `dword_635004` and `dword_63500C`: its x
//! and depth in the view as whole numbers. With the depth beyond the distance times
//! 65536: `dword_639974` and `dword_639978` = (the distance times 65536 less the
//! depth, plus 0x100000, not under 0) over 0x1000, `dword_63996C` = 1, and if the
//! first word is not 0 and an object of that kind exists (`sub_422298`: the first of
//! the objects 1 .. 0xFF with that kind), `g_objectNumber` becomes that object.
//! One caller, `sub_45045D`.
//! Refuses an object's number past the tables and, where a record has to be made,
//! no records or no points.
//! Shadow mode: the Rust version, its writes put back, the original, compared.

use crate::errors::BadInput;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::polygon::dword;
use crate::scene::project_point::{
    POINTS_RVA, RECORD_LEN, RECORDS_RVA, World, inputs as view_inputs, make_scene_record,
};
use crate::scene::set_object_material::{MATERIAL_LEN, MATERIALS_RVA};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_object_fade", rva(0x451609), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const OBJECTS: u32 = 0x100;
const KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
const WORDS_RVA: usize = rva(0x6214E0); // qword_6214E0
const PLACE_RVA: usize = rva(0x634FFC); // dword_634FFC
const PLACE_X_RVA: usize = rva(0x635004); // dword_635004
const PLACE_DEPTH_RVA: usize = rva(0x63500C); // dword_63500C
const FADING_RVA: usize = rva(0x63996C); // dword_63996C
const FADE_RVA: usize = rva(0x639974); // dword_639974, dword_639978

// ---- Rust version ----

unsafe fn record_at() -> Option<u32> {
    unsafe {
        let (place, records): (u32, u32) = (game_read(PLACE_RVA), game_read(RECORDS_RVA));
        (records != 0).then(|| records.wrapping_add((place & 0xFFF) << 5))
    }
}

/// The two words of the object's kind, and whether the object's place is looked at;
/// None for an object's number past the tables.
unsafe fn words() -> Option<(u32, u32, bool)> {
    unsafe {
        let object: u32 = game_read(OBJECT_RVA);
        if object >= OBJECTS {
            return None;
        }
        let kind: u32 = game_read(KINDS_RVA + object as usize * 4);
        let material =
            game_addr(MATERIALS_RVA).wrapping_add((kind as usize).wrapping_mul(MATERIAL_LEN));
        let word = |at: usize| ((material + at) as *const u16).read_unaligned() as u32;
        let (other_kind, distance) = (word(0x1D), word(0x22));
        let looked_at = (other_kind | distance) != 0 && game_read::<u32>(PLACE_RVA) != u32::MAX;
        Some((other_kind, distance, looked_at))
    }
}

pub unsafe fn set_object_fade() -> Result<(), BadInput> {
    unsafe {
        let Some((other_kind, distance, needs_record)) = words() else {
            return Err(BadInput("an object's number past the tables"));
        };
        let place: u32 = game_read(PLACE_RVA);
        // What can refuse is asked before anything is written.
        let record = match (needs_record, record_at()) {
            (false, _) => 0,
            (true, None) => return Err(BadInput("no records for the points")),
            (true, Some(record)) => {
                if dword(record as usize) != place && game_read::<u32>(POINTS_RVA) == 0 {
                    return Err(BadInput("no points"));
                }
                record
            }
        };
        game_write(WORDS_RVA, [other_kind, distance]);
        game_write(PLACE_X_RVA, 0u32);
        game_write(PLACE_DEPTH_RVA, 0u32);
        game_write(FADING_RVA, 0u32);
        game_write(FADE_RVA, 0x100u32);
        if !needs_record {
            return Ok(());
        }
        if dword(record as usize) != place {
            make_scene_record(place, record)?;
        }
        game_write(PLACE_X_RVA, x87::to_int(dword(record as usize + 0x10)));
        let depth = x87::to_int(dword(record as usize + 0x18));
        game_write(PLACE_DEPTH_RVA, depth);
        let near = ((distance << 16) as i32).wrapping_sub(depth);
        if near >= 0 {
            return Ok(());
        }
        let fade = (near.wrapping_add(0x10_0000).max(0) as u32) >> 12;
        game_write(FADE_RVA, [fade, fade]);
        game_write(FADING_RVA, 1u32);
        if other_kind != 0 {
            let kinds: [u32; 0x100] = game_read(KINDS_RVA);
            if let Some(other) = (1..0x100).find(|&n| kinds[n] == other_kind) {
                game_write(OBJECT_RVA, other as u32);
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![
        (game_addr(WORDS_RVA) as u32, 8),
        at(PLACE_X_RVA),
        at(PLACE_DEPTH_RVA),
        at(FADING_RVA),
        (game_addr(FADE_RVA) as u32, 8),
        at(OBJECT_RVA),
        at(CHECKED_RVA),
    ];
    // The record only when the function looks at it: a pointer is no proof that its
    // memory is there.
    if let (Some((_, _, true)), Some(record)) = (unsafe { words() }, unsafe { record_at() }) {
        regions.push((record, RECORD_LEN));
    }
    regions
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if !shadow() {
        if let Err(e) = unsafe { set_object_fade() } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let regions = regions();
    let before = capture(&regions);
    let mine = unsafe { set_object_fade() }.map(|()| capture(&regions));
    restore(&regions, &before);
    unsafe { call_original(r) };
    let Ok(mine) = mine else {
        return;
    };
    let theirs = capture(&regions);
    if mine != theirs {
        HOOK.mismatch(|| format!("before {before:x?}; original {theirs:x?}; rust {mine:x?}"));
    }
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    const OBJECT: u32 = 0xF1;
    const OTHER: u32 = 0xF3;
    // The test's object, an unused one, is given the kind of the first real object for
    // the time of the test: that kind's record is there to be read and changed.
    let kinds: [u32; 0x100] = unsafe { game_read(KINDS_RVA) };
    // The records run from `unk_97DF30` up to `g_faces`: room for 0x4B0 kinds.
    let Some(&kind) = kinds[1..].iter().find(|&&kind| kind < 0x4B0) else {
        return t.skip(&format!(
            "no object is loaded whose kind's record the test could use (the first kinds: {:x?})",
            &kinds[..4]
        ));
    };
    let material = game_addr(MATERIALS_RVA) + kind as usize * MATERIAL_LEN;
    let mut inputs = view_inputs();
    inputs.extend(
        [
            PLACE_RVA,
            KINDS_RVA + OTHER as usize * 4,
            KINDS_RVA + OBJECT as usize * 4,
        ]
        .map(at),
    );
    inputs.push(((material + 0x1D) as u32, 2));
    inputs.push(((material + 0x22) as u32, 2));
    inputs.extend(regions().into_iter().take(7));
    let saved = capture(&inputs);
    unsafe { game_write(KINDS_RVA + OBJECT as usize * 4, kind) };
    let mut world = World::new();
    for view in 0..4 {
        for point in [0u32, 2, 5, 7, u32::MAX] {
            for (other_kind, distance) in [
                (0u16, 0u16),
                (0, 1),
                (0, 0xFFFF),
                (0x7E57, 0),
                (0x7E57, 3),
                (0x7E58, 0x80),
            ] {
                let run = |world: &mut World, rust: bool| unsafe {
                    world.set(view, false, view == 3);
                    game_write(OBJECT_RVA, OBJECT);
                    game_write(KINDS_RVA + OTHER as usize * 4, 0x7E57u32);
                    game_write(PLACE_RVA, point);
                    ((material + 0x1D) as *mut u16).write_unaligned(other_kind);
                    ((material + 0x22) as *mut u16).write_unaligned(distance);
                    let written: Vec<Region> = regions().into_iter().take(7).collect();
                    let blotted: Vec<Region> = written
                        .iter()
                        .copied()
                        .filter(|&(at, _)| at != game_addr(OBJECT_RVA) as u32)
                        .collect();
                    restore(&blotted, &vec![0xAA; blotted.iter().map(|r| r.1).sum()]);
                    let done = if rust {
                        set_object_fade().is_ok()
                    } else {
                        call_original(&mut Regs::default());
                        true
                    };
                    (done, capture(&written), world.state())
                };
                let (theirs, mine) = (run(&mut world, false), run(&mut world, true));
                t.check(theirs == mine, || {
                    format!(
                        "view {view}, place {point:#x}, other kind {other_kind:#x}, distance {distance:#x}: original {:x?}; rust done {} {:x?}; records the same {}",
                        theirs.1,
                        mine.0,
                        mine.1,
                        theirs.2 == mine.2
                    )
                });
            }
        }
    }
    restore(&inputs, &saved);
    t.finish()
}
