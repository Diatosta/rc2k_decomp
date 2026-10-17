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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::place_view", rva(0x4C1CC8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

/// The Rust version (see the header). Refuses as `transform_int_points` (never, ecx 1).
pub unsafe fn place_view(entry: &Regs) -> Result<Regs, Refused> {
    unsafe { place_view_with(entry, &FIRST) }
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
    let mut floats = [0u32; 10];
    for case in 0..20u32 {
        crate::scene::matrix3::test_floats(0x51A7_0000 + case, &mut floats);
        let place = [
            (case as i32 - 10) * 0x1_2345,
            -(case as i32) * 0x777,
            0x4000_0000 - case as i32 * 0x0800_0000,
        ];
        check_state(
            t,
            &format!(
                "case {case}: matrix {:x?}, place {place:x?}, height {:#x}",
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
            rust,
        );
    }
    restore(&inputs, &saved);
}

fn regions() -> Vec<Region> {
    vec![(game_addr(FIRST.point) as u32, WRITTEN_LEN)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { place_view(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_with(&mut t, &FIRST, call_original, &[], &rust);
    t.finish()
}
