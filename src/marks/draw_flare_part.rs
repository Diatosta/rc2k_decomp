//! `marks::draw_flare_part` (0x4C4FFB): one part of the lens flare drawn as a screen
//! square. Its centre (st(0) x, st(1) y) rounded into `g_polygonVertices`' fifth vertex
//! (+0x80, +0x84); the four vertices' colour (ecx << 24) | esi; their texture places from
//! corner eax of `g_marksCorners` (its words times `flt_8EB969`: 0/2, 4/6, 0xC/0xE, 8/0xA);
//! the side: edx (`dword_8EB8B5`) by edx times the corner's height over its width (the
//! words' differences, 16-bit, made positive: `dword_8EB8B9`), times `flt_9206DE` /
//! `flt_9206E2` into `flt_8EB941` / `flt_8EB945`; the corners (0, side) less those, turned
//! by `g_sines` / `g_cosines`[ebx >> 20] about the centre; depth `dword_8EB961` (as a
//! float) times `flt_8EB96D`; `dword_8F3981` plus 1; `marks::clip_quad`, and unless all is
//! outside `scene::draw_fan`.
//!
//! In:  st(0), st(1) the centre (popped), eax the corner, ebx the angle << 20, ecx the
//!      alpha, edx the size, esi the colour
//! Out: as `clip_quad` / `draw_fan` leave them (eax the corner's address, ebx the angle,
//!      ecx the colour, edi the height going in)
//!
//! Caller: `marks::draw_lens_flare` (0x4C4FEA). The clip is its Rust version (the copy's
//! call goes to its hook); the draw through `part` (1 `draw_fan`), in the game its Rust
//! version. Refuses a corner past `g_marksCorners` (0x100 or more) and null marks
//! settings (`clip_quad`'s refusal), before anything. Thunk by `two_in_hook!`. Shadow mode
//! runs the original alone (it draws). Self-test: a copy whose draw goes to a stand-in,
//! against the Rust version, centres, corners, sizes and angles by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva, shadow};
use crate::marks::clip_quad::clip_quad;
use crate::math::x87;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::vector3::{Pair80, call_with_two, two_in_hook};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PAIR: Pair80 = Pair80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::draw_flare_part", rva(0x4C4FFB), thunk, &ORIGINAL).self_test(self_test);

two_in_hook!(
    thunk,
    call_original,
    original_with_two,
    dispatch = dispatch,
    original = ORIGINAL,
    pair = PAIR
);

const SCALE: [usize; 2] = [rva(0x9206DE), rva(0x9206E2)]; // flt_9206DE, flt_9206E2
const SIDE_RVA: [usize; 2] = [rva(0x8EB941), rva(0x8EB945)]; // flt_8EB941, flt_8EB945
const SIZE_RVA: [usize; 2] = [rva(0x8EB8B5), rva(0x8EB8B9)]; // dword_8EB8B5, dword_8EB8B9
const CORNERS_RVA: usize = rva(0x8EB971); // g_marksCorners: 0x100 of 0x80 bytes
const CORNERS: u32 = 0x100;
const UV_K_RVA: usize = rva(0x8EB969); // flt_8EB969
const NEAR_RVA: usize = rva(0x8EB961); // dword_8EB961
const Z_K_RVA: usize = rva(0x8EB96D); // flt_8EB96D
const DRAWN_RVA: usize = rva(0x8F3981); // dword_8F3981
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: TLVertex[4], a fifth's x, y
const SINES_RVA: usize = rva(0xA23AE4); // g_sines
const COSINES_RVA: usize = rva(0xA24AE4); // g_cosines
/// Each vertex's texture place: the corner entry's word offsets (u, v).
const UV_WORDS: [(u32, u32); 4] = [(0, 2), (4, 6), (0xC, 0xE), (8, 0xA)];

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// A 16-bit difference made positive (`sub bp, ..; jns; neg bp`), the high half 0.
fn abs16(a: u32, b: u32) -> u32 {
    let d = (a as u16).wrapping_sub(b as u16);
    (if (d as i16) < 0 { d.wrapping_neg() } else { d }) as u32
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_flare_part(
    entry: &Regs,
    centre: [[u8; 10]; 2],
    part: Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.eax >= CORNERS {
        return Err(BadInput("a flare corner past g_marksCorners").into());
    }
    unsafe {
        if game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for n in 0..2 {
            game_write(SIDE_RVA[n], x87::loaded(game_read(SCALE[n])));
        }
        g.ecx = (g.ecx << 24) | g.esi;
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, g.ecx);
        }
        g.eax = game_addr(CORNERS_RVA) as u32 + g.eax * 0x80;
        let word = |off: u32| ((g.eax + off) as *const u16).read_unaligned() as u32;
        let uv_k: u32 = game_read(UV_K_RVA);
        for (n, (u, v)) in UV_WORDS.into_iter().enumerate() {
            let vertex = VERTICES_RVA + 0x20 * n;
            game_write(vertex + 0x18, x87::int_times(word(u) as i32, uv_k));
            game_write(vertex + 0x1C, x87::int_times(word(v) as i32, uv_k));
        }
        let [cx, cy] = centre.map(rounded);
        game_write(VERTICES_RVA + 0x80, cx);
        game_write(VERTICES_RVA + 0x84, cy);
        g.ebp = abs16(word(4), word(0));
        g.edi = abs16(word(0xA), word(2));
        let ratio = x87::int_over_int(g.edi as i32, g.ebp as i32);
        let width = x87::int_to_float(g.edx as i32);
        let height = x87::int_times(g.edx as i32, ratio);
        game_write(SIZE_RVA[0], width);
        game_write(SIZE_RVA[1], height);
        g.ebx >>= 20;
        let sw = x87::mul(width, game_read(SIDE_RVA[0]));
        let sh = x87::mul(height, game_read(SIDE_RVA[1]));
        game_write(SIDE_RVA[0], sw);
        game_write(SIDE_RVA[1], sh);
        let corners = [
            (x87::sub(0, sw), x87::sub(0, sh)),
            (x87::sub(width, sw), x87::sub(0, sh)),
            (x87::sub(width, sw), x87::sub(height, sh)),
            (x87::sub(0, sw), x87::sub(height, sh)),
        ];
        let sin: u32 = game_read(SINES_RVA + 4 * g.ebx as usize);
        let cos: u32 = game_read(COSINES_RVA + 4 * g.ebx as usize);
        let depth = x87::mul(game_read(NEAR_RVA), game_read(Z_K_RVA));
        for (n, (x, y)) in corners.into_iter().enumerate() {
            let vertex = VERTICES_RVA + 0x20 * n;
            let [tx, ty] = x87::turn_about(x, y, sin, cos, cx, cy);
            game_write(vertex, tx);
            game_write(vertex + 4, ty);
            game_write(vertex + 8, depth);
        }
        game_write(DRAWN_RVA, game_read::<u32>(DRAWN_RVA).wrapping_add(1));
        g.ebp = game_addr(VERTICES_RVA) as u32;
        g = clip_quad(&g)?;
        if g.carry() {
            return Ok(g);
        }
        part(1, &mut g)?;
    }
    Ok(g)
}

