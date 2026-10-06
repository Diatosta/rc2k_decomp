use crate::framework::{MAX_LOGGED, RawHook, Regs, SHADOW_MODE, game_addr, hook_asm, log};
use std::sync::atomic::{AtomicUsize, Ordering};

static TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);

const STATE_RVA: usize = 0x93E830 - 0x400000; // g_textLineState
const ROWS: usize = 64;
const STATE_LEN: usize = ROWS * 2;
type State = [u8; STATE_LEN];

pub static HOOK: RawHook = RawHook::new(
    "text::glyph_kerning",
    0x402577 - 0x400000,
    thunk,
    &TRAMPOLINE,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = TRAMPOLINE
);

fn state_ptr() -> *mut State {
    game_addr(STATE_RVA) as *mut State
}

/// Glyph record = table + (dword at table + 4*index), as `add esi, [esi+eax*4]` does.
unsafe fn glyph_ptr(table: *const u8, index: u32) -> *const u8 {
    unsafe {
        let off = table
            .wrapping_add((index as usize).wrapping_mul(4))
            .cast::<u32>()
            .read_unaligned();
        table.wrapping_add(off as usize)
    }
}

fn put(state: &mut State, row: usize, trailing: u8, gap: u8) {
    if row < ROWS {
        state[row * 2] = trailing;
        state[row * 2 + 1] = gap;
    }
}
fn trailing_of(state: &State, row: usize) -> u8 {
    if row < ROWS { state[row * 2] } else { 0 }
}

/// Pure core: updates `state` for one glyph and returns the amount to subtract from x.
/// Malformed glyphs (top rows + data rows > 64, or 0 data rows) are clamped, while the
/// original would write outside the table or loop 256 times.
unsafe fn kerning(glyph: *const u8, state: &mut State) -> u32 {
    unsafe {
        let adv = *glyph.add(2);
        if adv == 0 {
            return 0; // no contour data: table untouched
        }
        let mut row = 0usize;

        for _ in 0..*glyph.add(1) {
            let old = trailing_of(state, row);
            put(state, row, 0, old.wrapping_add(adv));
            row += 1;
        }

        let mut p = glyph.add(4);
        for _ in 0..*glyph.add(3) {
            let (mut lead_gap, mut trail) = (adv, adv);
            let mut count = *p;
            p = p.add(1);
            if count != 0 {
                lead_gap = *p;
                p = p.add(1);
                trail = adv.wrapping_sub(count).wrapping_sub(lead_gap);
                p = p.add(count as usize);
                loop {
                    count = *p;
                    p = p.add(1);
                    if count == 0 {
                        break;
                    }
                    let gap = *p;
                    p = p.add(1);
                    trail = trail.wrapping_sub(count).wrapping_sub(gap);
                    p = p.add(count as usize);
                }
            }
            let old = trailing_of(state, row);
            put(state, row, trail, lead_gap.wrapping_add(old));
            row += 1;
        }

        let half = adv >> 1;
        while row < ROWS {
            let old = trailing_of(state, row);
            put(state, row, half, old);
            row += 1;
        }

        (0..ROWS).map(|r| state[r * 2 + 1]).min().unwrap_or(0xFF) as u32
    }
}

// eax = glyph index, esi = glyph table -> eax = kerning; everything else preserved
extern "C" fn dispatch(r: &mut Regs) {
    HOOK.stats.calls.fetch_add(1, Ordering::Relaxed);
    let glyph = unsafe { glyph_ptr(r.esi as usize as *const u8, r.eax) };

    if !SHADOW_MODE.load(Ordering::Relaxed) {
        r.eax = unsafe { kerning(glyph, &mut *state_ptr()) };
        return;
    }

    // Shadow: Rust works on a copy, the original changes the real table, then compare.
    let mut local = unsafe { *state_ptr() };
    let mine = unsafe { kerning(glyph, &mut local) };
    let mut o = *r;
    unsafe { call_original(&mut o) };
    let after = unsafe { *state_ptr() };
    if o.eax != mine || after != local {
        let n = HOOK.stats.mismatches.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LOGGED {
            log(&format!(
                "MISMATCH glyph_kerning(glyph={}): original={} rust={mine} state_equal={}",
                r.eax,
                o.eax,
                after == local
            ));
        }
    }
    r.eax = o.eax;
}

