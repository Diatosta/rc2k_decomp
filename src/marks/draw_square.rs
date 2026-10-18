//! `marks::draw_square` (0x4BF80F): one square item drawn on the squares' frame. The four
//! vertices' texture places from `dword_8BC1AE` .. (u a, c, c, a; v b, b, d, d) plus the
//! item's +0xC / +0x10; its place (st(1) the across share times `flt_89CEEE`, st(0) the
//! along share times `flt_89CEF2`) along the two axes from the frame's place into
//! `flt_89CF92` ..; less `flt_89CF56` .. and `flt_89CF62` ..; the four corners (plus
//! `flt_89CF4A` .., `flt_89CF3E` .., less `flt_89CF4A` ..) into `flt_89CF92` .. `flt_89CFBE`;
//! unless the first corner's depth is at or below 4096.0 (a signed compare of the bits):
//! projected (`flt_624088` over each depth, the centre `flt_62409C` / `flt_6240A0`), depth
//! (plus `g_marksKindFloat`, times `flt_89CEEA`), colour edx, `scene::draw_fan`.
//!
//! In:  st(0) the along share, st(1) the across share (popped), esi the item, edx the
//!      colour
//! Out: eax, ebx, ecx the places a, b, c; ebp `g_polygonVertices`; the rest as `draw_fan`
//!      leaves them when drawn
//!
//! Caller: `sub_4BFFEC` (0x4C00FE). The x87 runs are the original's instructions over the
//! game's variables; the draw through `part` (1 `draw_fan`), in the game its Rust version.
//! Thunk by `two_in_hook!`. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose draw goes to a stand-in, against the Rust version, items near and far.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, rva, shadow};
use crate::scene::matrix3::extended;
use crate::scene::vector3::{Pair80, call_with_two, two_in_hook};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PAIR: Pair80 = Pair80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::draw_square", rva(0x4BF80F), thunk, &ORIGINAL).self_test(self_test);

two_in_hook!(
    thunk,
    call_original,
    original_with_two,
    dispatch = dispatch,
    original = ORIGINAL,
    pair = PAIR
);

const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const PLACES_RVA: usize = rva(0x8BC1AE); // dword_8BC1AE .. dword_8BC1BA
const CORNERS_RVA: usize = rva(0x89CF92); // flt_89CF92 .. flt_89CFBE: four of 12
const FAR_ENOUGH: i32 = 0x4580_0000; // 4096.0

/// A call by place: 1 `draw_fan`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The original's 0x4BF845 .. 0x4BF88D: the item's texture shift added to the places.
unsafe fn shifted_places(s: u32, p: u32) {
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x18]",
            "fstp dword ptr [{p} + 0x18]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x38]",
            "fstp dword ptr [{p} + 0x38]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x58]",
            "fstp dword ptr [{p} + 0x58]",
            "fld dword ptr [{s} + 0xC]",
            "fadd dword ptr [{p} + 0x78]",
            "fstp dword ptr [{p} + 0x78]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x1C]",
            "fstp dword ptr [{p} + 0x1C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x3C]",
            "fstp dword ptr [{p} + 0x3C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x5C]",
            "fstp dword ptr [{p} + 0x5C]",
            "fld dword ptr [{s} + 0x10]",
            "fadd dword ptr [{p} + 0x7C]",
            "fstp dword ptr [{p} + 0x7C]",
            s = in(reg) s,
            p = in(reg) p,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF88E .. 0x4BF9AA on the two values: the corners.
