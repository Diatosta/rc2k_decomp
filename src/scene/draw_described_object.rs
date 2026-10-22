//! `scene::draw_described_object` (0x451B61): an object drawn from its description at esi
//! (a course face record of kind 0xFF, part R1 of stream A; also `scene::draw_scene_object`'s
//! modes 1 and 2).
//!
//! In:  esi = the description: +1 the object (`g_objectNumber`), +4 its scene point
//!      (`g_objectPlace`), +0x30 .. +0x38 the offset from it, +8 / +0xC / +0x10 its turns
//!      (<< 8; the first & 0xFFFF), with +0x11 the second turns +9 / +0x12 / +0x13, +0x14 ..
//!      +0x17 its light, +0x18 its size, +0x22 / +0x23 its light's slant, +0x24 the sun's
//!      light, +0x25 bits 1 .. 2, +0x2C its shine
//! Out: the registers as the drawing leaves them (callers return, or `popa`)
//!
//! Nothing for an object above 0x64 or a point past `g_scenePointCount`, nor in a race for
//! object 0x3F. Objects 0x38 and 0x3A .. 0x3F are signs: upright (0x2000), full light,
//! 0x3D / 0x3E half bright when any of +0x19 .. +0x1B, +0x20, +0x28 .. +0x2B is set.
//! Then `dword_6365C8` 2, `race::place_object_matrix`, `g_lightKind` 2 (1 when the
//! object's material record has +0x1F), `scene::draw_scene_object` (esi the material
//! record, ecx the object, eax its +0x1F); after it (and in the cases of nothing)
//! `dword_635058` 0 and `g_objectPlace` -1.
//! Refuses (before anything) what `draw_scene_object` refuses for the object (a race car
//! without its block). Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose two calls go to stand-ins, against the Rust version with the same: test
//! descriptions (plain objects, signs, the sun's light, too high, past the points, 0x3F
//! in a race and not).

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::race::face_stage::compare_plain;
use crate::race::level_init::note;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::draw_described_object",
    rva(0x451B61),
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

const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const PLACE_RVA: usize = rva(0x634FFC); // g_objectPlace

fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// What `scene::draw_scene_object` would refuse for `object`: in a race (`g_inRace`) a car
/// (0x50 ..) without its block.
unsafe fn check_object(object: u32) -> Result<(), Refused> {
    unsafe {
        let car = object.wrapping_sub(0x50);
        if game_read::<u32>(rva(0x5C5CEC)) != 0
            && car < 0xA0
            && game_read::<u32>(rva(0x71B3B8) + 4 * (car / 0x14) as usize) == 0
        {
            return Err(BadInput("a car without its block").into());
        }
        Ok(())
    }
}

