//! `scene::draw_clipped_polygon` (0x44CCEF): a polygon's whole way from its points to
//! the screen.
//!
//! In:  nothing; the polygon's entries are the work table's first, in order
//! Out: the original leaves every register changed; the Rust version keeps them
//!
//! `dword_624120`, `dword_624124`, `dword_624134` = 0; the pool of records for new
//! vertices (`dword_6240A4`) = `unk_977F10`; the list = 0, 1, 2 .. and a 0 after it;
//! the next free entry (`dword_624114`) = the number of vertices. Then
//! `scene::project_polygon`; nothing more if every point is outside the depths;
//! `scene::scale_texture_coordinates`; `scene::fog_polygon`; if some point is outside,
//! `scene::clip_polygon_by_depth`, and nothing more with fewer than three vertices
//! left; `scene::clip_polygon_to_screen`, and the same; `scene::draw_marked_polygon`;
//! `dword_624134` = 1 (the polygon was drawn).
//! Four callers (`sub_451F69` and three more: the objects' polygons).
//! Refuses a polygon of no vertices (the original goes through 2^32) or of more than
//! 256, and what its parts refuse. A refusal after the texture coordinates were
//! scaled hands the original a polygon it scales once more.
//! Shadow mode runs the original alone (its parts are compared in their own hooks).
//! Self-test: a redirected copy whose six calls follow a script.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::clip::{
    DEPTH_CLIPPED_RVA, NEXT_ENTRY_RVA, NEXT_RECORD_RVA, POOL_RVA, SCREEN_CLIPPED_RVA,
};
use crate::scene::clip_polygon_by_depth::clip_polygon_by_depth;
use crate::scene::clip_polygon_to_screen::clip_polygon_to_screen;
use crate::scene::draw_marked_polygon::draw_marked_polygon;
use crate::scene::draw_polygon::{Game, draw_polygon};
use crate::scene::fog_polygon::fog_polygon;
use crate::scene::polygon::{LIST_RVA, MOST};
use crate::scene::project_polygon::{OUTSIDE_RVA, project_polygon};
use crate::scene::scale_texture_coordinates::scale_texture_coordinates;
use crate::scene::submit_polygon;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::draw_clipped_polygon",
    rva(0x44CCEF),
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

const DRAWN_RVA: usize = rva(0x624134); // dword_624134

// ---- Rust version ----

/// The function's six calls, in the order they can come.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Project,
    ScaleTextureCoordinates,
    Fog,
    ClipByDepth,
    ClipToScreen,
    Draw,
}

pub unsafe fn draw_clipped_polygon(
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let count: u32 = game_read(COUNT_RVA);
        if count == 0 || count > MOST {
            return Err(BadInput("a polygon of no vertices, or of more than 256").into());
        }
        game_write(SCREEN_CLIPPED_RVA, 0u32);
        game_write(DEPTH_CLIPPED_RVA, 0u32);
        game_write(DRAWN_RVA, 0u32);
        game_write(NEXT_RECORD_RVA, game_addr(POOL_RVA) as u32);
        for n in 0..count {
            game_write(LIST_RVA + n as usize * 4, n);
        }
        game_write(LIST_RVA + count as usize * 4, 0u32);
        game_write(NEXT_ENTRY_RVA, count);
        step(Step::Project)?;
        let outside: u32 = game_read(OUTSIDE_RVA);
        if outside == game_read::<u32>(COUNT_RVA) {
            return Ok(());
        }
        step(Step::ScaleTextureCoordinates)?;
        step(Step::Fog)?;
        if game_read::<u32>(OUTSIDE_RVA) != 0 {
            step(Step::ClipByDepth)?;
            if game_read::<u32>(COUNT_RVA) < 3 {
                return Ok(());
            }
        }
        step(Step::ClipToScreen)?;
        if game_read::<u32>(COUNT_RVA) < 3 {
            return Ok(());
        }
        step(Step::Draw)?;
        game_write(DRAWN_RVA, 1u32);
        Ok(())
    }
}

// ---- hook ----

/// The function with its parts' Rust versions, drawing for real. `entry`: the registers
/// the Direct3D layer is called with.
pub unsafe fn draw_clipped_polygon_in_game(entry: &Regs) -> Result<(), Refused> {
    unsafe {
        draw_clipped_polygon(&mut |step| {
            match step {
                Step::Project => project_polygon().map(|_| ())?,
                Step::ScaleTextureCoordinates => scale_texture_coordinates().map(|_| ())?,
                Step::Fog => fog_polygon().map(|_| ())?,
                Step::ClipByDepth => clip_polygon_by_depth()?,
                Step::ClipToScreen => clip_polygon_to_screen()?,
                Step::Draw => draw_marked_polygon(&mut || {
                    draw_polygon(&mut Game(submit_polygon::Game(entry)))
                })?,
            }
            Ok(())
        })
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { draw_clipped_polygon_in_game(&entry) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0xA1;
const ENTRY_BYTES: [u8; 10] = [0xC7, 0x05, 0x20, 0x41, 0x62, 0x00, 0, 0, 0, 0];
const CALLS: [usize; 6] = [0x4D, 0x5F, 0x64, 0x72, 0x80, 0x8E];
/// In `Step`'s order; the third is the stub in front of `scene::fog_polygon`.
const CALLEES: [usize; 6] = [0x44E1F7, 0x44E2FB, 0x44E25A, 0x44DB4F, 0x44D15A, 0x44B9FC];

/// What a case's stand-ins leave: points outside after the projection, vertices after
/// each of the two clippings.
static SCRIPT: Mutex<(u32, u32, u32)> = Mutex::new((0, 0, 0));
/// Each call: its place and the number of vertices it found.
static CALLED: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

fn handler(place: u32, _: &mut Regs) {
    let (outside, after_depth, after_screen) = *SCRIPT.lock().unwrap();
    unsafe {
        CALLED.lock().unwrap().push((place, game_read(COUNT_RVA)));
        match place {
            0 => game_write(OUTSIDE_RVA, outside),
            3 => game_write(COUNT_RVA, after_depth),
            4 => game_write(COUNT_RVA, after_screen),
            _ => {}
        }
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x44CCEF)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                CALLEES
                    .iter()
                    .position(|&known| rva(known) == callee)
                    .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut regions: Vec<Region> = [
        COUNT_RVA,
        SCREEN_CLIPPED_RVA,
        DEPTH_CLIPPED_RVA,
        DRAWN_RVA,
        NEXT_RECORD_RVA,
        NEXT_ENTRY_RVA,
        OUTSIDE_RVA,
    ]
    .map(at)
    .to_vec();
    regions.push((game_addr(LIST_RVA) as u32, 9 * 4));
    let saved = capture(&regions);
    for count in [1u32, 3, 4, 8] {
        for script in [
            (0u32, 9u32, 9u32),
            (count, 9, 9),
            (1, 3, 3),
            (1, 2, 5),
            (2, 5, 2),
            (0, 0, 0),
            (0, 0, 3),
            (count + 1, 4, 0xFFFF_FFFF),
        ] {
            let run = |rust: bool| unsafe {
                restore(&regions, &vec![0xAA; saved.len()]);
                game_write(COUNT_RVA, count);
                *SCRIPT.lock().unwrap() = script;
                CALLED.lock().unwrap().clear();
                set_handler(handler);
                let done = if rust {
                    draw_clipped_polygon(&mut |step| {
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
            t.check(theirs == mine && !theirs.2.is_empty(), || {
                format!("{count} vertices, script {script:?}: original {theirs:x?}; rust {mine:x?}")
            });
        }
    }
    restore(&regions, &saved);
    t.finish()
}
