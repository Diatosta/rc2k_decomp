//! `hud::draw_stage_bars_1` (0x4B5E29): the stage's progress bars of the split layout,
//! one row a player: as `hud::draw_stage_bar` with a frame of seven quads and each part
//! twice (rows 2 .. 0xE and 0x10 .. 0x1C), the second player's colours from
//! `dword_6D170C` (into `dword_75BFA8`; the first's into `dword_75BFA4`).
//! `hud::draw_stage_bars_2` (0x4B63C2) is the same 0x11E lower.
//!
//! In:  ebp the HUD's record
//! Out: ebx the parts drawn (kept with none), ecx 0; the rest kept
//!
//! Caller: layout 1's drawer (0x4B27F4). The quads through `part` (1 `add_solid_quad`),
//! in the game its Rust version. Refuses more than 0x40 parts and a colour offset past
//! `dword_75BF70`'s four, before anything. Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose quads go to a stand-in noting the quad record at each,
//! against the Rust version, parts by lot, at
//! both test control words (the test's values are exact at both).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva, shadow};
use crate::hud::add_quad::{QUAD_RVA, note_quad, take_quads};
use crate::hud::draw_stage_bar::*;
use crate::math::x87_run::{TEST_CONTROL_WORDS, with_control_word};
use crate::service::harness::{Region, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("hud::draw_stage_bars_1", rva(0x4B5E29), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KINDS_2_RVA: usize = rva(0x6D170C); // dword_6D170C: 0x40
const COLOUR_A_RVA: usize = rva(0x75BFA4); // dword_75BFA4
const COLOUR_B_RVA: usize = rva(0x75BFA8); // dword_75BFA8
pub const CALLS: [usize; 19] = [
    0x4A, 0x91, 0xD8, 0x11F, 0x166, 0x1AD, 0x1F4, 0x2EB, 0x318, 0x34F, 0x387, 0x3BF, 0x3EC, 0x423,
    0x45B, 0x4A7, 0x4E7, 0x53E, 0x57C,
];

/// The two players' bars `dy` lower (see the header): the registers they leave.
pub unsafe fn bars_with(entry: &Regs, dy: u32, part: Part) -> Result<Regs, Refused> {
    unsafe {
        bars(
            entry,
            &[KINDS_RVA, KINDS_2_RVA],
            &[COLOUR_A_RVA, COLOUR_B_RVA],
            dy,
            part,
        )
    }
}

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::hud::stage_bars` (through `draw_stage_bar::bars`)
pub unsafe fn draw_stage_bars_1(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { bars_with(entry, 0, part) }
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { draw_stage_bars_1(r, &mut game_part) } {
        Ok(left) => *r = Regs { esp: r.esp, ..left },
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

/// The test of the split bars: a copy whose quads go to the stand-in, against `rust`.
pub fn test_bars(
    hook: &RawHook,
    address: usize,
    rust: &dyn Fn(&Regs, Part) -> Result<Regs, Refused>,
) -> (usize, usize) {
    use crate::install::stand_ins::set_handler;
    use crate::race::mud::copy;
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    // mov g_hudQuad, 0E0h
    let entry: &[u8] = &[0xC7, 0x05, 0xB0, 0xBF, 0x75, 0x00, 0xE0, 0x00, 0x00, 0x00];
    let copy = match copy(address, 0x599, entry, &CALLS, &[(0x4B7216, 1)]) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    set_handler(note_quad);
    let regions: Vec<Region> = vec![
        (game_addr(QUAD_RVA) as u32, 0x38),
        (game_addr(COLOUR_A_RVA) as u32, 8),
        (game_addr(PLACES_RVA[0]) as u32, 0x14),
    ];
    let mut vars = regions.clone();
    vars.push((game_addr(MILES_RVA) as u32, 0x300));
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
                game_write(KINDS_2_RVA + 4 * n, ((n as u32 + 1) % 4) * 4);
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
        let mine = rust(&before, &mut |p, r| {
            note_quad(p, r);
            Ok(())
        });
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

fn self_test() -> (usize, usize) {
    test_bars(&HOOK, 0x4B5E29, &|r, p| unsafe { draw_stage_bars_1(r, p) })
}
