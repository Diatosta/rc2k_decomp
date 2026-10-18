//! `marks::draw_pong_box` (0x4B8599): one box of the pong game (ball, bat or a score
//! cell) drawn on the squares' frame: its height (st(0)) along the second axis times
//! `flt_89CEF2` into `flt_768664` .. (and times `flt_765158` into `flt_76867C` ..), its
//! width (st(1)) along the first times `flt_89CEEE` into `flt_768658` .. (`flt_768670` ..);
//! the texture places from `dword_76920A` ..; its place (st(2) along, st(3) across) on the
//! frame into `flt_89CF92` .., the four corners; unless the first corner's depth is below
//! `flt_76921A`: projected, depth times `flt_89CEEA`, colour -1, `scene::draw_fan`.
//!
//! In:  st(0) height, st(1) width, st(2) along, st(3) across (popped)
//! Out: eax the depth compare's status (& 0x700), ebx .. edx the places b .. d, ebp
//!      `g_polygonVertices`; the rest as `draw_fan` leaves them when drawn
//!
//! Callers: `marks::run_pong` (ball, bat), `marks::pong_cell`. The x87 runs are the
//! original's instructions over the game's variables; the draw through `part` (1
//! `draw_fan`), in the game its Rust version. Thunk by `four_in_hook!`. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose draw goes to a stand-in,
//! against the Rust version, boxes near and far.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva, shadow};
use crate::math::x87;
use crate::scene::matrix3::extended;
use crate::scene::vector3::{Four80, call_with_four, four_in_hook};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static FOUR: Four80 = Four80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::draw_pong_box", rva(0x4B8599), thunk, &ORIGINAL).self_test(self_test);

