//! `scene::draw_object_polygon` (0x451F69): draws a polygon of an object, as it is if
//! it lies on the screen and as triangles if it has to be clipped.
//!
//! In:  nothing; the polygon's entries are the work table's first, in order
//! Out: the original leaves every register changed; the Rust version keeps them
//!
//! The pool of records, the list (0, 1, 2 .. and a 0) and the next free entry are set
//! as `scene::draw_clipped_polygon` sets them; `scene::project_polygon`; then
//! `sub_451EED` says whether a polygon of three vertices or more has one that is
//! nearer than `flt_5C5D90`, or left of or above that same number, or at or past the
//! screen's right or bottom edge. If none, or with exactly three vertices:
//! `dword_639960` + 1 and `scene::draw_clipped_polygon`. Else `dword_639964` + 1, the
//! entries are kept at `unk_976610`, and the polygon is drawn as a fan of triangles:
//! the first three entries as they are, then for n = 3 .. the entries 0, n - 1 and n
//! from the kept ones (`sub_45204A` copies one), each with the number of vertices at
//! 3, where it stays.
//! Three call sites, all in `sub_450BDA`.
//! Refuses a polygon of no vertices or of more than 64 (the kept entries' room).
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose calls of the projection and the drawing note
//! what they are given.
//!
//! Converted 2026-10-21: the test, the list, the choice and the fan through
//! `rc2k_core::scene::polygons` (compares of stored floats: exact).

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::clip::{NEXT_ENTRY_RVA, NEXT_RECORD_RVA, POOL_RVA};
use crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game;
use crate::scene::polygon::{ENTRIES_RVA, ENTRY_LEN, LIST_RVA, dword, put};
use crate::scene::project_polygon::project_polygon;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::draw_object_polygon",
    rva(0x451F69),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const KEPT_RVA: usize = rva(0x976610); // unk_976610
const LEAST_RVA: usize = rva(0x5C5D90); // flt_5C5D90
const RIGHT_RVA: usize = rva(0x6241F0); // g_clipScreenRight
const BOTTOM_RVA: usize = rva(0x6241E8); // g_clipScreenBottom
const WHOLE_COUNT_RVA: usize = rva(0x639960); // dword_639960
const SPLIT_COUNT_RVA: usize = rva(0x639964); // dword_639964

// ---- Rust version ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Project,
    Draw,
}

/// `sub_451EED` on the work table's records.
unsafe fn needs_clipping(count: u32) -> Result<bool, BadInput> {
    use rc2k_core::scene::polygons::{Edges, Vertex};
    unsafe {
        if (count as i32).wrapping_sub(3) < 0 {
            return Ok(false);
        }
        let number = |at: usize| f32::from_bits(game_read(at));
        let edges = Edges {
            least: number(LEAST_RVA),
            right: number(RIGHT_RVA),
            bottom: number(BOTTOM_RVA),
        };
        rc2k_core::scene::polygons::needs_clipping(count, &edges, |n| {
            let record = dword(game_addr(ENTRIES_RVA) + n * ENTRY_LEN + 4) as usize;
            if record == 0 {
                return Err(BadInput("an entry without its record"));
            }
            let value = |at: usize| f32::from_bits(dword(record + at));
            Ok(Vertex {
                x: value(0xC),
                y: value(0x1C),
                depth: value(0x18),
            })
        })
    }
}

