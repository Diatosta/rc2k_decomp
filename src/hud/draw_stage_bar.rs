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
//! against the Rust version, parts by lot, at
//! both test control words (the test's values are exact at both).

use crate::errors::Refused;
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::hud::add_quad::{QUAD_RVA, note_quad, take_quads};
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
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
pub const PARTS: u32 = rc2k_core::hud::stage_bars::PARTS;

/// A call by place: 1 `add_solid_quad`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

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

/// The bars of `kinds`' players (each player's colour into `colour_vars`), `dy` lower
/// (see the header and `hud::draw_stage_bars_1`'s): the registers they leave.
// Converted: `rc2k_core::hud::stage_bars`
pub unsafe fn bars(
    entry: &Regs,
    kinds: &[usize],
    colour_vars: &[usize],
    dy: u32,
    part: Part,
) -> Result<Regs, Refused> {
    use rc2k_core::hud::stage_bars as sb;
    let mut part = part;
    let mut g = *entry;
    unsafe {
        let get = |o: u32| ((entry.ebp + o) as *const u32).read_unaligned();
        let f = |r: usize| f32::from_bits(game_read(r));
        let count = get(0x74);
        sb::check(count, kinds.len() as u32, |p, n| {
            game_read(kinds[p as usize] + 4 * n as usize)
        })?;
        set(0, sb::SOLID);
        for (colour, c) in sb::frame(dy, kinds.len() == 2) {
            quad(&g, Some(colour), c, &mut part)?;
        }
        g.ecx = count;
        if count == 0 {
            return Ok(g);
        }
        game_write(WIDTH_RVA, sb::WIDTH);
        game_write(ENDS_RVA[0], 0u32);
        game_write(ENDS_RVA[1], 0u32);
        let total = f32::from_bits(get(0x68));
        for n in 0..count {
            let left = count - n;
            let colours: Vec<u32> = kinds
                .iter()
                .map(|&k| {
                    let kind: u32 = game_read(k + 4 * n as usize);
                    ((game_addr(COLOURS_RVA) as u32 + kind) as *const u32).read_unaligned()
                })
                .collect();
            for (&to, &c) in colour_vars.iter().zip(&colours) {
                game_write(to, c);
            }
            let end = sb::part_end(
                f(MILES_RVA + 4 * n as usize),
                f(ENDS_RVA[1]),
                left == 1,
                total,
            );
            game_write(ENDS_RVA[1], end.to_bits());
            let a = sb::place(sb::WIDTH, f(ENDS_RVA[0]), total);
            game_write(PLACES_RVA[0], a);
            let b = sb::place(sb::WIDTH, end, total);
            game_write(PLACES_RVA[1], b);
            let mut r = Regs { ecx: left, ..g };
            for q in sb::part_quads(a, b, &colours, left == 1, dy) {
                if let Some(alpha) = q.alpha {
                    set(0, alpha);
                }
                [r.eax, r.ebx] = [q.eax, q.ebx];
                if let Some(edx) = q.edx {
                    r.edx = edx;
                }
                quad(&r, q.colour, q.corners, &mut part)?;
            }
            game_write(ENDS_RVA[0], end.to_bits());
        }
        g.ebx = count;
        g.ecx = 0;
    }
    Ok(g)
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::stage_bars` (through `bars`)
pub unsafe fn draw_stage_bar(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { bars(entry, &[KINDS_RVA], &[COLOUR_RVA], 0, part) }
}

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
    for (word, case) in TEST_CONTROL_WORDS
        .into_iter()
        .flat_map(|w| (0..4u32).map(move |c| (w, c)))
    {
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
        with_control_word(word, &mut || unsafe {
            crate::framework::call_game(&mut ro, copy.addr())
        });
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
            format!("control word {word:#x}, case {case}: original {:x?} {} quads; rust {:x?} {} quads; same quads {}; same state {}", theirs.0, theirs.1.len(), mine.0, mine.1.len(), theirs.1 == mine.1, theirs.2 == mine.2)
        });
    }
    restore(&vars, &saved);
    t.finish()
}
