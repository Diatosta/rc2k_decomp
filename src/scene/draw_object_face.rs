//! `scene::draw_object_face` (0x450BDA): one face of an object, from its record to the
//! work table and on to the drawing.
//!
//! In:  ebp = the face: +0 the number of vertices, +7 a byte whose low five bits go to
//!      `dword_636608`, +9 a byte for `dword_6240D8` (how much the face reflects),
//!      +0xC a dword for `dword_6240DC`, from +0x10 a vertex every 0x10 bytes: the
//!      point's number, two texture coordinates, the normal's number
//! Out: every register preserved on the usual way; on the other (below) the original
//!      leaves them changed and the Rust version does not
//!
//! Nothing at all unless `dword_6365C4` is 0. Then, per vertex, an entry of the work
//! table (`sub_450E37`): the point's number negated (an object's point), the point's
//! place in the view copied to `dword_62DA8C` .., the coordinates, and the light:
//! `scene::vertex_light`, or plain white with `dword_6365C8` at 4, with the face's
//! kind `dword_6D18CC` at 5, or with a kind of 10 and up other than 0x1C while
//! `dword_6D18D0` is not 0.
//! With reflections (`dword_5CCE74` and `dword_6238AC` not 0, `dword_6365C8` not 4, the
//! face's own amount not 0) each entry also gets (`sub_45188C`) a second pair of
//! coordinates from `scene::cached_reflection_coordinates` and, in its byte +0x21,
//! the face's amount times `dword_5CCE74` times the first entry's byte +0x20, over
//! 0x1000, held to 255; `dword_621504` says whether any of them was not 0, and the
//! second texture is 0x87. `dword_621504` is only cleared when this part runs: a
//! face without it goes by what an earlier one left.
//! Then the first pass's fade (`dword_63660C`, when `dword_606284` is set and the
//! face's five bits are `dword_608CAC`), the two switches of
//! `scene::draw_polygon` (the first texture always, the second with a reflection),
//! mode 4; or, with `dword_639970` not 0x100, the first sixteen entries get that as
//! their byte +0x21 and their first coordinates as their second, the switches the
//! other way round, the second texture = the first, mode 3. Then
//! `scene::draw_object_polygon`, mode 0, the fade off.
//! The other way, with `dword_621480` set and `dword_6214DC` at 2 (`sub_450EE2` per
//! vertex): the coordinates as the second pair, 0xFF in the byte +0x21, the light
//! `dword_635054`; second texture = first, mode 4, and the drawing with the depth
//! test off around it.
//! One caller, `scene::draw_object_faces`; a second call, at 0x450B9B, is in code
//! nothing refers to.
//! Refuses a face of no vertices (the original goes through 2^32) or of more than 64,
//! and when there are no records.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose drawing and depth test calls note what they
//! find.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::cached_reflection_coordinates::cached_reflection_coordinates;
use crate::scene::draw_clipped_polygon::draw_clipped_polygon_in_game;
use crate::scene::draw_object_polygon::{self, draw_object_polygon};
use crate::scene::draw_polygon::{Game, Outside, SWITCH_A_RVA, SWITCH_B_RVA};
use crate::scene::polygon::{ENTRIES, ENTRIES_RVA, ENTRY_LEN, dword, put};
use crate::scene::project_point::RECORDS_RVA;
use crate::scene::project_polygon::project_polygon;
use crate::scene::reflection_coordinates::{
    COORDINATES_RVA, Mirrors, NORMAL_NUMBER_RVA, POINT_NUMBER_RVA, regions as mirror_regions,
};
use crate::scene::submit_polygon;
use crate::scene::vertex_light::vertex_light;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::draw_object_face", rva(0x450BDA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const OFF_RVA: usize = rva(0x6365C4); // dword_6365C4
const FACE_DWORD_RVA: usize = rva(0x6240DC); // dword_6240DC
const FACE_BITS_RVA: usize = rva(0x636608); // dword_636608
const FACE_SHINE_RVA: usize = rva(0x6240D8); // dword_6240D8
const REFLECTS_RVA: usize = rva(0x621500); // dword_621500
const ANY_SHINE_RVA: usize = rva(0x621504); // dword_621504
const SHINE_RVA: usize = rva(0x5CCE74); // dword_5CCE74
const SHINE_ON_RVA: usize = rva(0x6238AC); // dword_6238AC
const VIEW_KIND_RVA: usize = rva(0x6365C8); // dword_6365C8
const OTHER_WAY_RVA: usize = rva(0x621480); // dword_621480
const OTHER_WAY_KIND_RVA: usize = rva(0x6214DC); // dword_6214DC
const POINT_PLACE_RVA: usize = rva(0x62DA8C); // dword_62DA8C, dword_62DA90, dword_62DA94
const FACE_KIND_RVA: usize = rva(0x6D18CC); // dword_6D18CC
const FACE_UNLIT_RVA: usize = rva(0x6D18D0); // dword_6D18D0
const TEXTURE_A_RVA: usize = rva(0x6240BC); // dword_6240BC
const TEXTURE_B_RVA: usize = rva(0x6240C0); // dword_6240C0
const REFLECTION_TEXTURE: u32 = 0x87;
const FADED_RVA: usize = rva(0x63660C); // dword_63660C
const FADE_ALPHA_RVA: usize = rva(0x606284); // dword_606284
const FADED_BITS_RVA: usize = rva(0x608CAC); // dword_608CAC
const SEE_THROUGH_RVA: usize = rva(0x639970); // dword_639970
const MODE_RVA: usize = rva(0x63DAB4); // g_polygonMode
const OTHER_LIGHT_RVA: usize = rva(0x635054); // dword_635054
/// What `scene::draw_object_polygon` can keep.
const MOST: u32 = 0x40;