/// The Rust version. Parts: 1 `race::place_object_matrix`, 2 `scene::draw_scene_object`.
pub unsafe fn draw_described_object(
    r: &mut Regs,
    part: crate::race::frame::Part,
) -> Result<(), Refused> {
    unsafe {
        let d = r.esi;
        let byte = |o: u32| ((d + o) as *const u8).read() as u32;
        let object = byte(1);
        let place = get(d + 4);
        let shown = object <= 0x64
            && place < game_read::<u32>(rva(0x5CCD20))
            && !(game_read::<u32>(rva(0x621480)) != 0 && object == 0x3F);
        if shown {
            check_object(object)?;
        }
        r.ebx = object;
        game_write(OBJECT_RVA, object);
        let end = || {
            game_write(rva(0x635058), 0u32);
            game_write(PLACE_RVA, u32::MAX);
        };
        if object > 0x64 {
            end();
            return Ok(());
        }
        r.eax = place;
        if place >= game_read::<u32>(rva(0x5CCD20)) {
            end();
            return Ok(());
        }
        game_write(PLACE_RVA, place);
        if !shown {
            end();
            return Ok(());
        }
        let p = place
            .wrapping_mul(12)
            .wrapping_add(game_read(rva(0x5CCCC0)));
        r.eax = get(p).wrapping_add(get(d + 0x30));
        r.ebx = get(p + 4).wrapping_add(get(d + 0x34));
        r.ecx = get(p + 8).wrapping_add(get(d + 0x38));
        game_write(rva(0x63501C), r.eax);
        game_write(rva(0x635020), r.ebx);
        game_write(rva(0x635024), r.ecx);
        game_write(rva(0x63503C), (byte(8) << 8) & 0xFFFF);
        game_write(rva(0x635040), byte(0xC) << 8);
        game_write(rva(0x635044), byte(0x10) << 8);
        game_write(rva(0x635060), 0u32);
        game_write(rva(0x63505C), 0u32);
        game_write(rva(0x635064), 0u32);
        if byte(0x11) != 0 {
            game_write(rva(0x635064), byte(9) << 8);
            game_write(rva(0x63505C), byte(0x12) << 8);
            game_write(rva(0x635060), byte(0x13) << 8);
        }
        game_write(rva(0x635054), get(d + 0x2C));
        game_write(rva(0x635058), (byte(0x25) >> 1) & 3);
        let light = |lowest: u32, highest: u32, level: u32, strength: u32| {
            game_write(rva(0x635084), lowest);
            game_write(rva(0x635088), highest);
            game_write(rva(0x63508C), level);
            game_write(rva(0x635090), strength);
        };
        if object == 0x38 || (0x3A..0x40).contains(&object) {
            // eax as the original leaves it at the call: the size it last loads.
            r.eax = byte(0x18);
            game_write(rva(0x63503C), 0x2000u32);
            game_write(rva(0x635040), 0u32);
            game_write(rva(0x635044), 0u32);
            game_write(rva(0x635038), byte(0x18));
            light(0xFF, 0xFF, 0xFF, 0);
            game_write(rva(0x635094), 0u32);
            game_write(rva(0x635098), 0u32);
            if (object == 0x3E || object == 0x3D)
                && [0x28u32, 0x29, 0x2A, 0x2B, 0x19, 0x1A, 0x1B, 0x20]
                    .iter()
                    .any(|&o| byte(o) != 0)
            {
                game_write(rva(0x635088), 0x80u32);
            }
        } else {
            game_write(rva(0x635038), byte(0x18));
            light(byte(0x14), byte(0x15), byte(0x16), byte(0x17));
            game_write(rva(0x635094), byte(0x22));
            game_write(rva(0x635098), byte(0x23));
            // eax at the call: +0x24, or 0xFF after the sun's light.
            r.eax = byte(0x24);
            if byte(0x24) != 0 {
                r.eax = 0xFF;
                game_write(
                    rva(0x635094),
                    game_read::<u32>(rva(0x609338)).wrapping_neg(),
                );
                game_write(rva(0x635098), game_read::<u32>(rva(0x60933C)));
                game_write(rva(0x63508C), game_read::<u32>(rva(0x6092E4)));
                game_write(rva(0x635090), game_read::<u32>(rva(0x6092E8)));
                game_write(rva(0x635084), 0u32);
                game_write(rva(0x635088), 0xFFu32);
            }
        }
        game_write(rva(0x6365C8), 2u32);
        part(1, r)?;
        game_write(rva(0x62DAF4), 2u32);
        r.ecx = game_read(OBJECT_RVA);
        let kind: u32 = game_read(rva(0x5CD27C) + 4 * r.ecx as usize);
        r.esi = (game_addr(rva(0x97DF30)) as u32).wrapping_add(kind.wrapping_mul(0x80));
        r.eax = ((r.esi + 0x1F) as *const u8).read() as u32;
        if r.eax != 0 {
            game_write(rva(0x62DAF4), 1u32);
        }
        part(2, r)?;
        end();
        Ok(())
    }
}

