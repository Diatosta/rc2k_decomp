//! `hud::add_quad` (0x4B72D1): the HUD's quad record (`dword_75BFB0`: alpha, left, top,
//! right, bottom, colour, then the four corners' texture places as whole numbers) made the
//! four vertices of `g_polygonVertices` (the places over `flt_75C00C` and over
//! `flt_75C010`; the colour's low 24 bits under the alpha) and `hud::add_polygon`.
//!
//! In:  nothing
//! Out: eax the colour going in, then as `add_polygon` leaves them
//!
//! Callers: `sub_4B716A`, `sub_4B7216`, `sub_4B70EE` (reached through code at 0x4B7163,
//! 0x4B720F, 0x4B7266). The polygon through `part` (1), in the game its Rust version.
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose call goes to a
//! stand-in, against the Rust version, records by lot, at both test control words.

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::batch::{POLYGON_COUNT_RVA, POLYGON_RVA};
use crate::service::harness::{Region, at, capture, restore};
use rc2k_core::hud::quads;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::add_quad", rva(0x4B72D1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const QUAD_RVA: usize = rva(0x75BFB0); // dword_75BFB0 .. dword_75BFE4
const SCALE_RVA: [usize; 2] = [rva(0x75C00C), rva(0x75C010)]; // flt_75C00C, flt_75C010

/// The quad record at each call, as the tests note it (`note_quad`, `take_quads`).
static QUADS: std::sync::Mutex<Vec<(u32, [u32; 14])>> = std::sync::Mutex::new(Vec::new());

/// For `install::stand_ins::set_handler` and the Rust side of a test: the call's place
/// and the quad record noted.
pub fn note_quad(place: u32, _r: &mut Regs) {
    let q: [u32; 14] = unsafe { game_read(QUAD_RVA) };
    QUADS.lock().unwrap().push((place, q));
}

pub fn take_quads() -> Vec<(u32, [u32; 14])> {
    std::mem::take(&mut *QUADS.lock().unwrap())
}

/// A call by place: 1 `add_polygon`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::quads::quad_vertices`
pub unsafe fn add_quad(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let q: [u32; quads::RECORD] = game_read(QUAD_RVA);
        let [a, b] = SCALE_RVA.map(|r| f32::from_bits(game_read(r)));
        let (vertices, colour, low) = quads::quad_vertices(&q, a, b);
        for (n, v) in vertices.into_iter().enumerate() {
            let at = POLYGON_RVA + 0x20 * n;
            game_write(at, [v.x.to_bits(), v.y.to_bits()]);
            game_write(at + 0x10, v.colour);
            game_write(at + 0x18, [v.u.to_bits(), v.v.to_bits()]);
        }
        [g.eax, g.ebx] = [colour, low];
        game_write(POLYGON_COUNT_RVA, 4u32);
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `add_polygon`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::add_polygon as ap;
    *r = unsafe { ap::add_polygon(r, &mut ap::game_part) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { add_quad(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::hud::batch::run_both_floats;
    use crate::race::mud::{copy, test_part};
    use crate::service::floats::FLOAT;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // fild dword_75BFB4
    let entry: &[u8] = &[0xDB, 0x05, 0xB4, 0xBF, 0x75, 0x00];
    let copy = match copy(0x4B72D1, 0x142, entry, &[0x13C], &[(0x4B7413, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![(game_addr(POLYGON_RVA) as u32, 0x80), at(POLYGON_COUNT_RVA)];
    let mut vars = regions.clone();
    vars.push((game_addr(QUAD_RVA) as u32, 0x38));
    vars.extend(SCALE_RVA.map(at));
    let saved = capture(&vars);
    // The texture places (u, v of each vertex).
    let places: Vec<_> = (0..4u32)
        .map(|n| ((game_addr(POLYGON_RVA) as u32 + 0x20 * n + 0x18, 8), FLOAT))
        .collect();
    for case in 0..4u32 {
        run_both_floats(
            &mut t,
            &format!("case {case}"),
            &copy,
            &regions,
            &places,
            &|| unsafe {
                restore(&vars, &saved);
                std::ptr::write_bytes(game_addr(POLYGON_RVA) as *mut u8, 0xAA, 0x80);
                for n in 0..14u32 {
                    game_write(QUAD_RVA + 4 * n as usize, (n * 37 + case * 11) % 300);
                }
                game_write(QUAD_RVA + 0x14, 0x12_3456 * (case + 1));
                game_write(SCALE_RVA[0], 256.0f32.to_bits());
                game_write(SCALE_RVA[1], (1.0f32 + case as f32 * 0.5).to_bits());
            },
            Regs::SENTINEL,
            &|r| unsafe { add_quad(r, &mut test_part) },
        );
    }
    restore(&vars, &saved);
    t.finish()
}