four_in_hook!(
    thunk,
    call_original,
    original_with_four,
    dispatch = dispatch,
    original = ORIGINAL,
    four = FOUR
);

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: usize = rva(0x76920A); // dword_76920A .. dword_769216
const SIDES_RVA: usize = rva(0x768658); // flt_768658 .. flt_768684
const CORNERS_RVA: usize = rva(0x89CF92); // flt_89CF92 .. flt_89CFBE
const NEAR_RVA: usize = rva(0x76921A); // flt_76921A

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The original's 0x4B8599 .. 0x4B8659 on the four values: the sides; the place's two
/// values (left untouched) popped after.
unsafe fn sides(b: u32, v: &[[u8; 10]; 4]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 30]",
            "fld tbyte ptr [{v} + 20]",
            "fld tbyte ptr [{v} + 10]",
            "fld tbyte ptr [{v}]",
            "fld dword ptr [{b} + 0x49CF26]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fst dword ptr [{b} + 0x368664]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x36867C]",
            "fld dword ptr [{b} + 0x49CF2A]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fst dword ptr [{b} + 0x368668]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368680]",
            "fld dword ptr [{b} + 0x49CF2E]",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{b} + 0x49CEF2]",
            "fst dword ptr [{b} + 0x36866C]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368684]",
            "fld dword ptr [{b} + 0x49CF1A]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fst dword ptr [{b} + 0x368658]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368670]",
            "fld dword ptr [{b} + 0x49CF1E]",
            ".byte 0xD8, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fst dword ptr [{b} + 0x36865C]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368674]",
            "fld dword ptr [{b} + 0x49CF22]",
            ".byte 0xDE, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            "fst dword ptr [{b} + 0x368660]",
            "fmul dword ptr [{b} + 0x365158]",
            "fstp dword ptr [{b} + 0x368678]",
            "fstp st(0)",
            "fstp st(0)",
            b = in(reg) b,
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4B868E .. 0x4B8786 on the place: the corners.
unsafe fn corners(b: u32, v: &[[u8; 10]; 4]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 30]",
            "fld tbyte ptr [{v} + 20]",
            "fmul dword ptr [{b} + 0x49CEF2]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CEEE]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF1A]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF26]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEF6]",
            "fstp dword ptr [{b} + 0x49CF92]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF1E]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{b} + 0x49CF2A]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEFA]",
            "fstp dword ptr [{b} + 0x49CF96]",
            "fmul dword ptr [{b} + 0x49CF2E]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CF22]",
            ".byte 0xDE, 0xC1",
            "fadd dword ptr [{b} + 0x49CEFE]",
            "fstp dword ptr [{b} + 0x49CF9A]",
            "fld dword ptr [{b} + 0x49CF92]",
            "fst dword ptr [{b} + 0x49CF92]",
            "fadd dword ptr [{b} + 0x368664]",
            "fst dword ptr [{b} + 0x49CF9E]",
            "fadd dword ptr [{b} + 0x368658]",
            "fst dword ptr [{b} + 0x49CFAA]",
            "fsub dword ptr [{b} + 0x368664]",
            "fstp dword ptr [{b} + 0x49CFB6]",
            "fld dword ptr [{b} + 0x49CF96]",
            "fst dword ptr [{b} + 0x49CF96]",
            "fadd dword ptr [{b} + 0x368668]",
            "fst dword ptr [{b} + 0x49CFA2]",
            "fadd dword ptr [{b} + 0x36865C]",
            "fst dword ptr [{b} + 0x49CFAE]",
            "fsub dword ptr [{b} + 0x368668]",
            "fstp dword ptr [{b} + 0x49CFBA]",
            "fld dword ptr [{b} + 0x49CF9A]",
            "fst dword ptr [{b} + 0x49CF9A]",
            "fadd dword ptr [{b} + 0x36866C]",
            "fst dword ptr [{b} + 0x49CFA6]",
            "fadd dword ptr [{b} + 0x368660]",
            "fst dword ptr [{b} + 0x49CFB2]",
            "fsub dword ptr [{b} + 0x36866C]",
            "fstp dword ptr [{b} + 0x49CFBE]",
            b = in(reg) b,
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4B87A5 .. 0x4B88A9: the corners projected, their depth.
unsafe fn projected(b: u32, p: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x49CFBE]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x49CFB2]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x49CFA6]",
            "fld dword ptr [{b} + 0x224088]",
            "fdiv dword ptr [{b} + 0x49CF9A]",
            "fld dword ptr [{b} + 0x49CF96]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CF92]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x4]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x0]",
            "fld dword ptr [{b} + 0x49CFA2]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CF9E]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x24]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x20]",
            "fld dword ptr [{b} + 0x49CFAE]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CFAA]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x44]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x40]",
            "fld dword ptr [{b} + 0x49CFBA]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{b} + 0x49CFB6]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{b} + 0x2240A0]",
            "fstp dword ptr [{p} + 0x64]",
            "fadd dword ptr [{b} + 0x22409C]",
            "fstp dword ptr [{p} + 0x60]",
            "fld dword ptr [{b} + 0x49CF9A]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFA6]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFB2]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFBE]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fstp dword ptr [{p} + 0x68]",
            "fstp dword ptr [{p} + 0x48]",
            "fstp dword ptr [{p} + 0x28]",
            "fstp dword ptr [{p} + 0x8]",
            b = in(reg) b,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_pong_box(entry: &Regs, v: [[u8; 10]; 4], part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let b = game_addr(0) as u32;
        sides(b, &v);
        g.ebp = game_addr(VERTICES_RVA) as u32;
        let [pa, pb, pc, pd]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, w)) in [(pa, pb), (pc, pb), (pc, pd), (pa, pd)]
            .into_iter()
            .enumerate()
        {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, w);
        }
        [g.eax, g.ebx, g.ecx, g.edx] = [pa, pb, pc, pd];
        corners(b, &v);
        g.eax = x87::compare_status(game_read(CORNERS_RVA + 8), game_read(NEAR_RVA)) as u32;
        if g.eax == 0x100 {
            return Ok(g);
        }
        projected(b, g.ebp);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, 0xFFFF_FFFFu32);
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
        unsafe { original_with_four(r) };
        return;
    }
    match unsafe { draw_pong_box(r, FOUR.inputs(), &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, original_with_four) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::race::level_init::{note, take_called};
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // fld flt_89CF26
    let entry: &[u8] = &[0xD9, 0x05, 0x26, 0xCF, 0x89, 0x00];
    let copy = match copy(0x4B8599, 0x332, entry, &[0x32C], &[(0x44EA65, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let frame = (game_addr(rva(0x89CEEE)) as u32, 0x44);
    let regions: Vec<Region> = vec![
        (game_addr(VERTICES_RVA) as u32, 0x80),
        (game_addr(CORNERS_RVA) as u32, 0x30),
        (game_addr(SIDES_RVA) as u32, 0x30),
    ];
    let mut vars = regions.clone();
    vars.push(frame);
    vars.push((game_addr(PLACES_RVA) as u32, 16));
    vars.extend(
        [
            rva(0x624088),
            rva(0x62409C),
            rva(0x6240A0),
            rva(0x765158),
            NEAR_RVA,
        ]
        .map(at),
    );
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let k = case as f32;
        let v = [
            extended((6.0f32 + k).to_bits()),
            extended((8.0f32 - k).to_bits()),
            extended((100.0f32 + k * 30.0).to_bits()),
            extended((290.0f32 - k * 11.0).to_bits()),
        ];
        let prepare = || unsafe {
            restore(&vars, &saved);
            let f = frame.0 as *mut f32;
            // flt_89CEEE .. flt_89CF2E: shares, place, the axes, the third axis
            for n in 0..(0x44 / 4) {
                *f.add(n) = ((n as f32) * 0.41 - 2.0 + k * 0.1).sin();
            }
            // flt_89CEFE: the place's depth
            *f.add(4) = 500.0 + if case == 3 { -900.0 } else { k * 40.0 };
            for n in 0..4 {
                game_write(
                    PLACES_RVA + 4 * n,
                    ((n as u32 + case) as f32 * 0.25).to_bits(),
                );
            }
            game_write(rva(0x624088), 320.0f32.to_bits());
            game_write(rva(0x62409C), 320.5f32.to_bits());
            game_write(rva(0x6240A0), 240.25f32.to_bits());
            game_write(rva(0x765158), 0.5f32.to_bits());
            game_write(NEAR_RVA, 1.0f32.to_bits());
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0x80);
        };
        prepare();
        take_called();
        let mut ro = Regs::SENTINEL;
        unsafe { call_with_four(&mut ro, copy.addr(), &v) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_pong_box(&Regs::SENTINEL, v, &mut |p, r| {
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
