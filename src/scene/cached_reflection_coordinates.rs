//! `scene::cached_reflection_coordinates` (0x451908): `scene::reflection_coordinates`
//! behind a table of results kept.
//!
//! In:  nothing; the two numbers in `dword_6399A8` and `dword_6399AC`
//! Out: the coordinates in `dword_6399B0`, `dword_6399B4`. With the table in use:
//!      ebx = the place in it, eax = the second coordinate, and after a working out
//!      edx and esi as that leaves them. Without it: as
//!      `scene::reflection_coordinates`
//!
//! With `dword_63D9B8` at 0 the original jumps straight on into the working out.
//! Else the normal's number & 0x3FF is a place in four tables of 0x400 dwords
//! (`dword_6399B8` the normal's number, `dword_63A9B8` the point's, `dword_63B9B8` and
//! `dword_63C9B8` the coordinates): if both numbers there are the ones asked for,
//! the coordinates come from the table; else they are worked out and all four kept.
//! One caller, `sub_45188C`.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, EDX, ESI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::scene::reflection_coordinates::{
    COORDINATES_RVA, Mirrors, NORMAL_NUMBER_RVA, POINT_NUMBER_RVA, TEST_NORMALS, TEST_POINTS,
    reflection_coordinates, regions as worked_out_regions,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::cached_reflection_coordinates",
    rva(0x451908),
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

const IN_USE_RVA: usize = rva(0x63D9B8); // dword_63D9B8
/// The four tables, 0x1000 bytes apart: the normal's number, the point's, the two
/// coordinates.
const TABLES_RVA: usize = rva(0x6399B8); // dword_6399B8, dword_63A9B8, dword_63B9B8, dword_63C9B8
const TABLE_BYTES: usize = 0x1000;

// ---- Rust version ----

pub unsafe fn cached_reflection_coordinates() -> Result<Left, Refused> {
    unsafe {
        let worked_out = || {
            reflection_coordinates().map(|left| {
                Left::nothing()
                    .with(EAX, left.eax)
                    .with(EDX, left.edx)
                    .with(ESI, left.esi)
            })
        };
        if game_read::<u32>(IN_USE_RVA) == 0 {
            return Ok(worked_out()?);
        }
        let asked: [u32; 2] = [game_read(NORMAL_NUMBER_RVA), game_read(POINT_NUMBER_RVA)];
        let place = (asked[0] & 0x3FF) as usize;
        let slot = |table: usize| TABLES_RVA + table * TABLE_BYTES + place * 4;
        let kept: [u32; 2] = [game_read(slot(0)), game_read(slot(1))];
        if kept == asked {
            let coordinates: [u32; 2] = [game_read(slot(2)), game_read(slot(3))];
            game_write(COORDINATES_RVA, coordinates);
            return Ok(Left::nothing()
                .with(EBX, place as u32)
                .with(EAX, coordinates[1]));
        }
        let left = worked_out()?;
        let coordinates: [u32; 2] = game_read(COORDINATES_RVA);
        for (table, value) in [asked[0], asked[1], coordinates[0], coordinates[1]]
            .into_iter()
            .enumerate()
        {
            game_write(slot(table), value);
        }
        Ok(left.with(EBX, place as u32).with(EAX, coordinates[1]))
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = worked_out_regions();
    let place = unsafe { game_read::<u32>(NORMAL_NUMBER_RVA) & 0x3FF } as usize;
    regions.extend((0..4).map(|table| {
        (
            game_addr(TABLES_RVA + table * TABLE_BYTES + place * 4) as u32,
            4,
        )
    }));
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { cached_reflection_coordinates() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut inputs = Mirrors::inputs();
    inputs.push(at(IN_USE_RVA));
    let saved_inputs = capture(&inputs);
    let mirrors = Mirrors::new();
    // The numbers the cases ask with, for the regions.
    unsafe { mirrors.set(0, 0, false) };
    let mut regions = worked_out_regions();
    for normal in 0..TEST_NORMALS as usize {
        regions.extend((0..4).map(|table| {
            (
                game_addr(TABLES_RVA + table * TABLE_BYTES + (0x10 + normal) * 4) as u32,
                4,
            )
        }));
    }
    let saved_regions = capture(&regions);
    for in_use in [0u32, 1] {
        for normal in 0..TEST_NORMALS {
            for point in 0..TEST_POINTS {
                // What the table holds: nothing, the answer to this very question
                // (as numbers of its own), or another point's.
                for kept in 0..3u32 {
                    check_state(
                        &mut t,
                        &format!("in use {in_use}, normal {normal}, point {point}, kept {kept}"),
                        call_original,
                        Regs::SENTINEL,
                        &|| unsafe {
                            restore(&regions, &vec![0xAA; saved_regions.len()]);
                            mirrors.set(normal, point, kept == 2);
                            game_write(IN_USE_RVA, in_use);
                            let place = (0x10 + normal) as usize;
                            let slot = |table: usize| TABLES_RVA + table * TABLE_BYTES + place * 4;
                            let asked: [u32; 2] =
                                [game_read(NORMAL_NUMBER_RVA), game_read(POINT_NUMBER_RVA)];
                            match kept {
                                1 => {
                                    game_write(slot(0), asked[0]);
                                    game_write(slot(1), asked[1]);
                                    game_write(slot(2), 0x1111_0000 + normal);
                                    game_write(slot(3), 0x2222_0000 + point);
                                }
                                2 => {
                                    game_write(slot(0), asked[0]);
                                    game_write(slot(1), asked[1].wrapping_add(1));
                                }
                                _ => {}
                            }
                        },
                        &regions,
                        &rust,
                    );
                }
            }
        }
    }
    restore(&regions, &saved_regions);
    restore(&inputs, &saved_inputs);
    t.finish()
}
