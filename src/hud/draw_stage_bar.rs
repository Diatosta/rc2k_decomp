//! `hud::draw_stage_bar` (0x4B5A86): the stage's progress bar along the top: the quad
//! record's alpha 0xE0; a frame of five solid quads (0x212463, the last black); then for
//! each of the record's +0x74 parts: its colour (`dword_75BF70` at the byte offset
//! `dword_6D160C`[n]) into `dword_75BFA0`, its end (`dword_75BFFC`, the sum of
//! `g_overlayPartMiles` .., the last the record's +0x68) and start (`dword_75BFF8`) as
//! places across 0x229 (`fistp` into `dword_75BFF0` / `dword_75BFF4`); a black border of
//! four quads, the part's colour inside under alpha 0x78, and between parts a 0x212463
//! line.
//!
//! In:  ebp the HUD's record
//! Out: ebx the parts drawn (kept with none: the original zeroes it after the test), ecx 0;
//!      the rest kept
//!
//! Caller: layout 0's drawer (0x4B1DC7). The quads through `part` (1 `add_solid_quad`),
//! in the game its Rust version. Refuses more than 0x40 parts and a colour offset past
//! `dword_75BF70`'s four, before anything. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose quads go to a stand-in noting the quad record at each,
//! against the Rust version, parts by lot.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::add_quad::{QUAD_RVA, note_quad, take_quads};
use crate::math::x87;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_stage_bar", rva(0x4B5A86), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const MILES_RVA: usize = rva(0x6D150C); // g_overlayPartMiles: 0x40
pub const KINDS_RVA: usize = rva(0x6D160C); // dword_6D160C: 0x40
pub const COLOURS_RVA: usize = rva(0x75BF70); // dword_75BF70: 4
const COLOUR_RVA: usize = rva(0x75BFA0); // dword_75BFA0
pub const PLACES_RVA: [usize; 2] = [rva(0x75BFF0), rva(0x75BFF4)];
pub const ENDS_RVA: [usize; 2] = [rva(0x75BFF8), rva(0x75BFFC)]; // start, end
pub const WIDTH_RVA: usize = rva(0x75C000); // dword_75C000
pub const PARTS: u32 = 0x40;
pub const FRAME: u32 = 0x21_2463;

/// A call by place: 1 `add_solid_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// `fild a; fmul b; fdiv c; fistp`.
pub fn int_times_over_to_int(a: u32, b: u32, c: u32) -> u32 {
    let (v, mut out) = ([a, b, c], 0u32);
    unsafe {
        core::arch::asm!("fild dword ptr [{v}]", "fmul dword ptr [{v} + 4]", "fdiv dword ptr [{v} + 8]", "fistp dword ptr [{o}]",
            v = in(reg) v.as_ptr(), o = in(reg) &raw mut out, out("st(0)") _, options(nostack))
    };
    out
}

pub fn set(n: usize, v: u32) {
    unsafe { game_write(QUAD_RVA + 4 * n, v) };
}

