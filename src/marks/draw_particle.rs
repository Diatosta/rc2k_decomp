//! `marks::draw_particle` (0x4CFDD5): one particle of the marks' ring drawn as a square
//! sprite. Its place (st(0) x, st(1) y, st(2) z) turned by `g_viewMatrix` read by columns
//! into `flt_8EB8D5` .. `flt_8EB8DD`; unless the turned depth is at or below
//! `dword_8EB961` (a signed compare of the bits): the four vertices' colour ecx and depth
//! (times `flt_8EB96D`), half the square (edx times `flt_624088` / depth) in
//! `dword_8EB8B5` and `dword_8EB8B9`, the corners around the projected place (centre
//! `flt_62409C`, `flt_6240A0`), and `scene::draw_fan`. The vertices' rhw and texture places
//! are left as they are (`marks::set_quad_uvs` sets the places once a frame).
//!
//! In:  st(0) .. st(2) the place (popped), ecx the colour, edx the size
//! Out: ebp the turned depth's bits; the rest as `draw_fan` leaves them when drawn
//!
//! Callers: the ring's update (code at 0x4D03E7, 0x4D04BE, 0x4D05AC). Its call through
//! `part` (1 `draw_fan`); in the game its Rust version (the device's through the game).
//! Thunk by `three_in_hook!`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose call goes to a stand-in, against the Rust version on the same, places near
//! and far, sizes and colours by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva, shadow};
use crate::math::x87;
use crate::scene::matrix3::extended;
use crate::scene::vector3::{Triple80, call_with_three, three_in_hook};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static TRIPLE: Triple80 = Triple80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::draw_particle", rva(0x4CFDD5), thunk, &ORIGINAL).self_test(self_test);

three_in_hook!(
    thunk,
    call_original,
    original_with_three,
    dispatch = dispatch,
    original = ORIGINAL,
    triple = TRIPLE
);

const VIEW_RVA: usize = rva(0x5EA2CC); // g_viewMatrix
const TURNED_RVA: usize = rva(0x8EB8D5); // flt_8EB8D5 .. flt_8EB8DD
const NEAR_RVA: usize = rva(0x8EB961); // dword_8EB961
const HALF_RVA: usize = rva(0x8EB8B5); // dword_8EB8B5, dword_8EB8B9
const FOCAL_RVA: usize = rva(0x624088); // flt_624088
const CENTRE_RVA: [usize; 2] = [rva(0x62409C), rva(0x6240A0)];
const Z_K_RVA: usize = rva(0x8EB96D); // flt_8EB96D
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices: TLVertex[4]

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_particle(
    entry: &Regs,
    place: [[u8; 10]; 3],
    part: Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let turned = x87::column_turn(&place, game_addr(VIEW_RVA) as u32);
        for (n, v) in turned.into_iter().enumerate() {
            game_write(TURNED_RVA + 4 * n, v);
        }
        let [a, b, c] = turned;
        g.ebp = c;
        if (c as i32) <= game_read::<i32>(NEAR_RVA) {
            return Ok(g);
        }
        let k = [
            game_read(FOCAL_RVA),
            game_read(CENTRE_RVA[0]),
            game_read(CENTRE_RVA[1]),
            game_read(Z_K_RVA),
        ];
        let sq = x87::particle_square(a, b, c, k, g.edx as i32);
        for n in 0..4 {
            let v = VERTICES_RVA + 0x20 * n;
            game_write(v + 0x10, g.ecx);
            game_write(v + 8, sq[1]);
            game_write(v, sq[6 + n]);
            game_write(v + 4, sq[2 + n]);
        }
        game_write(HALF_RVA, sq[0]);
        game_write(HALF_RVA + 4, sq[0]);
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
        unsafe { original_with_three(r) };
        return;
    }
    match unsafe { draw_particle(r, TRIPLE.inputs(), &mut game_part) } {
        Ok(left) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..left
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, original_with_three) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // fld g_viewMatrix
    let entry: &[u8] = &[0xD9, 0x05, 0xCC, 0xA2, 0x5E, 0x00];
    let copy = match copy(0x4CFDD5, 0x182, entry, &[0x17C], &[(0x44EA65, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let regions: Vec<Region> = vec![
        (game_addr(TURNED_RVA) as u32, 12),
        (game_addr(HALF_RVA) as u32, 8),
        (game_addr(VERTICES_RVA) as u32, 0x80),
    ];
    let mut vars = regions.clone();
    vars.extend([NEAR_RVA, FOCAL_RVA, CENTRE_RVA[0], CENTRE_RVA[1], Z_K_RVA].map(at));
    vars.push((game_addr(VIEW_RVA) as u32, 0x40));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..6u32 {
        let k = case as f32;
        let place = [
            extended((3.0 - k * 1.5).to_bits()),
            extended((0.25 * k).to_bits()),
            extended((if case == 2 { -50.0 } else { 40.0 + k * 9.0 }).to_bits()),
        ];
        let before = Regs {
            ecx: 0x8011_2233u32.rotate_left(case),
            edx: 3 + case * 17,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            for n in 0..16 {
                let v = if n % 5 == 0 {
                    0.9
                } else {
                    (n as f32 - 7.0) * 0.05 + k * 0.01
                };
                game_write(VIEW_RVA + 4 * n, f32::to_bits(v));
            }
            game_write(VIEW_RVA + 0x38, 5.0f32.to_bits());
            game_write(NEAR_RVA, 1.0f32.to_bits());
            game_write(FOCAL_RVA, 320.0f32.to_bits());
            game_write(CENTRE_RVA[0], 320.5f32.to_bits());
            game_write(CENTRE_RVA[1], 240.25f32.to_bits());
            game_write(Z_K_RVA, (1.0f32 / 3000.0).to_bits());
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0x80);
        };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { call_with_three(&mut ro, copy.addr(), &place) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_particle(&before, place, &mut |p, r| {
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