// ---- Rust version ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Draw,
    DepthTestOff,
    DepthTestOn,
}

pub unsafe fn draw_object_face(
    face: usize,
    step: &mut dyn FnMut(Step) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(OFF_RVA) != 0 {
            return Ok(());
        }
        if face == 0 {
            return Err(BadInput("no face").into());
        }
        let count = dword(face);
        let records: u32 = game_read(RECORDS_RVA);
        if count == 0 || count > MOST || records == 0 {
            return Err(BadInput("a face of no vertices or more than 64, or no records").into());
        }
        let entries = game_addr(ENTRIES_RVA);
        let vertices =
            (0..count as usize).map(|n| (entries + n * ENTRY_LEN, face + 0x10 + n * 0x10));
        let view_kind: u32 = game_read(VIEW_KIND_RVA);
        let shine: u32 = game_read(SHINE_RVA);

        game_write(FACE_DWORD_RVA, dword(face + 0xC));
        game_write(
            FACE_BITS_RVA,
            ((face + 7) as *const u8).read() as u32 & 0x1F,
        );
        let face_shine = ((face + 9) as *const u8).read() as u32;
        game_write(FACE_SHINE_RVA, face_shine);
        let reflects =
            shine != 0 && game_read::<u32>(SHINE_ON_RVA) != 0 && view_kind != 4 && face_shine != 0;
        game_write(REFLECTS_RVA, reflects as u32);
        game_write(COUNT_RVA, count);

        if game_read::<u32>(OTHER_WAY_RVA) != 0 && game_read::<u32>(OTHER_WAY_KIND_RVA) == 2 {
            for (entry, vertex) in vertices {
                put(entry, dword(vertex).wrapping_neg());
                put(entry + 0x10, dword(vertex + 4));
                put(entry + 0x14, dword(vertex + 8));
                ((entry + 0x21) as *mut u8).write(0xFF);
                put(entry + 0x18, game_read(OTHER_LIGHT_RVA));
            }
            game_write(TEXTURE_B_RVA, game_read::<u32>(TEXTURE_A_RVA));
            game_write(SWITCH_A_RVA, 0u32);
            game_write(SWITCH_B_RVA, 1u32);
            game_write(MODE_RVA, 4u32);
            step(Step::DepthTestOff)?;
            step(Step::Draw)?;
            step(Step::DepthTestOn)?;
            game_write(MODE_RVA, 0u32);
            return Ok(());
        }

        let (face_kind, face_unlit): (u32, u32) =
            (game_read(FACE_KIND_RVA), game_read(FACE_UNLIT_RVA));
        let lit = view_kind != 4
            && (face_kind == 0x1C || (face_kind != 5 && (face_kind < 0xA || face_unlit == 0)));
        for (entry, vertex) in vertices.clone() {
            let point = dword(vertex);
            put(entry, point.wrapping_neg());
            let record = records.wrapping_add(((point & 0xFFF) + 0x1000) << 5) as usize;
            game_write(
                POINT_PLACE_RVA,
                ((record + 0x10) as *const [u32; 3]).read_unaligned(),
            );
            put(entry + 8, dword(vertex + 4));
            put(entry + 0xC, dword(vertex + 8));
            let light = if lit {
                vertex_light(dword(vertex + 0xC))?
            } else {
                0x00FF_FFFF
            };
            put(entry + 0x18, light);
        }
        if reflects {
            game_write(ANY_SHINE_RVA, 0u32);
            for (entry, vertex) in vertices {
                game_write(NORMAL_NUMBER_RVA, dword(vertex + 0xC));
                game_write(POINT_NUMBER_RVA, dword(vertex).wrapping_neg());
                cached_reflection_coordinates()?;
                let coordinates: [u32; 2] = game_read(COORDINATES_RVA);
                put(entry + 0x10, coordinates[0]);
                put(entry + 0x14, coordinates[1]);
                let first_fog = ((entries + 0x20) as *const u8).read() as u32;
                let amount =
                    (face_shine.wrapping_mul(shine).wrapping_mul(first_fog) >> 12).min(0xFF);
                if amount != 0 {
                    game_write(ANY_SHINE_RVA, 1u32);
                }
                ((entry + 0x21) as *mut u8).write(amount as u8);
            }
            game_write(TEXTURE_B_RVA, REFLECTION_TEXTURE);
        }
        let faded = game_read::<u32>(FADE_ALPHA_RVA) != 0
            && game_read::<u32>(FACE_BITS_RVA) == game_read::<u32>(FADED_BITS_RVA);
        game_write(FADED_RVA, faded as u32);
        if game_read::<u32>(ANY_SHINE_RVA) == 0 {
            game_write(REFLECTS_RVA, 0u32);
        }
        game_write(SWITCH_A_RVA, 1u32);
        game_write(SWITCH_B_RVA, game_read::<u32>(REFLECTS_RVA));
        let see_through: u32 = game_read(SEE_THROUGH_RVA);
        if see_through != 0x100 {
            for n in 0..0x10 {
                let entry = entries + n * ENTRY_LEN;
                ((entry + 0x21) as *mut u8).write(see_through as u8);
                put(entry + 0x10, dword(entry + 8));
                put(entry + 0x14, dword(entry + 0xC));
            }
            game_write(SWITCH_A_RVA, 0u32);
            game_write(SWITCH_B_RVA, 1u32);
            game_write(MODE_RVA, 3u32);
            game_write(TEXTURE_B_RVA, game_read::<u32>(TEXTURE_A_RVA));
        } else {
            game_write(MODE_RVA, 4u32);
        }
        step(Step::Draw)?;
        game_write(MODE_RVA, 0u32);
        game_write(FADED_RVA, 0u32);
        Ok(())
    }
}