/// The record's corners (left, top, right, bottom) and, with `Some`, its colour; a quad.
pub fn quad(g: &Regs, colour: Option<u32>, c: [u32; 4], part: &mut Part) -> Result<(), Refused> {
    if let Some(colour) = colour {
        set(5, colour);
    }
    for (n, v) in c.into_iter().enumerate() {
        set(1 + n, v);
    }
    let mut r = *g;
    part(1, &mut r)
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_stage_bar(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut part = part;
    let mut g = *entry;
    unsafe {
        let get = |o: u32| ((g.ebp + o) as *const u32).read_unaligned();
        let count = get(0x74);
        if count > PARTS {
            return Err(BadInput("a stage bar of more than 0x40 parts").into());
        }
        if (0..count).any(|n| game_read::<u32>(KINDS_RVA + 4 * n as usize) > 0xC) {
            return Err(BadInput("a stage bar colour past dword_75BF70").into());
        }
        set(0, 0xE0);
        for (colour, c) in [
            (FRAME, [0, 0, 0x89, 0x20]),
            (FRAME, [0x2AF, 0, 0x320, 0x20]),
            (FRAME, [0x89, 0, 0x2AF, 3]),
            (FRAME, [0x89, 0x1D, 0x2AF, 0x20]),
            (0, [0, 0x1F, 0x320, 0x20]),
        ] {
            quad(&g, Some(colour), c, &mut part)?;
        }
        g.ecx = count;
        if count == 0 {
            return Ok(g);
        }
        game_write(WIDTH_RVA, 0x229u32);
        game_write(ENDS_RVA[0], 0u32);
        game_write(ENDS_RVA[1], 0u32);
        let total = get(0x68);
        for n in 0..count {
            let left = count - n;
            let kind: u32 = game_read(KINDS_RVA + 4 * n as usize);
            let colour = ((game_addr(COLOURS_RVA) as u32 + kind) as *const u32).read_unaligned();
            game_write(COLOUR_RVA, colour);
            game_write(
                ENDS_RVA[1],
                x87::add(
                    game_read(MILES_RVA + 4 * n as usize),
                    game_read(ENDS_RVA[1]),
                ),
            );
            if left == 1 {
                game_write(ENDS_RVA[1], x87::loaded(total));
            }
            let w: u32 = game_read(WIDTH_RVA);
            let a = int_times_over_to_int(w, game_read(ENDS_RVA[0]), total);
            game_write(PLACES_RVA[0], a);
            let b = int_times_over_to_int(w, game_read(ENDS_RVA[1]), total);
            game_write(PLACES_RVA[1], b);
            let [x0, x1] = [a.wrapping_add(0x89), b.wrapping_add(0x86)];
            let r = Regs {
                eax: x0,
                ebx: x1,
                ecx: left,
                ..g
            };
            set(0, 0xE0);
            quad(&r, Some(0), [x0, 3, x1, 4], &mut part)?;
            quad(&r, None, [x1, 3, x1 + 1, 0x1C], &mut part)?;
            quad(&r, None, [x0, 4, x0 + 1, 0x1D], &mut part)?;
            quad(&r, None, [x0 + 1, 0x1C, x1 + 1, 0x1D], &mut part)?;
            let [x0, x1] = [x0 + 1, x1 - 1];
            set(0, 0x78);
            let r = Regs {
                eax: x0,
                ebx: x1,
                edx: colour,
                ..r
            };
            quad(&r, Some(colour), [x0, 4, x1 + 1, 0x1C], &mut part)?;
            let [x0, x1] = [x1 + 2, x1 + 3];
            if left != 1 {
                set(0, 0xE0);
                let r = Regs {
                    eax: x0,
                    ebx: x1,
                    ..r
                };
                quad(&r, Some(FRAME), [x0, 3, x1 + 1, 0x1D], &mut part)?;
            }
            game_write(ENDS_RVA[0], x87::loaded(game_read(ENDS_RVA[1])));
        }
        g.ebx = count;
        g.ecx = 0;
    }
    Ok(g)
}

/// In the game: `add_solid_quad`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::add_solid_quad as sq;
    *r = unsafe { sq::add_solid_quad(r, &mut crate::hud::add_triangle::game_part_quad) }?;
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_stage_bar(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

fn self_test() -> (usize, usize) {
    use crate::install::stand_ins::set_handler;
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = [
        0x4B5AD0, 0x4B5B17, 0x4B5B5E, 0x4B5BA5, 0x4B5BEC, 0x4B5CD1, 0x4B5CFE, 0x4B5D35, 0x4B5D6D,
        0x4B5DB9, 0x4B5E0C,
    ]
    .iter()
    .map(|a| a - 0x4B5A86)
    .collect();
    // mov g_hudQuad, 0E0h
    let entry: &[u8] = &[0xC7, 0x05, 0xB0, 0xBF, 0x75, 0x00, 0xE0, 0x00, 0x00, 0x00];
    let copy = match copy(0x4B5A86, 0x3A3, entry, &calls, &[(0x4B7216, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(note_quad);
    let regions: Vec<Region> = vec![
        (game_addr(QUAD_RVA) as u32, 0x38),
        at(COLOUR_RVA),
        (game_addr(PLACES_RVA[0]) as u32, 0x14),
    ];
    let mut vars = regions.clone();
    vars.push((game_addr(MILES_RVA) as u32, 0x200));
    let saved = capture(&vars);
    let regs = |r: &Regs| [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp];
    for case in 0..4u32 {
        let mut record = [0u32; 0x40];
        record[0x74 / 4] = [0u32, 1, 3, 7][case as usize];
        record[0x68 / 4] = 12.5f32.to_bits();
        let before = Regs {
            ebp: record.as_ptr() as u32,
            ebx: 0x1000,
            ..Regs::SENTINEL
        };
        let prepare = || unsafe {
            restore(&vars, &saved);
            for n in 0..PARTS as usize {
                game_write(MILES_RVA + 4 * n, (1.0f32 + n as f32 * 0.5).to_bits());
                game_write(KINDS_RVA + 4 * n, (n as u32 % 4) * 4);
            }
        };
        prepare();
        take_quads();
        let mut ro = before;
        unsafe { crate::framework::call_game(&mut ro, copy.addr()) };
        let theirs = (Some(regs(&ro)), take_quads(), capture(&regions));
        prepare();
        let mine = unsafe {
            draw_stage_bar(&before, &mut |p, r| {
                note_quad(p, r);
                Ok(())
            })
        };
        let mine = (
            mine.as_ref().ok().map(regs),
            take_quads(),
            capture(&regions),
        );
        t.check(theirs == mine, || {
            format!("case {case}: original {:x?} {} quads; rust {:x?} {} quads; same quads {}; same state {}", theirs.0, theirs.1.len(), mine.0, mine.1.len(), theirs.1 == mine.1, theirs.2 == mine.2)
        });
    }
    restore(&vars, &saved);
    t.finish()
}
