//! `marks::place_view` (0x4C1CC8): the point (0, 0, 0x20000) turned by the 3 x 3 matrix at
//! `dword_8BC441` (`scene::transform_int_points`, in place at `dword_8BDD71`); the place
//! `dword_8BC435` .. less it into `dword_8BDD7D`, `dword_8BDD81`, and `flt_8C5FE1` less
//! its z plus `dword_8BC43D` into `dword_8BDD85` (each rounded); the matrix's nine
//! dwords copied to `dword_8BDD89`; the point put back to (0, 0, 0x20000).
//!
//! In:  nothing
//! Out: as `scene::transform_int_points` leaves them (ecx -1, esi and edi 0x8BDD7D, ebp
//!      the matrix)
//!
//! One caller, `sub_4C16F9` (the marks' frame). Self-test: the original and the Rust
//! version on matrices and places by lot; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `place_view_plain`, on the converted
//! `scene::transform_int_points` and `rc2k_core::marks::view::view_place` (the x87 version
//! stays for the Rust callers in unconverted code). Shadow mode and the self-test hold the
//! three whole numbers made from floats to one either way (exact at double; the self-test
//! at both test precisions).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::floats::{Floats, at_test_precisions, check_state_floats, floats_entry};
use crate::service::harness::{Region, at, capture, restore};

/// One of the two near-copies' variables (`place_view`'s and `place_view_2`'s).
pub struct ViewPlace {
    pub point: usize,
    pub start_z: u32,
    pub place: usize,
    pub matrix: usize,
    pub out: usize,
    pub height: usize,
    pub matrix_copy: usize,
}

/// `place_view`'s.
pub const FIRST: ViewPlace = ViewPlace {
    point: rva(0x8BDD71), // dword_8BDD71: three whole numbers
    start_z: 0x2_0000,
    place: rva(0x8BC435),       // dword_8BC435: three whole numbers
    matrix: rva(0x8BC441),      // dword_8BC441: nine floats
    out: rva(0x8BDD7D),         // dword_8BDD7D .. dword_8BDD85
    height: rva(0x8C5FE1),      // flt_8C5FE1
    matrix_copy: rva(0x8BDD89), // dword_8BDD89: nine dwords
};

/// From `point` to past the matrix's copy: 0x3C bytes.
pub const WRITTEN_LEN: usize = 0x3C;

/// The shared body (see the header), on `v`'s variables.
pub unsafe fn place_view_with(entry: &Regs, v: &ViewPlace) -> Result<Regs, Refused> {
    let mut g = *entry;
    let start = [0, 0, v.start_z];
    unsafe {
        game_write(v.point, start);
        g.esi = game_addr(v.point) as u32;
        g.edi = g.esi;
        g.ecx = 1;
        g.ebp = game_addr(v.matrix) as u32;
        g = crate::scene::matrix3::transform_int_points(&g)?;
        let point: [i32; 3] = game_read(v.point);
        let place: [i32; 3] = game_read(v.place);
        game_write(
            v.out,
            [
                x87::zero_less_int_plus_int(point[0], place[0]),
                x87::zero_less_int_plus_int(point[1], place[1]),
                x87::less_int_plus_int(game_read(v.height), point[2], place[2]),
            ],
        );
        game_write(v.matrix_copy, game_read::<[u32; 9]>(v.matrix));
        game_write(v.point, start);
    }
    Ok(g)
}

/// The converted shared body (plain floats), on `v`'s variables.
// Converted: `rc2k_core::marks::view::view_place`
pub unsafe fn place_view_with_plain(entry: &Regs, v: &ViewPlace) -> Result<Regs, Refused> {
    let mut g = *entry;
    let start = [0, 0, v.start_z];
    unsafe {
        game_write(v.point, start);
        g.esi = game_addr(v.point) as u32;
        g.edi = g.esi;
        g.ecx = 1;
        g.ebp = game_addr(v.matrix) as u32;
        g = crate::scene::matrix3::transform_int_points_plain(&g)?;
        let out = rc2k_core::marks::view::view_place(
            game_read(v.point),
            game_read(v.place),
            f32::from_bits(game_read(v.height)),
        );
        game_write(v.out, out);
        game_write(v.matrix_copy, game_read::<[u32; 9]>(v.matrix));
        game_write(v.point, start);
    }
    Ok(g)
}

/// The places' tolerances: `out`'s three whole numbers made from floats.
pub fn float_places(v: &ViewPlace) -> [(Region, crate::service::floats::Tolerance); 1] {
    [(
        (game_addr(v.out) as u32, 12),
        crate::service::floats::ROUNDED,
    )]
}

/// The Rust version (see the header). Refuses as `transform_int_points` (never, ecx 1).
pub unsafe fn place_view(entry: &Regs) -> Result<Regs, Refused> {
    unsafe { place_view_with(entry, &FIRST) }
}

/// The converted version (plain floats): the hook's.
pub unsafe fn place_view_plain(entry: &Regs) -> Result<Regs, Refused> {
    unsafe { place_view_with_plain(entry, &FIRST) }
}

/// The test of either: matrices, places and heights by lot, `extra` the other regions it
/// writes.
pub fn test_with(
    t: &mut SelfTest,
    v: &ViewPlace,
    call_original: crate::screens::harness::CallOriginal,
    extra: &[Region],
    rust: crate::service::harness::Rust,
) {
    let mut regions = vec![(game_addr(v.point) as u32, WRITTEN_LEN)];
    regions.extend_from_slice(extra);
    let mut inputs = regions.clone();
    inputs.extend([
        (game_addr(v.place) as u32, 12),
        (game_addr(v.matrix) as u32, 36),
        at(v.height),
    ]);
    let saved = capture(&inputs);
    let places = float_places(v);
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        let mut floats = [0u32; 10];
        for case in 0..20u32 {
            crate::scene::matrix3::test_floats(0x51A7_0000 + case, &mut floats);
            let place = [
                (case as i32 - 10) * 0x1_2345,
                -(case as i32) * 0x777,
                0x4000_0000 - case as i32 * 0x0800_0000,
            ];
            check_state_floats(
                t,
                "marks::place_view",
                &format!(
                    "control word {word:#x}, case {case}: matrix {:x?}, place {place:x?}, height {:#x}",
                    &floats[..9],
                    floats[9]
                ),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&inputs, &saved);
                    for &(a, n) in &regions {
                        std::ptr::write_bytes(a as *mut u8, 0xAA, n);
                    }
                    game_write(v.matrix, <[u32; 9]>::try_from(&floats[..9]).unwrap());
                    game_write(v.place, place);
                    game_write(v.height, floats[9]);
                },
                &regions,
                held,
                rust,
            );
        }
    });
    restore(&inputs, &saved);
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { place_view_plain(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
