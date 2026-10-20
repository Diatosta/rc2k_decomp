//! `text::glyph_kerning` (0x402577): updates the kerning row table for one glyph and returns
//! how far the glyph can move left.
//!
//! In:  eax = glyph index, esi = glyph table
//! Out: eax = kerning (amount to subtract from x); everything else preserved

use crate::bridge::{Raw, read_through};
use crate::framework::{EAX, RawHook, Regs, SelfTest, game_ptr, hook_asm, rva, shadow};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("text::glyph_kerning", rva(0x402577), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const STATE_RVA: usize = rva(0x93E830); // g_textLineState: the kerning row table
pub const STATE_LEN: usize = rc2k_core::text::KERNING_ROWS * 2;
/// The kerning row table: for each of 64 rows, trailing blank pixels and gap.
pub type State = rc2k_core::text::Kerning;

pub fn state_ptr() -> *mut State {
    game_ptr(STATE_RVA)
}

// ---- Rust version ----

/// Glyph record = table + (dword at table + 4*index), as `add esi, [esi+eax*4]` does.
pub unsafe fn glyph_ptr(table: *const u8, index: u32) -> *const u8 {
    unsafe {
        let off = table
            .wrapping_add((index as usize).wrapping_mul(4))
            .cast::<u32>()
            .read_unaligned();
        table.wrapping_add(off as usize)
    }
}

/// Updates `state` for one glyph record and returns the amount to subtract from x
/// (converted: `rc2k_core::text::add_record`). Rows past the 64th are left out, while the
/// original would write outside the table.
pub unsafe fn glyph_kerning(glyph: *const u8, state: &mut State) -> u32 {
    let record = unsafe { Raw::new(glyph) };
    read_through(rc2k_core::text::add_record(state, &record))
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let glyph = unsafe { glyph_ptr(r.esi as usize as *const u8, r.eax) };

    if !shadow() {
        r.eax = unsafe { glyph_kerning(glyph, &mut *state_ptr()) };
        return;
    }

    // Shadow: Rust works on a copy, the original changes the real table, then compare.
    let mut local = unsafe { *state_ptr() };
    let mine = unsafe { glyph_kerning(glyph, &mut local) };
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let after = unsafe { *state_ptr() };
    if o.eax != mine || after != local {
        HOOK.mismatch(|| {
            format!(
                "glyph={}: original={} rust={mine} state_equal={}",
                r.eax,
                o.eax,
                after == local
            )
        });
    }
    r.eax = o.eax;
}

// ---- self-test: synthetic glyph table (the original only reads through esi) ----

pub(crate) fn row(runs: &[(u8, &[u8])]) -> Vec<u8> {
    let mut b = Vec::new();
    for &(gap, px) in runs {
        b.push(px.len() as u8);
        b.push(gap);
        b.extend_from_slice(px);
    }
    b.push(0);
    b
}

pub(crate) fn glyph(width: u8, lead: u8, adv: u8, rows: &[Vec<u8>]) -> Vec<u8> {
    let mut g = vec![width, lead, adv, rows.len() as u8];
    for r in rows {
        g.extend_from_slice(r);
    }
    g
}

/// n offsets (relative to the table start) followed by the glyph records.
pub(crate) fn build_table(glyphs: &[Vec<u8>]) -> Vec<u8> {
    let (mut offsets, mut body) = (Vec::new(), Vec::new());
    let base = glyphs.len() * 4;
    for g in glyphs {
        offsets.extend_from_slice(&((base + body.len()) as u32).to_le_bytes());
        body.extend_from_slice(g);
    }
    offsets.extend(body);
    offsets
}

pub(crate) fn test_glyphs() -> Vec<Vec<u8>> {
    vec![
        glyph(5, 0, 0, &[]), // 0: no contour data
        glyph(
            8,
            2,
            8,
            &[
                // 1: top rows, two runs, an empty row
                row(&[(1, &[1, 2, 3])]),
                row(&[(0, &[9]), (2, &[7, 7])]),
                row(&[]),
            ],
        ),
        glyph(
            6,
            0,
            6,
            &(0..8u8)
                .map(|i| row(&[(i % 3, &[1u8, 2][..(i % 2 + 1) as usize])]))
                .collect::<Vec<_>>(),
        ),
        glyph(
            9,
            10,
            9,
            &(0..20u8)
                .map(|i| row(&[(i % 4, &[5; 3])]))
                .collect::<Vec<_>>(),
        ), // rows left below
        glyph(
            7,
            0,
            7,
            &(0..64u8).map(|i| row(&[(1, &[i])])).collect::<Vec<_>>(),
        ), // fills all 64 rows
        glyph(3, 1, 1, &[row(&[(0, &[4])])]), // odd advance: >> 1
        glyph(
            10,
            3,
            200,
            &[
                // byte wraparound
                row(&[(150, &[1; 60])]),
                row(&[(250, &[2; 30]), (100, &[3; 80])]),
            ],
        ),
    ]
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let glyphs = test_glyphs();
    let table = build_table(&glyphs);
    let tp = table.as_ptr();

    // The original rewrites live renderer state: save it and restore it afterwards.
    let saved = unsafe { *state_ptr() };

    let regs_for = |index: u32| Regs {
        eax: index,
        esi: tp as u32,
        ..Regs::SENTINEL
    };
    let call_orig = |index: u32| -> (Regs, State) {
        let mut ro = regs_for(index);
        unsafe { call_original(&mut ro) };
        (ro, unsafe { *state_ptr() })
    };
    let regs_ok = |ro: &Regs, index: u32| ro.preserved_except(&regs_for(index), EAX);

    // Single glyphs from different starting states.
    let patterns: [State; 3] = [
        [0; STATE_LEN],
        std::array::from_fn(|i| (i as u8).wrapping_mul(37).wrapping_add(11)),
        [0xFF; STATE_LEN],
    ];
    for pattern in &patterns {
        for index in 0..glyphs.len() as u32 {
            unsafe { *state_ptr() = *pattern };
            let (ro, after) = call_orig(index);
            let mut local = *pattern;
            let mine = unsafe { glyph_kerning(glyph_ptr(tp, index), &mut local) };
            t.check(
                ro.eax == mine && after == local && regs_ok(&ro, index),
                || {
                    format!(
                        "glyph={index} original={} rust={mine} state_equal={} regs_ok={}",
                        ro.eax,
                        after == local,
                        regs_ok(&ro, index)
                    )
                },
            );
        }
    }

    // Sequences: the table carries over from glyph to glyph, like a rendered line.
    for seq in [
        &[1u32, 2, 0, 3, 4, 5, 6, 2, 1][..],
        &[6, 6, 6, 1, 5, 5][..],
        &[4, 3, 2, 1, 0][..],
    ] {
        unsafe { *state_ptr() = [0; STATE_LEN] };
        let mut local = [0u8; STATE_LEN];
        let mut failed = None;
        for (step, &index) in seq.iter().enumerate() {
            let (ro, after) = call_orig(index);
            let mine = unsafe { glyph_kerning(glyph_ptr(tp, index), &mut local) };
            if ro.eax != mine || after != local || !regs_ok(&ro, index) {
                failed = Some((step, index, ro.eax, mine));
                break;
            }
        }
        t.check(failed.is_none(), || {
            let (step, index, orig, mine) = failed.unwrap_or_default();
            format!("sequence {seq:?} at step {step} (glyph {index}): original={orig} rust={mine}")
        });
    }

    unsafe { *state_ptr() = saved };
    t.finish()
}
