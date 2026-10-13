//! `scene::clip_polygon_by_depth` (0x44DB4F): cuts the polygon to the depths that can
//! be drawn.
//!
//! In:  nothing
//! Out: the original leaves every register changed; its callers go on by the number
//!      of vertices alone, and the Rust version keeps the registers
//!
//! `dword_624124` = 1; `dword_62411C` = 0x1000 and one pass of `scene/clip.rs` that
//! keeps what is deeper than that (`sub_44DB93`); then, with `dword_621510` at 0,
//! `dword_621900` not 0 and three vertices or more left, `dword_62411C` =
//! `dword_624230` and a pass that keeps what is not deeper (`sub_44DEAA`).
//! Seven callers. Refuses as `scene::clip_polygon_to_screen`.
//! Shadow mode: the Rust version, its writes put back, the original, compared.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::BadInput;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::scene::clip::{
    DEPTH_CLIPPED_RVA, Edge, PLANE_RVA, clip_pass, dispatch_clip, listed_with_records, or_undone,
    regions,
};
use crate::scene::clip_test::{POLYGONS, Polygons};
use crate::service::harness::at;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::clip_polygon_by_depth",
    rva(0x44DB4F),
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

const NEAREST: i32 = 0x1000;
const NO_FAR_RVA: usize = rva(0x621510); // dword_621510
const FAR_ON_RVA: usize = rva(0x621900); // dword_621900
const FARTHEST_RVA: usize = rva(0x624230); // dword_624230

// ---- Rust version ----

pub unsafe fn clip_polygon_by_depth() -> Result<(), BadInput> {
    unsafe {
        listed_with_records()?;
        or_undone(|| {
            game_write(DEPTH_CLIPPED_RVA, 1u32);
            game_write(PLANE_RVA, NEAREST);
            clip_pass(Edge::Depth(NEAREST), true, 0)?;
            let far = game_read::<u32>(NO_FAR_RVA) == 0 && game_read::<u32>(FAR_ON_RVA) != 0;
            if far && game_read::<i32>(COUNT_RVA).wrapping_sub(3) >= 0 {
                let farthest: i32 = game_read(FARTHEST_RVA);
                game_write(PLANE_RVA, farthest);
                clip_pass(Edge::Depth(farthest), false, 0)?;
            }
            Ok(())
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_clip(&HOOK, r, call_original, &regions, &|| unsafe {
        clip_polygon_by_depth()
    });
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let switches = [NO_FAR_RVA, FAR_ON_RVA, FARTHEST_RVA];
    let mut polygons = Polygons::begin(&switches.map(at));
    for (n, corners) in POLYGONS.into_iter().enumerate() {
        for (variant, (no_far, far_on, farthest)) in [
            (0u32, 1u32, 50_000i32),
            (0, 0, 50_000),
            (1, 1, 50_000),
            (0, 5, 8_500),
            (0, 1, 0x7FFF_FFFF),
            (0, 1, 16_777_217),
        ]
        .into_iter()
        .enumerate()
        {
            let run = |polygons: &mut Polygons, rust: bool| unsafe {
                polygons.set(corners, variant, variant % 2 == 1);
                game_write(NO_FAR_RVA, no_far);
                game_write(FAR_ON_RVA, far_on);
                game_write(FARTHEST_RVA, farthest);
                let done = if rust {
                    clip_polygon_by_depth().is_ok()
                } else {
                    call_original(&mut Regs::SENTINEL.clone());
                    true
                };
                (done, polygons.state())
            };
            let (theirs, mine) = (run(&mut polygons, false), run(&mut polygons, true));
            t.check(theirs == mine, || {
                format!(
                    "polygon {n}, switches {no_far} {far_on}, farthest {farthest}: rust done {}; first different byte {:?}, pool the same {}",
                    mine.0,
                    theirs.1.0.iter().zip(&mine.1.0).position(|(a, b)| a != b),
                    theirs.1.1 == mine.1.1
                )
            });
        }
    }
    polygons.end();
    t.finish()
}