// Converted: `rc2k_core::scene::polygons`
pub unsafe fn draw_object_polygon(
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    use rc2k_core::scene::polygons::{drawn_whole, fan, first_list, fits};
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if !fits(count) {
            return Err(BadInput("a polygon of no vertices, or of more than 64").into());
        }
        game_write(NEXT_RECORD_RVA, game_addr(POOL_RVA) as u32);
        for (n, number) in first_list(count).into_iter().enumerate() {
            game_write(LIST_RVA + n * 4, number);
        }
        game_write(NEXT_ENTRY_RVA, count);
        step(Step::Project)?;
        if drawn_whole(count, needs_clipping(count)?) {
            game_write(
                WHOLE_COUNT_RVA,
                game_read::<u32>(WHOLE_COUNT_RVA).wrapping_add(1),
            );
            return step(Step::Draw);
        }
        game_write(
            SPLIT_COUNT_RVA,
            game_read::<u32>(SPLIT_COUNT_RVA).wrapping_add(1),
        );
        let entries = game_addr(ENTRIES_RVA);
        let kept = game_addr(KEPT_RVA);
        std::ptr::copy(
            entries as *const u8,
            kept as *mut u8,
            count as usize * ENTRY_LEN,
        );
        game_write(COUNT_RVA, 3u32);
        step(Step::Draw)?;
        game_write(COUNT_RVA, count);
        for triangle in fan(count) {
            for (to, from) in triangle.into_iter().enumerate() {
                std::ptr::copy(
                    (kept + from * ENTRY_LEN) as *const u8,
                    (entries + to * ENTRY_LEN) as *mut u8,
                    ENTRY_LEN,
                );
            }
            game_write(COUNT_RVA, 3u32);
            step(Step::Draw)?;
        }
        Ok(())
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    let done = unsafe {
        draw_object_polygon(&mut |step| match step {
            Step::Project => project_polygon().map(|_| ()),
            Step::Draw => draw_clipped_polygon_in_game(&entry),
        })
    };
    if let Err(e) = done {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0xE1;
const ENTRY_BYTES: [u8; 12] = [
    0x8D, 0x35, 0x10, 0x7F, 0x97, 0x00, 0x89, 0x35, 0xA4, 0x40, 0x62, 0x00,
];
const CALLS: [usize; 8] = [0x2F, 0x34, 0x52, 0x85, 0xA8, 0xB8, 0xC6, 0xD5];

/// Each call: its place, the number of vertices, the first three entries' points.
static CALLED: Mutex<Vec<(u32, u32, [u32; 3])>> = Mutex::new(Vec::new());

fn handler(place: u32, _: &mut Regs) {
    unsafe {
        let entries = game_addr(ENTRIES_RVA);
        let points = [0, 1, 2].map(|n| dword(entries + n * ENTRY_LEN));
        CALLED
            .lock()
            .unwrap()
            .push((place, game_read(COUNT_RVA), points));
        if place == 1 {
            // A drawing changes the entries it draws, as the real one does.
            for n in 0..3 {
                put(entries + n * ENTRY_LEN + 8, 0xD0D0_0000 + n as u32);
            }
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x451F69)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                if callee == rva(0x44E1F7) {
                    Some(stand_in(0))
                } else if callee == rva(0x44CCEF) {
                    Some(stand_in(1))
                } else {
                    None
                }
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    const MOST: usize = 6;
    let mut regions: Vec<Region> = [
        COUNT_RVA,
        NEXT_RECORD_RVA,
        NEXT_ENTRY_RVA,
        WHOLE_COUNT_RVA,
        SPLIT_COUNT_RVA,
    ]
    .map(at)
    .to_vec();
    regions.push((game_addr(LIST_RVA) as u32, (MOST + 1) * 4));
    regions.push((game_addr(ENTRIES_RVA) as u32, MOST * ENTRY_LEN));
    regions.push((game_addr(KEPT_RVA) as u32, MOST * ENTRY_LEN));
    let inputs = [at(RIGHT_RVA), at(BOTTOM_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    let least = f32::from_bits(unsafe { game_read(LEAST_RVA) });
    // (x, y, depth) of the one vertex that is not in the middle of the screen.
    let odd: [(f32, f32, f32); 9] = [
        (400.0, 300.0, 9000.0),
        (least - 1.0, 300.0, 9000.0),
        (least, 300.0, 9000.0),
        (800.0, 300.0, 9000.0),
        (799.5, 599.5, 9000.0),
        (400.0, least - 0.5, 9000.0),
        (400.0, 600.0, 9000.0),
        (400.0, 300.0, least - 1.0),
        (f32::NAN, 300.0, 9000.0),
    ];
    for count in [1usize, 2, 3, 4, 6] {
        for (case, odd) in odd.into_iter().enumerate() {
            let mut records = [[0u32; 8]; MOST];
            for (n, record) in records.iter_mut().enumerate() {
                let (x, y, depth) = if n == case % count {
                    odd
                } else {
                    (400.0, 300.0, 9000.0)
                };
                (record[3], record[7], record[6]) = (x.to_bits(), y.to_bits(), depth.to_bits());
            }
            let run = |rust: bool| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                for (n, record) in records.iter().enumerate() {
                    let entry = game_addr(ENTRIES_RVA) + n * ENTRY_LEN;
                    put(entry, 100 + n as u32);
                    put(entry + 4, record.as_ptr() as u32);
                }
                game_write(COUNT_RVA, count as u32);
                game_write(RIGHT_RVA, 800f32.to_bits());
                game_write(BOTTOM_RVA, 600f32.to_bits());
                CALLED.lock().unwrap().clear();
                set_handler(handler);
                let done = if rust {
                    draw_object_polygon(&mut |step| {
                        act(step as u32, &mut Regs::default());
                        Ok(())
                    })
                    .is_ok()
                } else {
                    call_game(&mut Regs::default(), copy.addr());
                    true
                };
                (done, capture(&regions), CALLED.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.2.len() >= 2, || {
                format!(
                    "{count} vertices, case {case}: original's calls {:x?}; rust done {} calls {:x?}; first different byte {:?}",
                    theirs.2,
                    mine.0,
                    mine.2,
                    theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b)
                )
            });
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