/// The function with its parts' Rust versions, drawing for real.
pub unsafe fn draw_object_face_in_game(face: usize, entry: &Regs) -> Result<(), Refused> {
    unsafe {
        draw_object_face(face, &mut |step| match step {
            Step::Draw => draw_object_polygon(&mut |step| match step {
                draw_object_polygon::Step::Project => project_polygon().map(|_| ()),
                draw_object_polygon::Step::Draw => draw_clipped_polygon_in_game(entry),
            }),
            Step::DepthTestOff => {
                Game(submit_polygon::Game(entry)).state(0xE, 0);
                Ok(())
            }
            Step::DepthTestOn => {
                Game(submit_polygon::Game(entry)).state(0xE, 1);
                Ok(())
            }
        })
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
    if let Err(e) = unsafe { draw_object_face_in_game(entry.ebp as usize, &entry) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0x25D;
const ENTRY_BYTES: [u8; 12] = [
    0x83, 0x3D, 0xC4, 0x65, 0x63, 0x00, 0x02, 0x74, 0x13, 0x83, 0x3D, 0xC4,
];
/// The three per-vertex functions, the drawing three times, the depth test off and on
/// twice each.
const CALLS: [usize; 10] = [
    0x9E, 0xDF, 0x1B2, 0x1E7, 0x230, 0x235, 0x23A, 0x241, 0x246, 0x24B,
];

fn written() -> Vec<Region> {
    let mut regions = vec![
        (game_addr(ENTRIES_RVA) as u32, ENTRIES as usize * ENTRY_LEN),
        (game_addr(POINT_PLACE_RVA) as u32, 12),
        // The light's work variables.
        (game_addr(rva(0x62DAA8)) as u32, 0x20),
    ];
    regions.extend(
        [
            COUNT_RVA,
            FACE_DWORD_RVA,
            FACE_BITS_RVA,
            FACE_SHINE_RVA,
            REFLECTS_RVA,
            ANY_SHINE_RVA,
            TEXTURE_B_RVA,
            FADED_RVA,
            SWITCH_A_RVA,
            SWITCH_B_RVA,
            MODE_RVA,
            NORMAL_NUMBER_RVA,
            POINT_NUMBER_RVA,
        ]
        .map(at),
    );
    regions.extend(mirror_regions());
    regions
}

/// Each call: its place and everything written by then.
static CALLED: Mutex<Vec<(u32, Vec<u8>)>> = Mutex::new(Vec::new());

fn handler(place: u32, _: &mut Regs) {
    CALLED.lock().unwrap().push((place, capture(&written())));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x450BDA)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| {
                [0x451F69, 0x44E3D1, 0x44E3E1]
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
    let regions = written();
    let mut inputs = Mirrors::inputs();
    inputs.extend(
        [
            OFF_RVA,
            SHINE_RVA,
            SHINE_ON_RVA,
            VIEW_KIND_RVA,
            OTHER_WAY_RVA,
            OTHER_WAY_KIND_RVA,
            FACE_KIND_RVA,
            FACE_UNLIT_RVA,
            TEXTURE_A_RVA,
            FADE_ALPHA_RVA,
            FADED_BITS_RVA,
            SEE_THROUGH_RVA,
            OTHER_LIGHT_RVA,
            rva(0x63D9B8), // g_reflectionsKept
        ]
        .map(at),
    );
    let saved = (capture(&regions), capture(&inputs));
    let mirrors = Mirrors::new();
    // A face of four vertices: the points and normals `Mirrors` has records for.
    let mut face = [0u32; 4 + 4 * 4];
    face[0] = 4;
    face[1] = 0x6500_0000; // byte +7: 0x65, its low five bits 5
    face[3] = 0x1234_5678;
    for n in 0..4u32 {
        let at = 4 + n as usize * 4;
        face[at..at + 4].copy_from_slice(&[
            0x3020 + n,
            0x0040_0000 * n,
            0x0010_0000 + n,
            0x5010 + (3 - n),
        ]);
    }
    let mut cases = 0u32;
    for off in [0u32, 1, 2, 3] {
        for (shine, shine_on, face_shine) in [
            (0u32, 1u32, 0x80u32),
            (0x40, 1, 0x80),
            (0x40, 0, 0x80),
            (0x40, 1, 0),
            (0x1000, 1, 0xFF),
        ] {
            for (other_way, other_kind) in [(0u32, 2u32), (1, 2), (1, 0)] {
                cases += 1;
                let view_kind = if cases.is_multiple_of(5) { 4 } else { 0 };
                let (face_kind, face_unlit) =
                    [(0x1Cu32, 1u32), (5, 0), (3, 1), (0xB, 1), (0xB, 0)][(cases % 5) as usize];
                let see_through = [0x100u32, 0x55, 0x100][(cases % 3) as usize];
                let (fade, any_shine) = (cases % 2, cases % 4 / 2);
                face[2] = face_shine << 8; // byte +9
                let face_at = face.as_ptr() as usize;
                let run = |rust: bool| unsafe {
                    restore(&regions, &vec![0xAA; saved.0.len()]);
                    mirrors.set(0, 0, cases % 2 == 1);
                    game_write(rva(0x63D9B8), 0u32);
                    game_write(ANY_SHINE_RVA, any_shine);
                    game_write(OFF_RVA, off);
                    game_write(SHINE_RVA, shine);
                    game_write(SHINE_ON_RVA, shine_on);
                    game_write(VIEW_KIND_RVA, view_kind);
                    game_write(OTHER_WAY_RVA, other_way);
                    game_write(OTHER_WAY_KIND_RVA, other_kind);
                    game_write(FACE_KIND_RVA, face_kind);
                    game_write(FACE_UNLIT_RVA, face_unlit);
                    game_write(TEXTURE_A_RVA, 0x8000_0123u32);
                    game_write(FADE_ALPHA_RVA, fade);
                    game_write(FADED_BITS_RVA, 5u32);
                    game_write(SEE_THROUGH_RVA, see_through);
                    game_write(OTHER_LIGHT_RVA, 0x00C0_FFEEu32);
                    CALLED.lock().unwrap().clear();
                    set_handler(handler);
                    let done = if rust {
                        draw_object_face(face_at, &mut |step| {
                            act(step as u32, &mut Regs::default());
                            Ok(())
                        })
                        .is_ok()
                    } else {
                        let mut ro = Regs {
                            ebp: face_at as u32,
                            ..Regs::default()
                        };
                        call_game(&mut ro, copy.addr());
                        true
                    };
                    (done, capture(&regions), CALLED.lock().unwrap().clone())
                };
                let (theirs, mine) = (run(false), run(true));
                t.check(theirs == mine && theirs.2.is_empty() == (off != 0), || {
                    let places = |calls: &[(u32, Vec<u8>)]| {
                        calls.iter().map(|call| call.0).collect::<Vec<_>>()
                    };
                    format!(
                        "off {off}, shine {shine:#x} {shine_on} {face_shine:#x}, other way {other_way} {other_kind}, case {cases}: original's calls {:?}, rust done {} calls {:?}; first different byte at the end {:?}, at the first call {:?}",
                        places(&theirs.2),
                        mine.0,
                        places(&mine.2),
                        theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
                        theirs.2.first().zip(mine.2.first()).and_then(|(a, b)| a.1.iter().zip(&b.1).position(|(a, b)| a != b)),
                    )
                });
            }
        }
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