unsafe fn corners(b: u32, v: &[[u8; 10]; 2]) {
    unsafe {
        core::arch::asm!(
            "fld tbyte ptr [{v} + 10]",
            "fld tbyte ptr [{v}]",
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
            "fsub dword ptr [{b} + 0x49CF56]",
            "fsub dword ptr [{b} + 0x49CF62]",
            "fst dword ptr [{b} + 0x49CF92]",
            "fadd dword ptr [{b} + 0x49CF4A]",
            "fst dword ptr [{b} + 0x49CF9E]",
            "fadd dword ptr [{b} + 0x49CF3E]",
            "fst dword ptr [{b} + 0x49CFAA]",
            "fsub dword ptr [{b} + 0x49CF4A]",
            "fstp dword ptr [{b} + 0x49CFB6]",
            "fld dword ptr [{b} + 0x49CF96]",
            "fsub dword ptr [{b} + 0x49CF5A]",
            "fsub dword ptr [{b} + 0x49CF66]",
            "fst dword ptr [{b} + 0x49CF96]",
            "fadd dword ptr [{b} + 0x49CF4E]",
            "fst dword ptr [{b} + 0x49CFA2]",
            "fadd dword ptr [{b} + 0x49CF42]",
            "fst dword ptr [{b} + 0x49CFAE]",
            "fsub dword ptr [{b} + 0x49CF4E]",
            "fstp dword ptr [{b} + 0x49CFBA]",
            "fld dword ptr [{b} + 0x49CF9A]",
            "fsub dword ptr [{b} + 0x49CF5E]",
            "fsub dword ptr [{b} + 0x49CF6A]",
            "fst dword ptr [{b} + 0x49CF9A]",
            "fadd dword ptr [{b} + 0x49CF52]",
            "fst dword ptr [{b} + 0x49CFA6]",
            "fadd dword ptr [{b} + 0x49CF46]",
            "fst dword ptr [{b} + 0x49CFB2]",
            "fsub dword ptr [{b} + 0x49CF52]",
            "fstp dword ptr [{b} + 0x49CFBE]",
            b = in(reg) b,
            v = in(reg) v.as_ptr(),
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    }
}

/// The original's 0x4BF9BA .. 0x4BFAD6: the corners projected, their depth.
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
            "fadd dword ptr [{b} + 0x36529C]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFA6]",
            "fadd dword ptr [{b} + 0x36529C]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFB2]",
            "fadd dword ptr [{b} + 0x36529C]",
            "fmul dword ptr [{b} + 0x49CEEA]",
            "fld dword ptr [{b} + 0x49CFBE]",
            "fadd dword ptr [{b} + 0x36529C]",
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
pub unsafe fn draw_square(
    entry: &Regs,
    shares: [[u8; 10]; 2],
    part: Part,
) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let b = game_addr(0) as u32;
        g.ebp = game_addr(VERTICES_RVA) as u32;
        let [a, pb, c, d]: [u32; 4] = game_read(PLACES_RVA);
        for (n, (u, v)) in [(a, pb), (c, pb), (c, d), (a, d)].into_iter().enumerate() {
            game_write(VERTICES_RVA + 0x20 * n + 0x18, u);
            game_write(VERTICES_RVA + 0x20 * n + 0x1C, v);
        }
        [g.eax, g.ebx, g.ecx] = [a, pb, c];
        shifted_places(g.esi, g.ebp);
        corners(b, &shares);
        if game_read::<i32>(CORNERS_RVA + 8) <= FAR_ENOUGH {
            return Ok(g);
        }
        projected(b, g.ebp);
        for n in 0..4 {
            game_write(VERTICES_RVA + 0x20 * n + 0x10, g.edx);
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
    match unsafe { draw_square(r, PAIR.inputs(), &mut game_part) } {
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
    // lea ebp, g_polygonVertices
    let entry: &[u8] = &[0x8D, 0x2D, 0xA0, 0x05, 0x96, 0x00];
    let copy = match copy(0x4BF80F, 0x2D9, entry, &[0x2D3], &[(0x44EA65, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let frame = (game_addr(rva(0x89CEEE)) as u32, 0x7C);
    let regions: Vec<Region> = vec![
        (game_addr(VERTICES_RVA) as u32, 0x80),
        (game_addr(CORNERS_RVA) as u32, 0x30),
    ];
    let mut vars = regions.clone();
    vars.push(frame);
    vars.push((game_addr(PLACES_RVA) as u32, 16));
    vars.extend(
        [
            rva(0x624088),
            rva(0x62409C),
            rva(0x6240A0),
            rva(0x76529C),
            rva(0x89CEEA),
        ]
        .map(at),
    );
    let saved = capture(&vars);
    let mut item = [0u32; 0x3C / 4];
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..5u32 {
        let k = case as f32;
        item[3] = (0.125 * k).to_bits();
        item[4] = (0.5 - 0.0625 * k).to_bits();
        let ia = item.as_ptr() as u32;
        let shares = [
            extended((300.0f32 - k * 70.0).to_bits()),
            extended((40.0f32 + k * 13.0).to_bits()),
        ];
        let before = Regs {
            esi: ia,
            edx: 0x7F00_FF00u32.rotate_left(case),
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            let f = frame.0 as *mut f32;
            // flt_89CEEE .. flt_89CF6A: shares, place, the axes, the corner offsets
            for n in 0..(0x7C / 4) {
                *f.add(n) = ((n as f32) * 0.37 - 3.0 + k * 0.1).sin() * 2.0;
            }
            // flt_89CEFE: the place's depth
            *f.add(4) = 5000.0 + if case == 3 { -9000.0 } else { k * 300.0 };
            for n in 0..4 {
                game_write(
                    PLACES_RVA + 4 * n,
                    ((n as u32 + case) as f32 * 0.25).to_bits(),
                );
            }
            game_write(rva(0x624088), 320.0f32.to_bits());
            game_write(rva(0x62409C), 320.5f32.to_bits());
            game_write(rva(0x6240A0), 240.25f32.to_bits());
            game_write(rva(0x76529C), 1.5f32.to_bits());
            game_write(rva(0x89CEEA), (1.0f32 / 3000.0).to_bits());
            std::ptr::write_bytes(game_addr(VERTICES_RVA) as *mut u8, 0xAA, 0x80);
        };
        prepare();
        take_called();
        let mut ro = before;
        unsafe { call_with_two(&mut ro, copy.addr(), &shares) };
        let theirs = (Some(regs(&ro)), take_called(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_square(&before, shares, &mut |p, r| {
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