/// In the game: `draw_fan`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe { crate::scene::draw_fan::draw_fan(r, &mut crate::scene::draw_fan::game_part) };
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { original_with_two(r) };
        return;
    }
    match unsafe { draw_flare_part(r, PAIR.inputs(), &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, original_with_two) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(crate::marks::setup::SETTINGS_RVA) } == 0 {
        return t.skip("no marks settings (the clip reads them)");
    }
    // fld flt_9206DE
    let entry: &[u8] = &[0xD9, 0x05, 0xDE, 0x06, 0x92, 0x00];
    let copy = match copy(0x4C4FFB, 0x39D, entry, &[0x390, 0x397], &[(0x44EA65, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (game_addr(SIDE_RVA[0]) as u32, 8),
        (game_addr(SIZE_RVA[0]) as u32, 8),
        (game_addr(VERTICES_RVA) as u32, 0xC0),
        at(DRAWN_RVA),
        at(rva(0x6240F8)),
        (game_addr(rva(0x7687F4)) as u32, 6 * 0x20),
        at(rva(0x93CC94)),
    ];
    let mut vars = regions.clone();
    vars.extend([NEAR_RVA, Z_K_RVA, UV_K_RVA, SCALE[0], SCALE[1]].map(at));
    let corner_base = game_addr(CORNERS_RVA) as u32;
    vars.push((corner_base + 0x10 * 0x80, 4 * 0x80));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..6u32 {
        let k = case as f32;
        let centre = [
            extended((100.0 + k * 130.0).to_bits()),
            extended((50.0 + k * 77.5).to_bits()),
        ];
        let before = Regs {
            eax: 0x10 + (case % 4),
            ebx: (case * 0x123) << 20,
            ecx: 0x40 + case * 30,
            edx: 8 + case * 13,
            esi: 0x00FF_C080,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            game_write(NEAR_RVA, 2.0f32.to_bits());
            game_write(Z_K_RVA, (1.0f32 / 3000.0).to_bits());
            game_write(UV_K_RVA, (1.0f32 / 256.0).to_bits());
            game_write(SCALE[0], (0.5f32 + k * 0.1).to_bits());
            game_write(SCALE[1], (0.5f32 - k * 0.05).to_bits());
            for n in 0..4 * 0x40u32 {
                let w = (n * 29 + case * 7) % 0x120;
                ((corner_base + 0x10 * 0x80 + 2 * n) as *mut u16).write_unaligned(w as u16);
            }
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0xC0);
        };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { call_with_two(&mut ro, copy.addr(), &centre) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_flare_part(&before, centre, &mut |p, r| {
                note(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_called(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!(
                "case {case}: original {:x?} {:x?}; rust {:x?} {:x?}; state first differs at {:?}",
                theirs.0,
                theirs.1,
                mine.0,
                mine.1,
                theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b)
            )
        });
    }
    restore(&vars, &saved);
    t.finish()
}