// ---- self-test: synthetic glyph table (the original only reads through esi) ----

fn row(runs: &[(u8, &[u8])]) -> Vec<u8> {
    let mut b = Vec::new();
    for &(gap, px) in runs {
        b.push(px.len() as u8);
        b.push(gap);
        b.extend_from_slice(px);
    }
    b.push(0);
    b
}

fn glyph(width: u8, lead: u8, adv: u8, rows: &[Vec<u8>]) -> Vec<u8> {
    let mut g = vec![width, lead, adv, rows.len() as u8];
    for r in rows {
        g.extend_from_slice(r);
    }
    g
}

/// n offsets (relative to the table start) followed by the glyph records.
fn build_table(glyphs: &[Vec<u8>]) -> Vec<u8> {
    let (mut offsets, mut body) = (Vec::new(), Vec::new());
    let base = glyphs.len() * 4;
    for g in glyphs {
        offsets.extend_from_slice(&((base + body.len()) as u32).to_le_bytes());
        body.extend_from_slice(g);
    }
    offsets.extend(body);
    offsets
}

fn test_glyphs() -> Vec<Vec<u8>> {
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

pub fn self_test() -> (usize, usize) {
    if TRAMPOLINE.load(Ordering::Relaxed) == 0 {
        log("glyph_kerning selftest SKIPPED: original not available");
        return (0, 0);
    }
    let glyphs = test_glyphs();
    let table = build_table(&glyphs);
    let tp = table.as_ptr();

    // The original rewrites live renderer state: save it and restore it afterwards.
    let saved = unsafe { *state_ptr() };
    let (mut cases, mut failures) = (0, 0);

    let call_orig = |index: u32| -> (Regs, State) {
        let mut ro = Regs {
            eax: index,
            esi: tp as u32,
            ecx: 0x1111_1111,
            edx: 0x3333_3333,
            ebx: 0x4444_4444,
            edi: 0x2222_2222,
            ebp: 0x6666_6666,
            ..Regs::default()
        };
        unsafe { call_original(&mut ro) };
        (ro, unsafe { *state_ptr() })
    };
    let regs_ok = |ro: &Regs| {
        ro.esi == tp as u32
            && ro.ecx == 0x1111_1111
            && ro.edx == 0x3333_3333
            && ro.ebx == 0x4444_4444
            && ro.edi == 0x2222_2222
            && ro.ebp == 0x6666_6666
    };

    // Single glyphs from different starting states.
    let patterns: [State; 3] = [
        [0; STATE_LEN],
        std::array::from_fn(|i| (i as u8).wrapping_mul(37).wrapping_add(11)),
        [0xFF; STATE_LEN],
    ];
    for pattern in &patterns {
        for index in 0..glyphs.len() as u32 {
            cases += 1;
            unsafe { *state_ptr() = *pattern };
            let (ro, after) = call_orig(index);
            let mut local = *pattern;
            let mine = unsafe { kerning(glyph_ptr(tp, index), &mut local) };
            if ro.eax != mine || after != local || !regs_ok(&ro) {
                failures += 1;
                log(&format!(
                    "glyph_kerning selftest FAILED: glyph={index} original={} rust={mine} state_equal={} regs_ok={}",
                    ro.eax,
                    after == local,
                    regs_ok(&ro)
                ));
            }
        }
    }

    // Sequences: the table carries over from glyph to glyph, like a rendered line.
    for seq in [
        &[1u32, 2, 0, 3, 4, 5, 6, 2, 1][..],
        &[6, 6, 6, 1, 5, 5][..],
        &[4, 3, 2, 1, 0][..],
    ] {
        cases += 1;
        unsafe { *state_ptr() = [0; STATE_LEN] };
        let mut local = [0u8; STATE_LEN];
        for (step, &index) in seq.iter().enumerate() {
            let (ro, after) = call_orig(index);
            let mine = unsafe { kerning(glyph_ptr(tp, index), &mut local) };
            if ro.eax != mine || after != local || !regs_ok(&ro) {
                failures += 1;
                log(&format!(
                    "glyph_kerning selftest FAILED: sequence {seq:?} at step {step} (glyph {index}): original={} rust={mine}",
                    ro.eax
                ));
                break;
            }
        }
    }

    unsafe { *state_ptr() = saved };
    (cases, failures)
}