/// The parts in the game.
pub fn described_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if place == 1 {
            *r = crate::race::frame_draw::place_object_matrix(r);
            Ok(())
        } else {
            crate::scene::draw_scene_object::draw_scene_object_in_game(r)
        }
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    if let Err(e) = unsafe { draw_described_object(r, &mut described_part) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_described_object(r, &mut described_part) }?;
    Ok(())
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::stand_ins::set_handler(note);
    let copy = match crate::audio::test_copy::make(
        &HOOK,
        &crate::audio::test_copy::Copy {
            address: 0x451B61,
            len: 0x2DD,
            entry: &[0x0F, 0xB6, 0x5E, 0x01, 0x89, 0x1D, 0xFC, 0x14],
            calls: &[0x27E, 0x2B8],
            jumps_out: &[],
        },
        &|c| {
            [(0x42739Ausize, 1u32), (0x451E3E, 2)]
                .iter()
                .find(|(a, _)| rva(*a) == c)
                .map(|(_, p)| crate::install::stand_ins::stand_in(*p))
        },
        &[],
        0,
    ) {
        Ok(c) => c,
        Err(e) => return t.skip(&format!("copy: {e}")),
    };
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    let regions: Vec<Region> = vec![
        g(0x63501C, 0xC),
        g(0x635038, 0x30),
        g(0x635084, 0x18),
        at(OBJECT_RVA),
        at(PLACE_RVA),
        at(rva(0x6365C8)),
        at(rva(0x62DAF4)),
    ];
    let mut kept = regions.clone();
    kept.extend([
        at(rva(0x5CCCC0)),
        at(rva(0x5CCD20)),
        at(rva(0x621480)),
        at(rva(0x5C5CEC)),
    ]);
    let saved = capture(&kept);
    let points: Vec<u32> = (0..12u32).map(|k| k.wrapping_mul(0x0012_3457)).collect();
    let mut d = vec![0u8; 0x40];
    for (k, b) in d.iter_mut().enumerate() {
        *b = (k as u8).wrapping_mul(29).wrapping_add(3);
    }
    let f = d.as_mut_ptr() as u32;
    for (object, place, race, second, sun, extras) in [
        (0x10u8, 1u32, 0u32, 0u8, 0u8, 0u8),
        (0x20, 2, 1, 1, 0, 0),
        (0x21, 3, 1, 0, 1, 0),
        (0x38, 0, 0, 1, 0, 0),
        (0x3D, 1, 0, 0, 0, 0),
        (0x3E, 1, 1, 0, 0, 5),
        (0x3F, 1, 1, 0, 0, 0),
        (0x3F, 1, 0, 0, 0, 0),
        (0x65, 1, 0, 0, 0, 0),
        (0x10, 4, 0, 0, 0, 0),
    ] {
        let line = compare_plain(
            &format!(
                "object {object:#x} place {place} race {race} second {second} sun {sun} extras {extras}"
            ),
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x5CCCC0), points.as_ptr() as u32);
                game_write(rva(0x5CCD20), 4u32);
                game_write(rva(0x621480), race);
                game_write(rva(0x5C5CEC), 0u32);
                ((f + 1) as *mut u8).write(object);
                ((f + 4) as *mut u32).write_unaligned(place);
                ((f + 0x11) as *mut u8).write(second);
                ((f + 0x24) as *mut u8).write(sun);
                for o in [0x28u32, 0x29, 0x2A, 0x2B, 0x19, 0x1A, 0x1B, 0x20] {
                    ((f + o) as *mut u8).write(0);
                }
                ((f + 0x2A) as *mut u8).write(extras);
            },
            &regions,
            copy.addr(),
            Regs {
                esi: f,
                ..Regs::SENTINEL
            },
            &mut |r| unsafe {
                draw_described_object(r, &mut |p, r| {
                    note(p, r);
                    Ok(())
                })
            },
        );
        t.check(line.is_none(), || line.unwrap_or_default());
    }
    restore(&kept, &saved);
    drop(d);
    t.finish()
}
